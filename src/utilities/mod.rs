mod ble_uuid;
pub use ble_uuid::BleUuid;

mod ble_reader;
pub(crate) use ble_reader::BLEReader;

mod ble_writer;
pub(crate) use ble_writer::BLEWriter;

pub mod mutex;

mod unsafe_arc;
pub use unsafe_arc::*;

#[inline]
#[allow(unused)]
pub(crate) unsafe fn extend_lifetime_mut<'a, 'b: 'a, T: ?Sized>(r: &'a mut T) -> &'b mut T {
  core::mem::transmute::<&'a mut T, &'b mut T>(r)
}
