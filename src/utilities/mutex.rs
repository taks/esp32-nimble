// original: https://github.com/esp-rs/esp-idf-svc/blob/master/src/private/mutex.rs

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use esp_idf_svc::sys::*;

// NOTE: ESP-IDF-specific
const PTHREAD_MUTEX_INITIALIZER: u32 = 0xFFFFFFFF;

pub struct RawMutex(UnsafeCell<pthread_mutex_t>);

impl RawMutex {
  #[inline(always)]
  pub const fn new() -> Self {
    Self(UnsafeCell::new(PTHREAD_MUTEX_INITIALIZER as _))
  }

  #[inline(always)]
  #[allow(clippy::missing_safety_doc)]
  pub unsafe fn lock(&self) {
    let r = pthread_mutex_lock(self.0.get());
    debug_assert_eq!(r, 0);
  }

  #[inline(always)]
  #[allow(clippy::missing_safety_doc)]
  pub unsafe fn unlock(&self) {
    let r = pthread_mutex_unlock(self.0.get());
    debug_assert_eq!(r, 0);
  }
}

impl Drop for RawMutex {
  fn drop(&mut self) {
    let r = unsafe { pthread_mutex_destroy(self.0.get_mut() as *mut _) };
    debug_assert_eq!(r, 0);
  }
}

unsafe impl Sync for RawMutex {}
unsafe impl Send for RawMutex {}

pub struct Mutex<T>(RawMutex, UnsafeCell<T>);

impl<T> Mutex<T> {
  #[inline(always)]
  pub const fn new(data: T) -> Self {
    Self(RawMutex::new(), UnsafeCell::new(data))
  }

  #[inline(always)]
  pub fn lock(&self) -> MutexGuard<'_, T> {
    MutexGuard::new(self)
  }

  #[inline(always)]
  pub(crate) fn into_innter(self) -> T {
    self.1.into_inner()
  }

  #[inline]
  pub(crate) unsafe fn raw(&self) -> &'_ T {
    self.1.get().as_mut().unwrap()
  }
}

unsafe impl<T> Sync for Mutex<T> where T: Send {}
unsafe impl<T> Send for Mutex<T> where T: Send {}

pub struct MutexGuard<'a, T>(&'a Mutex<T>);

impl<'a, T> MutexGuard<'a, T> {
  #[inline(always)]
  fn new(mutex: &'a Mutex<T>) -> Self {
    unsafe {
      mutex.0.lock();
    }

    Self(mutex)
  }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
  #[inline(always)]
  fn drop(&mut self) {
    unsafe {
      self.0 .0.unlock();
    }
  }
}

impl<'a, T> Deref for MutexGuard<'a, T> {
  type Target = T;

  #[inline(always)]
  fn deref(&self) -> &Self::Target {
    unsafe { self.0 .1.get().as_mut().unwrap() }
  }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
  #[inline(always)]
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { self.0 .1.get().as_mut().unwrap() }
  }
}
