use crate::utilities::os_mbuf_into_slice;

pub struct ReceivedData(esp_idf_sys::ble_l2cap_event__bindgen_ty_1__bindgen_ty_4);

impl ReceivedData {
  #[inline]
  pub(crate) fn from_raw(raw: esp_idf_sys::ble_l2cap_event__bindgen_ty_1__bindgen_ty_4) -> Self {
    Self(raw)
  }

  #[inline]
  pub fn conn_handle(&self) -> u16 {
    self.0.conn_handle
  }

  #[inline]
  pub fn data(&self) -> &[u8] {
    os_mbuf_into_slice(self.0.sdu_rx)
  }
}

impl Drop for ReceivedData {
  fn drop(&mut self) {
    unsafe { super::os_mbuf_free(self.0.sdu_rx) };
  }
}
