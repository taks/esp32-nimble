use crate::BLEClient;
use esp_idf_svc::sys as esp_idf_sys;

pub(crate) trait BLEAttribute {
  fn get_client(&self) -> Option<BLEClient>;

  fn conn_handle(&self) -> u16 {
    match self.get_client() {
      Some(x) => x.conn_handle(),
      None => esp_idf_sys::BLE_HS_CONN_HANDLE_NONE as _,
    }
  }
}
