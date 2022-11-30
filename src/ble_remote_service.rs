use crate::utilities::BleUuid;

#[derive(Copy, Clone, Debug)]
pub struct BLERemoteService {
  pub uuid: BleUuid,
  pub start_handle: u16,
  pub end_handle: u16,
}

impl BLERemoteService {
  pub fn new(uuid: BleUuid, start_handle: u16, end_handle: u16) -> Self {
    Self {
      uuid,
      start_handle,
      end_handle,
    }
  }
}
