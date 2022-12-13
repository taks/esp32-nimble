use alloc::sync::{Arc, Weak};
use core::{
  cell::UnsafeCell,
  ops::{Deref, DerefMut},
};

pub struct ArcUnsafeCell<T: ?Sized> {
  value: Arc<UnsafeCell<T>>,
}

impl<T> ArcUnsafeCell<T> {
  #[inline(always)]
  pub(crate) fn new(value: T) -> ArcUnsafeCell<T> {
    Self {
      value: Arc::new(UnsafeCell::new(value)),
    }
  }

  pub fn downgrade(this: &ArcUnsafeCell<T>) -> WeakUnsafeCell<T> {
    WeakUnsafeCell {
      value: Arc::downgrade(&this.value),
    }
  }
}

impl<T: ?Sized> Deref for ArcUnsafeCell<T> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &T {
    unsafe { &*self.value.get() }
  }
}

impl<T: ?Sized> DerefMut for ArcUnsafeCell<T> {
  #[inline]
  fn deref_mut(&mut self) -> &mut T {
    unsafe { &mut *self.value.get() }
  }
}

pub struct WeakUnsafeCell<T: ?Sized> {
  pub value: Weak<UnsafeCell<T>>,
}

impl<T> WeakUnsafeCell<T> {
  pub fn upgrade(&self) -> Option<ArcUnsafeCell<T>> {
    self.value.upgrade().map(|x| ArcUnsafeCell { value: x })
  }
}
