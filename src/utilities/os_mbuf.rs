use core::ffi::c_int;
use esp_idf_svc::sys;
use sys::os_mbuf;

#[cfg(not(esp_idf_soc_esp_nimble_controller))]
use sys::os_mbuf_append as _os_mbuf_append;

#[cfg(esp_idf_soc_esp_nimble_controller)]
use sys::r_os_mbuf_append as _os_mbuf_append;

/// Append data onto a mbuf
#[inline]
#[allow(unused)]
pub(crate) fn os_mbuf_append(m: *mut os_mbuf, data: &[u8]) -> c_int {
  unsafe { _os_mbuf_append(m, data.as_ptr() as _, data.len() as _) }
}

#[inline]
pub(crate) fn os_mbuf_into_slice<'a>(m: *const os_mbuf) -> &'a [u8] {
  unsafe { core::slice::from_raw_parts((*m).om_data, (*m).om_len as _) }
}

pub(crate) fn ble_hs_mbuf_from_flat(buf: &[u8]) -> *mut os_mbuf {
  unsafe { sys::ble_hs_mbuf_from_flat(buf.as_ptr() as _, buf.len() as _) }
}
