use alloc::{boxed::Box, sync::Arc, vec::Vec};
use bitflags::bitflags;
use core::{cell::UnsafeCell, ffi::c_void};
use esp_idf_svc::sys;
#[cfg(not(cpfd))]
use zerocopy::IntoBytes;

use crate::{
  ble,
  cpfd::Cpfd,
  utilities::{
    ble_npl_hw_enter_critical, ble_npl_hw_exit_critical, mutex::Mutex, voidp_to_ref, BleUuid,
    OsMBuf,
  },
  AttValue, BLEConnDesc, BLEDescriptor, BLEDevice, BLEError, DescriptorProperties, OnWriteArgs,
};

cfg_if::cfg_if! {
  if #[cfg(any(
    all(
      esp_idf_version_major = "5",
      esp_idf_version_minor = "2",
      not(any(esp_idf_version_patch = "0", esp_idf_version_patch = "1", esp_idf_version_patch="2"))),
    all(
      esp_idf_version_major = "5",
      esp_idf_version_minor = "3",
      not(any(esp_idf_version_patch = "0", esp_idf_version_patch = "1"))),
    all(
      esp_idf_version_major = "5",
      esp_idf_version_minor = "4"),
  ))] {
    type NotifyTxType = sys::ble_gap_event__bindgen_ty_1__bindgen_ty_12;
    type Subscribe = sys::ble_gap_event__bindgen_ty_1__bindgen_ty_13;
  } else {
    type NotifyTxType = sys::ble_gap_event__bindgen_ty_1__bindgen_ty_11;
    type Subscribe = sys::ble_gap_event__bindgen_ty_1__bindgen_ty_12;
  }
}

const NULL_HANDLE: u16 = 0xFFFF;

cfg_if::cfg_if! {
  if #[cfg(any(
    all(
      esp_idf_version_major = "5",
      esp_idf_version_minor = "4",
      not(any(esp_idf_version_patch = "0", esp_idf_version_patch = "1"))),
  ))] {
    type NimblePropertiesType = u32;
  } else {
    type NimblePropertiesType = u16;
  }
}

bitflags! {
  #[repr(transparent)]
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub struct NimbleProperties: NimblePropertiesType {
    /// Read Access Permitted
    const READ = sys::BLE_GATT_CHR_F_READ as _;
    /// Read Requires Encryption
    const READ_ENC = sys::BLE_GATT_CHR_F_READ_ENC as _;
    /// Read requires Authentication
    const READ_AUTHEN = sys::BLE_GATT_CHR_F_READ_AUTHEN as _;
    /// Read requires Authorization
    const READ_AUTHOR = sys::BLE_GATT_CHR_F_READ_AUTHOR as _;
    /// Write Permited
    const WRITE = sys::BLE_GATT_CHR_F_WRITE as _;
    /// Write with no Ack Response
    const WRITE_NO_RSP = sys::BLE_GATT_CHR_F_WRITE_NO_RSP as _;
    /// Write Requires Encryption
    const WRITE_ENC = sys::BLE_GATT_CHR_F_WRITE_ENC as _;
    /// Write requires Authentication
    const WRITE_AUTHEN = sys::BLE_GATT_CHR_F_WRITE_AUTHEN as _;
    /// Write requires Authorization
    const WRITE_AUTHOR = sys::BLE_GATT_CHR_F_WRITE_AUTHOR as _;
    /// Broadcasts are included in the advertising data
    const BROADCAST = sys::BLE_GATT_CHR_F_BROADCAST as _;
    /// Notifications are Sent from Server to Client with no Response
    const NOTIFY = sys::BLE_GATT_CHR_F_NOTIFY as _;
    /// Indications are Sent from Server to Client where Server expects a Response
    const INDICATE = sys::BLE_GATT_CHR_F_INDICATE as _;
  }
}

#[derive(PartialEq, Debug)]
pub enum NotifyTxStatus {
  SuccessIndicate,
  SuccessNotify,
  ErrorIndicateDisabled,
  ErrorNotifyDisabled,
  ErrorGatt,
  ErrorNoClient,
  ErrorIndicateTimeout,
  ErrorIndicateFailure,
}

pub struct NotifyTx<'a> {
  pub(crate) notify_tx: &'a NotifyTxType,
}

impl NotifyTx<'_> {
  pub fn status(&self) -> NotifyTxStatus {
    if self.notify_tx.indication() > 0 {
      match self.notify_tx.status as _ {
        sys::BLE_HS_EDONE => NotifyTxStatus::SuccessIndicate,
        sys::BLE_HS_ETIMEOUT => NotifyTxStatus::ErrorIndicateTimeout,
        _ => NotifyTxStatus::ErrorIndicateFailure,
      }
    } else {
      #[allow(clippy::collapsible_else_if)]
      if self.notify_tx.status == 0 {
        NotifyTxStatus::SuccessNotify
      } else {
        NotifyTxStatus::ErrorGatt
      }
    }
  }

  pub fn desc(&self) -> Result<BLEConnDesc, crate::BLEError> {
    crate::utilities::ble_gap_conn_find(self.notify_tx.conn_handle)
  }
}

bitflags! {
  #[repr(transparent)]
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  ///Empty NimbleSub i.e. `NimbleSub::is_empty()==true` means Unsubscribe(d)
  pub struct NimbleSub: u16 {
    /// Subscribe if Notify
    const NOTIFY = 0x0001;
    /// Subscribe if Indicate
    const INDICATE = 0x0002;
  }
}

#[allow(clippy::type_complexity)]
pub struct BLECharacteristic {
  pub(crate) uuid: sys::ble_uuid_any_t,
  pub(crate) handle: u16,
  pub(crate) properties: NimbleProperties,
  value: AttValue,
  on_read: Option<Box<dyn FnMut(&mut Self, &BLEConnDesc) + Send + Sync>>,
  on_write: Option<Box<dyn FnMut(&mut OnWriteArgs) + Send + Sync>>,
  pub(crate) on_notify_tx: Option<Box<dyn FnMut(NotifyTx) + Send + Sync>>,
  descriptors: Vec<Arc<Mutex<BLEDescriptor>>>,
  svc_def_descriptors: Vec<sys::ble_gatt_dsc_def>,
  subscribed_list: Vec<(u16, NimbleSub)>,
  on_subscribe: Option<Box<dyn FnMut(&Self, &BLEConnDesc, NimbleSub) + Send + Sync>>,
  #[cfg(cpfd)]
  pub(crate) cpfd: [sys::ble_gatt_cpfd; 2],
}

impl BLECharacteristic {
  pub(crate) fn new(uuid: BleUuid, properties: NimbleProperties) -> Self {
    Self {
      uuid: sys::ble_uuid_any_t::from(uuid),
      handle: NULL_HANDLE,
      properties,
      value: AttValue::new(),
      on_read: None,
      on_write: None,
      on_notify_tx: None,
      descriptors: Vec::new(),
      svc_def_descriptors: Vec::new(),
      subscribed_list: Vec::new(),
      on_subscribe: None,
      #[cfg(cpfd)]
      cpfd: [Default::default(); 2],
    }
  }

  pub fn uuid(&self) -> BleUuid {
    BleUuid::from(self.uuid)
  }

  pub fn set_value(&mut self, value: &[u8]) -> &mut Self {
    self.value.set_value(value);
    self
  }

  #[deprecated(note = "Please use `set_value` + zerocopy::IntoBytes")]
  pub fn set_from<T: Sized>(&mut self, value: &T) -> &mut Self {
    #[allow(deprecated)]
    self.value.set_from(value);
    self
  }

  pub fn value_mut(&mut self) -> &mut AttValue {
    &mut self.value
  }

  pub fn on_read(
    &mut self,
    callback: impl FnMut(&mut Self, &BLEConnDesc) + Send + Sync + 'static,
  ) -> &mut Self {
    self.on_read = Some(Box::new(callback));
    self
  }

  /// This characteristic is locked while the callback is executing. If you call `.lock()` on this characteristic from inside the callback, it will never execute.
  pub fn on_write(
    &mut self,
    callback: impl FnMut(&mut OnWriteArgs) + Send + Sync + 'static,
  ) -> &mut Self {
    self.on_write = Some(Box::new(callback));
    self
  }

  pub fn on_notify_tx(
    &mut self,
    callback: impl FnMut(NotifyTx) + Send + Sync + 'static,
  ) -> &mut Self {
    self.on_notify_tx = Some(Box::new(callback));
    self
  }

  pub fn create_descriptor(
    &mut self,
    uuid: BleUuid,
    properties: DescriptorProperties,
  ) -> Arc<Mutex<BLEDescriptor>> {
    if uuid == BleUuid::Uuid16(sys::BLE_GATT_DSC_CLT_CFG_UUID16 as _) {
      panic!("0x2902 descriptors cannot be manually created");
    }

    let descriptor = Arc::new(Mutex::new(BLEDescriptor::new(uuid, properties)));
    self.descriptors.push(descriptor.clone());
    descriptor
  }

  pub(crate) fn construct_svc_def_descriptors(&mut self) -> *mut sys::ble_gatt_dsc_def {
    if self.descriptors.is_empty() {
      return core::ptr::null_mut();
    }
    self.svc_def_descriptors.clear();

    for dsc in &mut self.descriptors {
      let arg = unsafe { Arc::get_mut_unchecked(dsc) } as *mut Mutex<BLEDescriptor>;
      let dsc = dsc.lock();
      self.svc_def_descriptors.push(sys::ble_gatt_dsc_def {
        uuid: unsafe { &dsc.uuid.u },
        att_flags: dsc.properties.bits(),
        min_key_size: 0,
        access_cb: Some(BLEDescriptor::handle_gap_event),
        arg: arg as _,
      });
    }
    self
      .svc_def_descriptors
      .push(sys::ble_gatt_dsc_def::default());
    self.svc_def_descriptors.as_mut_ptr()
  }

  pub fn notify_with(&self, value: &[u8], conn_handle: u16) -> Result<(), BLEError> {
    if let Some((_, flag)) = self.subscribed_list.iter().find(|x| x.0 == conn_handle) {
      self.send_value(value, conn_handle, *flag)
    } else {
      BLEError::convert(sys::BLE_HS_EINVAL)
    }
  }

  pub fn notify(&self) {
    for it in &self.subscribed_list {
      if let Err(err) = self.send_value(self.value.as_slice(), it.0, it.1) {
        ::log::warn!("notify error({}): {:?}", it.0, err);
      }
    }
  }

  fn send_value(&self, value: &[u8], conn_handle: u16, flag: NimbleSub) -> Result<(), BLEError> {
    let mtu = unsafe { sys::ble_att_mtu(conn_handle) - 3 };
    if mtu == 0 || flag.is_empty() {
      return BLEError::convert(sys::BLE_HS_EINVAL);
    }
    let server = BLEDevice::take().get_server();

    if flag.contains(NimbleSub::INDICATE) && self.properties.contains(NimbleProperties::INDICATE) {
      if !server.set_indicate_wait(conn_handle) {
        ::log::error!("prior Indication in progress");
        return BLEError::convert(sys::BLE_HS_EBUSY);
      }

      let om = OsMBuf::from_flat(value);
      let rc = unsafe { sys::ble_gatts_indicate_custom(conn_handle, self.handle, om.0) };
      if rc != 0 {
        server.clear_indicate_wait(conn_handle);
      }
      BLEError::convert(rc as _)
    } else if flag.contains(NimbleSub::NOTIFY) && self.properties.contains(NimbleProperties::NOTIFY)
    {
      let om = OsMBuf::from_flat(value);
      ble!(unsafe { sys::ble_gatts_notify_custom(conn_handle, self.handle, om.0) })
    } else {
      BLEError::convert(sys::BLE_HS_EINVAL)
    }
  }

  #[cfg(cpfd)]
  /// Set the Characteristic Presentation Format.
  pub fn cpfd(&mut self, cpfd: Cpfd) {
    if cpfd.name_space == (sys::BLE_GATT_CHR_NAMESPACE_BT_SIG as _) {
      debug_assert!(cpfd.description <= (sys::BLE_GATT_CHR_BT_SIG_DESC_EXTERNAL as _));
    }

    self.cpfd[0].format = cpfd.format.into();
    self.cpfd[0].exponent = cpfd.exponent;
    self.cpfd[0].unit = cpfd.unit.into();
    self.cpfd[0].name_space = cpfd.name_space;
    self.cpfd[0].description = cpfd.description;
  }

  #[cfg(not(cpfd))]
  /// Set the Characteristic Presentation Format.
  pub fn cpfd(&mut self, cpfd: Cpfd) {
    let descriptor = Arc::new(Mutex::new(BLEDescriptor::new(
      BleUuid::Uuid16(0x2904),
      DescriptorProperties::READ,
    )));
    descriptor.lock().set_value(cpfd.as_bytes());
    self.descriptors.push(descriptor);
  }

  pub(super) extern "C" fn handle_gap_event(
    conn_handle: u16,
    _attr_handle: u16,
    ctxt: *mut sys::ble_gatt_access_ctxt,
    arg: *mut c_void,
  ) -> i32 {
    let ctxt = unsafe { &*ctxt };

    let mutex = unsafe { voidp_to_ref::<Mutex<Self>>(arg) };

    if crate::utilities::ble_gap_conn_find(conn_handle).is_err() {
      ::log::warn!("the conn handle does not exist");
      return sys::BLE_ATT_ERR_UNLIKELY as _;
    }

    let mut characteristic = mutex.lock();
    if unsafe { sys::ble_uuid_cmp((*ctxt.__bindgen_anon_1.chr).uuid, &characteristic.uuid.u) != 0 }
    {
      return sys::BLE_ATT_ERR_UNLIKELY as _;
    }

    match ctxt.op as _ {
      sys::BLE_GATT_ACCESS_OP_READ_CHR => {
        let desc = crate::utilities::ble_gap_conn_find(conn_handle).unwrap();

        unsafe {
          if (*(ctxt.om)).om_pkthdr_len > 8 || characteristic.value.len() <= (desc.mtu() - 3) as _ {
            let characteristic = UnsafeCell::new(&mut characteristic);
            if let Some(callback) = &mut (&mut (*characteristic.get())).on_read {
              callback(*characteristic.get(), &desc);
            }
          }
        }

        ble_npl_hw_enter_critical();
        let value = characteristic.value.as_slice();
        let rc = OsMBuf(ctxt.om).append(value);
        ble_npl_hw_exit_critical();
        if rc == 0 {
          0
        } else {
          sys::BLE_ATT_ERR_INSUFFICIENT_RES as _
        }
      }
      sys::BLE_GATT_ACCESS_OP_WRITE_CHR => {
        let om = OsMBuf(ctxt.om);
        let buf = om.as_flat();

        let mut notify = false;

        unsafe {
          let characteristic = UnsafeCell::new(&mut characteristic);
          if let Some(callback) = &mut (&mut (*characteristic.get())).on_write {
            let desc = crate::utilities::ble_gap_conn_find(conn_handle).unwrap();
            let mut arg = OnWriteArgs {
              current_data: (&(*characteristic.get())).value.as_slice(),
              recv_data: buf.as_slice(),
              desc: &desc,
              reject: false,
              error_code: 0,
              notify: false,
            };
            callback(&mut arg);

            if arg.reject {
              return arg.error_code as _;
            }
            notify = arg.notify;
          }
        }
        characteristic.set_value(buf.as_slice());
        if notify {
          characteristic.notify();
        }

        0
      }
      _ => sys::BLE_ATT_ERR_UNLIKELY as _,
    }
  }

  pub(super) fn subscribe(&mut self, subscribe: &Subscribe) {
    let Ok(desc) = crate::utilities::ble_gap_conn_find(subscribe.conn_handle) else {
      return;
    };

    let mut sub_val = NimbleSub::empty();
    if subscribe.cur_notify() > 0 && (self.properties.contains(NimbleProperties::NOTIFY)) {
      sub_val.insert(NimbleSub::NOTIFY);
    }
    if subscribe.cur_indicate() > 0 && (self.properties.contains(NimbleProperties::INDICATE)) {
      sub_val.insert(NimbleSub::INDICATE);
    }

    if let Some(idx) = self
      .subscribed_list
      .iter()
      .position(|x| x.0 == subscribe.conn_handle)
    {
      if !sub_val.is_empty() {
        self.subscribed_list[idx].1 = sub_val;
      } else {
        self.subscribed_list.swap_remove(idx);
      }
    } else if !sub_val.is_empty() {
      self.subscribed_list.push((subscribe.conn_handle, sub_val));
    }

    unsafe {
      let self_ = UnsafeCell::new(self);
      if let Some(callback) = &mut (*self_.get()).on_subscribe {
        callback(*self_.get(), &desc, sub_val);
      }
    }
  }

  /// Do not call `lock` on this characteristic inside the callback, use the first input instead.
  /// In the future, this characteristic could be locked while the callback executes.
  /// * `callback` - Function to call when a subscription event is recieved, including subscribe and unsubscribe events
  ///   see [`crate::NimbleSub`] for event type
  pub fn on_subscribe(
    &mut self,
    callback: impl FnMut(&Self, &BLEConnDesc, NimbleSub) + Send + Sync + 'static,
  ) -> &mut Self {
    self.on_subscribe = Some(Box::new(callback));
    self
  }

  pub fn subscribed_count(&self) -> usize {
    self.subscribed_list.len()
  }
}

impl core::fmt::Debug for BLECharacteristic {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("BLECharacteristic")
      .field("uuid", &BleUuid::from(self.uuid))
      .field("properties", &self.properties)
      .finish()
  }
}

unsafe impl Send for BLECharacteristic {}
