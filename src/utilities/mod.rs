mod ble_uuid;
pub use ble_uuid::BleUuid;

pub mod mutex;

mod arc_unsafe_cell;
pub(crate) use arc_unsafe_cell::*;

mod ble_functions;
pub(crate) use ble_functions::*;

mod nimble_npl_os;
pub(crate) use nimble_npl_os::*;

mod os_mbuf;
pub(crate) use os_mbuf::*;

#[inline]
#[allow(unused)]
pub(crate) unsafe fn extend_lifetime_mut<'a, 'b: 'a, T: ?Sized>(r: &'a mut T) -> &'b mut T {
  unsafe { core::mem::transmute::<&'a mut T, &'b mut T>(r) }
}

#[inline]
#[allow(unused)]
pub(crate) const unsafe fn as_mut_ptr<T>(ptr: *const T) -> *mut T {
  ptr as *mut T
}

#[inline]
pub(crate) unsafe fn as_void_ptr<T>(r: &mut T) -> *mut ::core::ffi::c_void {
  (r as *mut T).cast()
}

#[inline]
pub(crate) unsafe fn voidp_to_ref<'a, T>(ptr: *mut core::ffi::c_void) -> &'a mut T {
  unsafe { &mut *ptr.cast() }
}
