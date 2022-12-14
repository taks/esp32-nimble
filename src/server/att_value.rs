use alloc::vec::Vec;

pub struct AttValue {
  value: Vec<u8>,
}

impl AttValue {
  pub(super) fn new() -> Self {
    Self { value: Vec::new() }
  }

  pub fn value(&self) -> &[u8] {
    &self.value
  }

  pub fn is_empty(&self) -> bool {
    self.value.is_empty()
  }

  pub fn len(&self) -> usize {
    self.value.len()
  }

  pub fn clear(&mut self) {
    self.value.clear();
  }

  pub fn set_value(&mut self, value: &[u8]) {
    self.value.clear();
    self.value.extend_from_slice(value);
  }

  pub fn extend(&mut self, value: &[u8]) {
    self.value.extend_from_slice(value);
  }
}
