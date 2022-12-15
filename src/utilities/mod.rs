mod ble_uuid;
pub use ble_uuid::BleUuid;

pub mod mutex;

mod arc_unsafe_cell;
pub(crate) use arc_unsafe_cell::*;

mod nimble_npl_os;
pub(crate) use nimble_npl_os::*;

#[inline]
#[allow(unused)]
pub(crate) unsafe fn extend_lifetime_mut<'a, 'b: 'a, T: ?Sized>(r: &'a mut T) -> &'b mut T {
  core::mem::transmute::<&'a mut T, &'b mut T>(r)
}

#[inline]
#[allow(unused)]
pub(crate) unsafe fn as_mut_ptr<T>(ptr: *const T) -> *mut T {
  ptr as *mut T
}
