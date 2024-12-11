use alloc::vec::Vec;

pub struct AttValue {
  value: Vec<u8>,
}

impl AttValue {
  pub(super) const fn new() -> Self {
    Self { value: Vec::new() }
  }

  #[inline]
  pub fn as_slice(&self) -> &[u8] {
    &self.value
  }

  #[inline]
  pub fn as_ref<T: Sized>(&self) -> Option<&T> {
    if self.len() == core::mem::size_of::<T>() {
      unsafe { Some(&*(self.value.as_ptr() as *const T)) }
    } else {
      None
    }
  }

  #[inline]
  pub fn as_mut<T: Sized>(&mut self) -> Option<&mut T> {
    if self.len() == core::mem::size_of::<T>() {
      unsafe { Some(&mut *(self.value.as_mut_ptr() as *mut T)) }
    } else {
      None
    }
  }

  #[inline]
  pub fn is_empty(&self) -> bool {
    self.value.is_empty()
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.value.len()
  }

  #[inline]
  pub fn clear(&mut self) {
    self.value.clear();
  }

  #[inline]
  pub fn set_value(&mut self, value: &[u8]) {
    self.value.clear();
    self.value.extend_from_slice(value);
  }

  #[deprecated(note = "Please use `set_value` + zerocopy::IntoBytes")]
  #[inline]
  pub fn set_from<T: Sized>(&mut self, p: &T) {
    let slice = unsafe {
      ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
    };
    self.set_value(slice);
  }

  #[inline]
  pub fn extend(&mut self, value: &[u8]) {
    self.value.extend_from_slice(value);
  }
}
