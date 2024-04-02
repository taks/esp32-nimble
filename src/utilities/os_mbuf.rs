use core::ffi::c_int;
use esp_idf_sys::{os_mbuf, os_mbuf_pool};

pub(crate) fn os_mbuf_get_pkthdr(omp: *mut os_mbuf_pool, pkthdr_len: u8) -> *mut os_mbuf {
  #[cfg(not(esp_idf_soc_esp_nimble_controller))]
  unsafe {
    esp_idf_sys::os_mbuf_get_pkthdr(omp, pkthdr_len)
  }

  #[cfg(esp_idf_soc_esp_nimble_controller)]
  unsafe {
    esp_idf_sys::r_os_mbuf_get_pkthdr(omp, pkthdr_len)
  }
}

/// Allocate a packet header structure from the MSYS pool. See os_msys_register() for a description of MSYS.
#[inline]
#[allow(unused)]
pub(crate) fn os_msys_get_pkthdr(dsize: u16, user_hdr_len: u16) -> *mut os_mbuf {
  #[cfg(not(esp_idf_soc_esp_nimble_controller))]
  unsafe {
    esp_idf_sys::os_msys_get_pkthdr(dsize, user_hdr_len)
  }

  #[cfg(esp_idf_soc_esp_nimble_controller)]
  unsafe {
    esp_idf_sys::r_os_msys_get_pkthdr(dsize, user_hdr_len)
  }
}

/// Append data onto a mbuf
#[inline]
#[allow(unused)]
pub(crate) fn os_mbuf_append(m: *mut os_mbuf, data: &[u8]) -> c_int {
  #[cfg(not(esp_idf_soc_esp_nimble_controller))]
  unsafe {
    esp_idf_sys::os_mbuf_append(m, data.as_ptr() as _, data.len() as _)
  }

  #[cfg(esp_idf_soc_esp_nimble_controller)]
  unsafe {
    esp_idf_sys::r_os_mbuf_append(m, data.as_ptr() as _, data.len() as _)
  }
}

#[inline]
pub(crate) fn os_mbuf_into_slice<'a>(m: *const os_mbuf) -> &'a [u8] {
  unsafe { core::slice::from_raw_parts((*m).om_data, (*m).om_len as _) }
}
