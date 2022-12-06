use bitflags::bitflags;
use esp_idf_sys::c_types::c_void;

use crate::utilities::BleUuid;

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

#[derive(Debug)]
pub struct BLERemoteCharacteristic {
  conn_handle: u16,
  uuid: BleUuid,
  handle: u16,
  properties: GattCharacteristicProperties,
}

impl BLERemoteCharacteristic {
  pub fn new(conn_handle: u16, chr: &esp_idf_sys::ble_gatt_chr) -> Self {
    Self {
      conn_handle,
      uuid: BleUuid::from(chr.uuid),
      handle: chr.val_handle,
      properties: GattCharacteristicProperties::from_bits_truncate(chr.properties),
    }
  }

  pub fn uuid(&self) -> BleUuid {
    self.uuid
  }

  pub fn properties(&self) -> GattCharacteristicProperties {
    self.properties
  }

  pub fn read_value<T>(&mut self) {
    unsafe {
      let _rc = esp_idf_sys::ble_gattc_read_long(
        self.conn_handle,
        self.handle,
        0,
        Some(Self::on_read_cb),
        self as *mut Self as _,
      );
    }

    todo!()
  }
  extern "C" fn on_read_cb(
    conn_handle: u16,
    error: *const esp_idf_sys::ble_gatt_error,
    attr: *mut esp_idf_sys::ble_gatt_attr,
    arg: *mut c_void,
  ) -> i32 {
    let characteristic = unsafe { &mut *(arg as *mut Self) };
    if conn_handle != characteristic.conn_handle {
      return 0;
    }

    let error = unsafe { &*error };

    if error.status == 0 {
      if let Some(_attr) = unsafe { attr.as_ref() } {}
    }

    todo!()
  }
}
