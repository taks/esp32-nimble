use crate::{BLEConnDesc, BLEError};
use esp_idf_svc::sys as esp_idf_sys;

#[inline]
pub fn ble_gap_conn_find(handle: u16) -> Result<BLEConnDesc, BLEError> {
  let mut desc = esp_idf_sys::ble_gap_conn_desc::default();
  let rc = unsafe { esp_idf_sys::ble_gap_conn_find(handle, &mut desc) };
  BLEError::check_and_return(rc as _, BLEConnDesc(desc))
}
