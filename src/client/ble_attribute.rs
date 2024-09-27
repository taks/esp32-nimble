use crate::utilities::ArcUnsafeCell;
use esp_idf_svc::sys as esp_idf_sys;

use super::ble_client::BLEClientState;

pub(crate) trait BLEAttribute {
  fn get_client(&self) -> Option<ArcUnsafeCell<BLEClientState>>;

  fn conn_handle(&self) -> u16 {
    match self.get_client() {
      Some(x) => x.conn_handle,
      None => esp_idf_sys::BLE_HS_CONN_HANDLE_NONE as _,
    }
  }
}
