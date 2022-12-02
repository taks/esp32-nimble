pub(crate) mod leaky_box;

mod ble_uuid;
pub use ble_uuid::BleUuid;

pub mod mutex;

#[inline]
#[allow(unused)]
pub(crate) unsafe fn extend_lifetime_mut<'a, 'b: 'a, T: ?Sized>(r: &'a mut T) -> &'b mut T {
  core::mem::transmute::<&'a mut T, &'b mut T>(r)
}
