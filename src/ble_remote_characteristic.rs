use crate::{
  ble,
  ble_remote_service::BLERemoteServiceState,
  utilities::{BLEReader, BLEWriter, BleUuid, UnsafeArc, WeakUnsafeCell},
  BLERemoteDescriptor, BLEReturnCode, Signal,
};
use alloc::{boxed::Box, vec::Vec};
use bitflags::bitflags;
use core::ffi::c_void;

bitflags! {
  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
  pub struct GattCharacteristicProperties: u8 {
    const Broadcast = esp_idf_sys::BLE_GATT_CHR_PROP_BROADCAST as _;
    const Read = esp_idf_sys::BLE_GATT_CHR_PROP_READ as _;
    const WriteWithoutResponse = esp_idf_sys::BLE_GATT_CHR_PROP_WRITE_NO_RSP as _;
    const Write = esp_idf_sys::BLE_GATT_CHR_PROP_WRITE as _;
    const Notify = esp_idf_sys::BLE_GATT_CHR_PROP_NOTIFY as _;
    const Indicate = esp_idf_sys::BLE_GATT_CHR_PROP_INDICATE as _;
    const AuthenticatedSignedWrites = esp_idf_sys::BLE_GATT_CHR_PROP_AUTH_SIGN_WRITE as _;
    const ExtendedProperties =  esp_idf_sys::BLE_GATT_CHR_PROP_EXTENDED as _;
  }
}

#[allow(clippy::type_complexity)]
pub(crate) struct BLERemoteCharacteristicState {
  service: WeakUnsafeCell<BLERemoteServiceState>,
  uuid: BleUuid,
  pub handle: u16,
  end_handle: u16,
  properties: GattCharacteristicProperties,
  descriptors: Option<Vec<UnsafeArc<BLERemoteDescriptor>>>,
  signal: Signal<u32>,
  on_notify: Option<Box<dyn FnMut(&[u8]) + Send + Sync>>,
}

impl BLERemoteCharacteristicState {
  pub fn conn_handle(&self) -> u16 {
    match self.service.upgrade() {
      Some(x) => x.conn_handle(),
      None => esp_idf_sys::BLE_HS_CONN_HANDLE_NONE as _,
    }
  }
}

pub struct BLERemoteCharacteristic {
  state: UnsafeArc<BLERemoteCharacteristicState>,
}

impl BLERemoteCharacteristic {
  pub(crate) fn new(
    service: WeakUnsafeCell<BLERemoteServiceState>,
    chr: &esp_idf_sys::ble_gatt_chr,
  ) -> Self {
    Self {
      state: UnsafeArc::new(BLERemoteCharacteristicState {
        service,
        uuid: BleUuid::from(chr.uuid),
        handle: chr.val_handle,
        end_handle: 0,
        properties: GattCharacteristicProperties::from_bits_truncate(chr.properties),
        descriptors: None,
        signal: Signal::new(),
        on_notify: None,
      }),
    }
  }

  pub(crate) fn state(&self) -> &BLERemoteCharacteristicState {
    &self.state
  }

  pub fn uuid(&self) -> BleUuid {
    self.state.uuid
  }

  pub fn properties(&self) -> GattCharacteristicProperties {
    self.state.properties
  }

  pub async fn get_descriptors(
    &mut self,
  ) -> Result<core::slice::IterMut<'_, UnsafeArc<BLERemoteDescriptor>>, BLEReturnCode> {
    if self.state.descriptors.is_none() {
      self.state.descriptors = Some(Vec::new());

      if self.state.end_handle == 0 {
        unsafe {
          ble!(esp_idf_sys::ble_gattc_disc_all_chrs(
            self.state.conn_handle(),
            self.state.handle,
            self.state.service.upgrade().unwrap().end_handle,
            Some(Self::next_char_cb),
            self as *mut Self as _,
          ))?;
        }

        ble!(self.state.signal.wait().await)?;
      }

      unsafe {
        ble!(esp_idf_sys::ble_gattc_disc_all_dscs(
          self.state.conn_handle(),
          self.state.handle,
          self.state.end_handle,
          Some(Self::descriptor_disc_cb),
          self as *mut Self as _,
        ))?;
      }
      ble!(self.state.signal.wait().await)?;
    }

    Ok(self.state.descriptors.as_mut().unwrap().iter_mut())
  }

  pub async fn get_descriptor(
    &mut self,
    uuid: BleUuid,
  ) -> Result<&mut UnsafeArc<BLERemoteDescriptor>, BLEReturnCode> {
    let mut iter = self.get_descriptors().await?;
    iter
      .find(|x| x.uuid() == uuid)
      .ok_or_else(|| BLEReturnCode::fail().unwrap_err())
  }

  extern "C" fn next_char_cb(
    conn_handle: u16,
    error: *const esp_idf_sys::ble_gatt_error,
    chr: *const esp_idf_sys::ble_gatt_chr,
    arg: *mut c_void,
  ) -> i32 {
    let characteristic = unsafe { &mut *(arg as *mut Self) };
    if characteristic.state.conn_handle() != conn_handle {
      return 0;
    }

    let error = unsafe { &*error };
    if error.status == 0 {
      characteristic.state.end_handle = unsafe { (*chr).def_handle - 1 };
    } else if error.status == esp_idf_sys::BLE_HS_EDONE as _ {
      characteristic.state.end_handle = characteristic.state.service.upgrade().unwrap().end_handle;
    }

    characteristic.state.signal.signal(error.status as _);
    error.status as _
  }

  extern "C" fn descriptor_disc_cb(
    conn_handle: u16,
    error: *const esp_idf_sys::ble_gatt_error,
    _chr_val_handle: u16,
    dsc: *const esp_idf_sys::ble_gatt_dsc,
    arg: *mut c_void,
  ) -> i32 {
    let characteristic = unsafe { &mut *(arg as *mut Self) };
    if characteristic.state.conn_handle() != conn_handle {
      return 0;
    }

    let error = unsafe { &*error };
    let dsc = unsafe { &*dsc };

    if error.status == 0 {
      let descriptor = UnsafeArc::new(BLERemoteDescriptor::new(
        UnsafeArc::downgrade(&characteristic.state),
        dsc,
      ));
      characteristic
        .state
        .descriptors
        .as_mut()
        .unwrap()
        .push(descriptor);
      return 0;
    }

    characteristic.state.signal.signal(error.status as _);
    error.status as _
  }

  pub async fn read_value(&mut self) -> Result<Vec<u8>, BLEReturnCode> {
    let mut reader = BLEReader::new(self.state.conn_handle(), self.state.handle);
    reader.read_value().await
  }

  pub async fn write_value(&mut self, data: &[u8], response: bool) -> Result<(), BLEReturnCode> {
    let mut writer = BLEWriter::new(self.state.conn_handle(), self.state.handle);
    writer.write_value(data, response).await
  }

  pub async fn subscribe(
    &mut self,
    notifications: bool,
    response: bool,
  ) -> Result<(), BLEReturnCode> {
    if notifications {
      self.set_notify(0x01, response).await
    } else {
      self.set_notify(0x02, response).await
    }
  }

  pub async fn unsubscribe(&mut self, response: bool) -> Result<(), BLEReturnCode> {
    self.set_notify(0x00, response).await
  }

  async fn set_notify(&mut self, val: u16, response: bool) -> Result<(), BLEReturnCode> {
    let desc = self.get_descriptor(BleUuid::from_uuid16(0x2902)).await?;
    desc.write_value(&val.to_ne_bytes(), response).await
  }

  pub fn on_notify(&mut self, callback: impl FnMut(&[u8]) + Send + Sync + 'static) -> &mut Self {
    self.state.on_notify = Some(Box::new(callback));
    self
  }

  pub(crate) unsafe fn notify(&mut self, om: *mut esp_idf_sys::os_mbuf) {
    if let Some(no_notify) = self.state.on_notify.as_mut() {
      let data = unsafe { core::slice::from_raw_parts((*om).om_data, (*om).om_len as _) };
      no_notify(data);
    }
  }
}
