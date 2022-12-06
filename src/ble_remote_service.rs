use crate::{utilities::BleUuid, BLERemoteCharacteristic, Signal};
use alloc::vec::Vec;
use esp_idf_sys::{c_types::c_void, *};

pub struct BLERemoteService {
  conn_handle: u16,
  uuid: BleUuid,
  start_handle: u16,
  end_handle: u16,
  characteristics: Option<Vec<BLERemoteCharacteristic>>,
  signal: Signal<u32>,
}

impl BLERemoteService {
  pub fn new(conn_handle: u16, service: &esp_idf_sys::ble_gatt_svc) -> Self {
    Self {
      conn_handle,
      uuid: BleUuid::from(service.uuid),
      start_handle: service.start_handle,
      end_handle: service.end_handle,
      characteristics: None,
      signal: Signal::new(),
    }
  }

  pub fn uuid(&self) -> BleUuid {
    self.uuid
  }

  pub async fn get_characteristics(
    &mut self,
  ) -> Result<core::slice::IterMut<'_, BLERemoteCharacteristic>, EspError> {
    if self.characteristics.is_none() {
      self.characteristics = Some(Vec::new());
      unsafe {
        esp_idf_sys::ble_gattc_disc_all_chrs(
          self.conn_handle,
          self.start_handle,
          self.end_handle,
          Some(Self::characteristic_disc_cb),
          self as *mut Self as _,
        );
      }
      esp!(self.signal.wait().await)?;
    }

    Ok(self.characteristics.as_mut().unwrap().iter_mut())
  }

  pub async fn get_characteristic(
    &mut self,
    uuid: BleUuid,
  ) -> Result<&mut BLERemoteCharacteristic, EspError> {
    let mut iter = self.get_characteristics().await?;
    iter
      .find(|x| x.uuid() == uuid)
      .ok_or(EspError::from(ESP_FAIL).unwrap())
  }

  extern "C" fn characteristic_disc_cb(
    conn_handle: u16,
    error: *const ble_gatt_error,
    chr: *const ble_gatt_chr,
    arg: *mut c_void,
  ) -> i32 {
    let service = unsafe { &mut *(arg as *mut Self) };
    if service.conn_handle != conn_handle {
      return 0;
    }
    let error = unsafe { &*error };
    let chr = unsafe { &*chr };

    if error.status == 0 {
      let chr = BLERemoteCharacteristic::new(conn_handle, chr);
      service.characteristics.as_mut().unwrap().push(chr);
      return 0;
    }

    let ret = if error.status == (BLE_HS_EDONE as _) {
      0
    } else {
      error.status as _
    };

    service.signal.signal(ret);
    ret as _
  }
}

impl core::fmt::Debug for BLERemoteService {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "BLERemoteService[{}]", self.uuid)?;
    Ok(())
  }
}
