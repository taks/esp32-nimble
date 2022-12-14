use crate::BLEReturnCode;

#[inline]
#[allow(unused)]
pub(super) fn ble_gap_conn_find(
  handle: u16,
) -> Result<esp_idf_sys::ble_gap_conn_desc, BLEReturnCode> {
  let mut desc = esp_idf_sys::ble_gap_conn_desc::default();
  let rc = unsafe { esp_idf_sys::ble_gap_conn_find(handle, &mut desc) };
  BLEReturnCode::check_and_return(rc as _, desc)
}

#[inline]
#[allow(unused)]
pub(super) unsafe fn extend_lifetime_mut<'a, 'b: 'a, T: ?Sized>(r: &'a mut T) -> &'b mut T {
  core::mem::transmute::<&'a mut T, &'b mut T>(r)
}
