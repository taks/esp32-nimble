use crate::utilities::BleUuid;

#[derive(Copy, Clone, Debug)]
pub struct BLERemoteService {
  pub uuid: BleUuid,
  pub start_handle: u16,
  pub end_handle: u16,
}

impl BLERemoteService {
  pub fn new(service: &esp_idf_sys::ble_gatt_svc) -> Self {
    Self {
      uuid: BleUuid::from(service.uuid),
      start_handle: service.start_handle,
      end_handle: service.end_handle,
    }
  }
}
