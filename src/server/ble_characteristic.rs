use core::{cell::UnsafeCell, ffi::c_void};

use alloc::{boxed::Box, sync::Arc, vec::Vec};
use bitflags::bitflags;
use esp_idf_sys::{ble_uuid_any_t, ble_uuid_cmp};

use crate::{
  utilities::{
    as_mut_ptr, ble_npl_hw_enter_critical, ble_npl_hw_exit_critical, mutex::Mutex, os_mbuf_append,
    voidp_to_ref, BleUuid,
  },
  AttValue, BLEConnDesc, BLEDescriptor, BLEDevice, DescriptorProperties, OnWriteArgs, BLE2904,
};

const NULL_HANDLE: u16 = 0xFFFF;

bitflags! {
  #[repr(transparent)]
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub struct NimbleProperties: u16 {
    const READ = esp_idf_sys::BLE_GATT_CHR_F_READ as _;
    const READ_ENC = esp_idf_sys::BLE_GATT_CHR_F_READ_ENC as _;
    const READ_AUTHEN = esp_idf_sys::BLE_GATT_CHR_F_READ_AUTHEN as _;
    const READ_AUTHOR = esp_idf_sys::BLE_GATT_CHR_F_READ_AUTHOR as _;
    const WRITE = esp_idf_sys::BLE_GATT_CHR_F_WRITE as _;
    const WRITE_NO_RSP = esp_idf_sys::BLE_GATT_CHR_F_WRITE_NO_RSP as _;
    const WRITE_ENC = esp_idf_sys::BLE_GATT_CHR_F_WRITE_ENC as _;
    const WRITE_AUTHEN = esp_idf_sys::BLE_GATT_CHR_F_WRITE_AUTHEN as _;
    const WRITE_AUTHOR = esp_idf_sys::BLE_GATT_CHR_F_WRITE_AUTHOR as _;
    const BROADCAST = esp_idf_sys::BLE_GATT_CHR_F_BROADCAST as _;
    const NOTIFY = esp_idf_sys::BLE_GATT_CHR_F_NOTIFY as _;
    const INDICATE = esp_idf_sys::BLE_GATT_CHR_F_INDICATE as _;
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
  pub(crate) notify_tx: &'a esp_idf_sys::ble_gap_event__bindgen_ty_1__bindgen_ty_11,
}

impl NotifyTx<'_> {
  pub fn status(&self) -> NotifyTxStatus {
    if self.notify_tx.indication() > 0 {
      match self.notify_tx.status as _ {
        esp_idf_sys::BLE_HS_EDONE => NotifyTxStatus::SuccessIndicate,
        esp_idf_sys::BLE_HS_ETIMEOUT => NotifyTxStatus::ErrorIndicateTimeout,
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
  pub(crate) uuid: ble_uuid_any_t,
  pub(crate) handle: u16,
  pub(crate) properties: NimbleProperties,
  value: AttValue,
  on_read: Option<Box<dyn FnMut(&mut AttValue, &BLEConnDesc) + Send + Sync>>,
  on_write: Option<Box<dyn FnMut(&mut OnWriteArgs) + Send + Sync>>,
  pub(crate) on_notify_tx: Option<Box<dyn FnMut(NotifyTx) + Send + Sync>>,
  descriptors: Vec<Arc<Mutex<BLEDescriptor>>>,
  svc_def_descriptors: Vec<esp_idf_sys::ble_gatt_dsc_def>,
  subscribed_list: Vec<(u16, NimbleSub)>,
  on_subscribe: Option<Box<dyn FnMut(&Self, &BLEConnDesc, NimbleSub) + Send + Sync>>,
}

impl BLECharacteristic {
  pub(crate) fn new(uuid: BleUuid, properties: NimbleProperties) -> Self {
    Self {
      uuid: ble_uuid_any_t::from(uuid),
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
    }
  }

  pub fn set_value(&mut self, value: &[u8]) -> &mut Self {
    self.value.set_value(value);
    self
  }

  pub fn set_from<T: Sized>(&mut self, value: &T) -> &mut Self {
    self.value.set_from(value);
    self
  }

  pub fn value_mut(&mut self) -> &mut AttValue {
    &mut self.value
  }

  pub fn on_read(
    &mut self,
    callback: impl FnMut(&mut AttValue, &BLEConnDesc) + Send + Sync + 'static,
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
    if uuid == BleUuid::Uuid16(esp_idf_sys::BLE_GATT_DSC_CLT_CFG_UUID16 as _) {
      panic!("0x2902 descriptors cannot be manually created");
    }

    let descriptor = Arc::new(Mutex::new(BLEDescriptor::new(uuid, properties)));
    self.descriptors.push(descriptor.clone());
    descriptor
  }

  pub fn create_2904_descriptor(&mut self) -> BLE2904 {
    let descriptor = Arc::new(Mutex::new(BLEDescriptor::new(
      BleUuid::Uuid16(0x2904),
      DescriptorProperties::READ,
    )));
    self.descriptors.push(descriptor.clone());
    BLE2904::new(descriptor)
  }

  pub(crate) fn construct_svc_def_descriptors(&mut self) -> *mut esp_idf_sys::ble_gatt_dsc_def {
    if self.descriptors.is_empty() {
      return core::ptr::null_mut();
    }
    self.svc_def_descriptors.clear();

    for dsc in &self.descriptors {
      let arg = unsafe { as_mut_ptr(Arc::into_raw(dsc.clone())) };
      let dsc = dsc.lock();
      self
        .svc_def_descriptors
        .push(esp_idf_sys::ble_gatt_dsc_def {
          uuid: unsafe { &dsc.uuid.u },
          att_flags: dsc.properties.bits(),
          min_key_size: 0,
          access_cb: Some(BLEDescriptor::handle_gap_event),
          arg: arg as _,
        });
    }
    self
      .svc_def_descriptors
      .push(esp_idf_sys::ble_gatt_dsc_def::default());
    self.svc_def_descriptors.as_mut_ptr()
  }

  pub fn notify(&self) {
    if self.subscribed_list.is_empty() {
      return;
    }

    let server = BLEDevice::take().get_server();

    for it in &self.subscribed_list {
      let _mtu = unsafe { esp_idf_sys::ble_att_mtu(it.0) - 3 };
      if _mtu == 0 || it.1.is_empty() {
        continue;
      }

      if it.1.contains(NimbleSub::INDICATE) && self.properties.contains(NimbleProperties::INDICATE)
      {
        if !server.set_indicate_wait(it.0) {
          ::log::error!("prior Indication in progress");
          continue;
        }

        let om = unsafe {
          esp_idf_sys::ble_hs_mbuf_from_flat(
            self.value.value().as_ptr() as _,
            self.value.len() as _,
          )
        };

        let rc = unsafe { esp_idf_sys::ble_gattc_indicate_custom(it.0, self.handle, om) };
        if rc != 0 {
          server.clear_indicate_wait(it.0);
        }
      } else if it.1.contains(NimbleSub::NOTIFY)
        && self.properties.contains(NimbleProperties::NOTIFY)
      {
        let om = unsafe {
          esp_idf_sys::ble_hs_mbuf_from_flat(
            self.value.value().as_ptr() as _,
            self.value.len() as _,
          )
        };
        unsafe { esp_idf_sys::ble_gattc_notify_custom(it.0, self.handle, om) };
      }
    }
  }

  pub(super) extern "C" fn handle_gap_event(
    conn_handle: u16,
    _attr_handle: u16,
    ctxt: *mut esp_idf_sys::ble_gatt_access_ctxt,
    arg: *mut c_void,
  ) -> i32 {
    let ctxt = unsafe { &*ctxt };

    let mutex = unsafe { voidp_to_ref::<Mutex<Self>>(arg) };
    let mut characteristic = mutex.lock();

    if unsafe { ble_uuid_cmp((*ctxt.__bindgen_anon_1.chr).uuid, &characteristic.uuid.u) != 0 } {
      return esp_idf_sys::BLE_ATT_ERR_UNLIKELY as _;
    }

    match ctxt.op as _ {
      esp_idf_sys::BLE_GATT_ACCESS_OP_READ_CHR => {
        let desc = crate::utilities::ble_gap_conn_find(conn_handle).unwrap();

        unsafe {
          if (*(ctxt.om)).om_pkthdr_len > 8 || characteristic.value.len() <= (desc.mtu() - 3) as _ {
            let characteristic = UnsafeCell::new(&mut characteristic);
            if let Some(callback) = &mut (*characteristic.get()).on_read {
              callback(&mut (*characteristic.get()).value, &desc);
            }
          }
        }

        ble_npl_hw_enter_critical();
        let value = characteristic.value.value();
        let rc = os_mbuf_append(ctxt.om, value);
        ble_npl_hw_exit_critical();
        if rc == 0 {
          0
        } else {
          esp_idf_sys::BLE_ATT_ERR_INSUFFICIENT_RES as _
        }
      }
      esp_idf_sys::BLE_GATT_ACCESS_OP_WRITE_CHR => {
        let mut buf = Vec::with_capacity(esp_idf_sys::BLE_ATT_ATTR_MAX_LEN as _);
        let mut om = ctxt.om;
        while !om.is_null() {
          let slice = unsafe { core::slice::from_raw_parts((*om).om_data, (*om).om_len as _) };
          buf.extend_from_slice(slice);
          om = unsafe { (*om).om_next.sle_next };
        }

        let mut notify = false;

        unsafe {
          let characteristic = UnsafeCell::new(&mut characteristic);
          if let Some(callback) = &mut (*characteristic.get()).on_write {
            let desc = crate::utilities::ble_gap_conn_find(conn_handle).unwrap();
            let mut arg = OnWriteArgs {
              current_data: (*characteristic.get()).value.value(),
              recv_data: &buf,
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
        characteristic.set_value(&buf);
        if notify {
          characteristic.notify();
        }

        0
      }
      _ => esp_idf_sys::BLE_ATT_ERR_UNLIKELY as _,
    }
  }

  pub(super) fn subscribe(
    &mut self,
    subscribe: &esp_idf_sys::ble_gap_event__bindgen_ty_1__bindgen_ty_12,
  ) {
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
  /// see [`crate::NimbleSub`] for event type
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
