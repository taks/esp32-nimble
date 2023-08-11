pub struct OnWriteArgs<'a> {
  pub recv_data: &'a [u8],
  pub desc: &'a esp_idf_sys::ble_gap_conn_desc,
  pub(crate) reject: bool,
  pub(crate) error_code: u8,
}

impl OnWriteArgs<'_> {
  /// If the reject is called, no value is written to BLECharacteristic or BLEDescriptor.
  /// A write error (0xFF) is sent to the sender.
  pub fn reject(&mut self) {
    self.reject_with_error_code(0xFF);
  }

  /// If the reject is called, no value is written to BLECharacteristic or BLEDescriptor.
  /// The argument error code is sent to the sender.
  pub fn reject_with_error_code(&mut self, error_code: u8) {
    self.reject = true;
    self.error_code = error_code;
  }
}
