use crate::BLEConnDesc;

pub struct OnWriteArgs<'a> {
  pub(crate) current_data: &'a [u8],
  pub(crate) recv_data: &'a [u8],
  pub(crate) desc: &'a BLEConnDesc,
  pub(crate) reject: bool,
  pub(crate) error_code: u8,
  pub(crate) notify: bool,
}

impl OnWriteArgs<'_> {
  pub fn current_data(&self) -> &[u8] {
    self.current_data
  }

  pub fn recv_data(&self) -> &[u8] {
    self.recv_data
  }

  pub fn desc(&self) -> &BLEConnDesc {
    self.desc
  }

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

  pub fn notify(&mut self) {
    self.notify = true;
  }
}

pub struct OnWriteDescriptorArgs<'a> {
  pub(crate) current_data: &'a [u8],
  pub(crate) recv_data: &'a [u8],
  pub(crate) desc: &'a BLEConnDesc,
  pub(crate) reject: bool,
  pub(crate) error_code: u8,
}

impl OnWriteDescriptorArgs<'_> {
  pub fn current_data(&self) -> &[u8] {
    self.current_data
  }

  pub fn recv_data(&self) -> &[u8] {
    self.recv_data
  }

  pub fn desc(&self) -> &BLEConnDesc {
    self.desc
  }

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
