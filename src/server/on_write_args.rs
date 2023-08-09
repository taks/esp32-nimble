pub struct OnWriteArgs<'a> {
  pub recv_data: &'a [u8],
  pub desc: &'a esp_idf_sys::ble_gap_conn_desc,
  pub(crate) reject: bool,
  pub(crate) error_code: u8,
}

impl OnWriteArgs<'_> {
  pub fn reject(&mut self) {
    self.reject_with_error_code(0xFF);
  }

  pub fn reject_with_error_code(&mut self, error_code: u8) {
    self.reject = true;
    self.error_code = error_code;
  }
}
