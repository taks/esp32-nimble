use core::ffi::c_int;
use esp_idf_svc::sys;
use sys::os_mbuf;

#[cfg(not(esp_idf_soc_esp_nimble_controller))]
use sys::os_mbuf_append as _os_mbuf_append;

#[cfg(esp_idf_soc_esp_nimble_controller)]
use sys::r_os_mbuf_append as _os_mbuf_append;

#[derive(Copy, Clone)]
pub(crate) struct OsMBuf(pub *mut os_mbuf);

#[allow(unused)]
impl OsMBuf {
  #[inline]
  pub fn as_slice<'a>(&self) -> &'a [u8] {
    unsafe { core::slice::from_raw_parts((*self.0).om_data, (*self.0).om_len as _) }
  }

  #[inline]
  pub fn entire_len(&self) -> u16 {
    unsafe { (*self.0.add(1).cast::<sys::os_mbuf_pkthdr>()).omp_len }
  }

  /// Append data onto a mbuf
  #[inline]
  pub(crate) fn append(&mut self, data: &[u8]) -> c_int {
    unsafe { _os_mbuf_append(self.0, data.as_ptr() as _, data.len() as _) }
  }

  #[inline]
  pub fn from_flat(buf: &[u8]) -> Self {
    OsMBuf(unsafe { sys::ble_hs_mbuf_from_flat(buf.as_ptr() as _, buf.len() as _) })
  }

  #[inline]
  pub fn iter(&self) -> OsMBufIterator {
    OsMBufIterator(*self)
  }
}

pub(crate) struct OsMBufIterator(OsMBuf);

impl Iterator for OsMBufIterator {
  type Item = OsMBuf;

  fn next(&mut self) -> Option<Self::Item> {
    if (self.0).0.is_null() {
      None
    } else {
      let current = self.0;

      self.0 = OsMBuf(unsafe { (*self.0 .0).om_next.sle_next });

      Some(current)
    }
  }
}
