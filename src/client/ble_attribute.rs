use crate::BLEClient;

pub(crate) trait BLEAttribute {
  fn get_client(&self) -> Option<BLEClient>;

  fn conn_handle(&self) -> u16 {
    match self.get_client() {
      Some(x) => x.conn_handle(),
      None => esp_idf_sys::BLE_HS_CONN_HANDLE_NONE as _,
    }
  }
}
