use core::ffi::c_int;
use esp_idf_sys::os_mbuf;

/// Allocate a packet header structure from the MSYS pool. See os_msys_register() for a description of MSYS.
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
