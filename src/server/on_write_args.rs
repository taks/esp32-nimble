pub struct OnWriteArgs<'a> {
  pub recv_data: &'a [u8],
  pub desc: &'a esp_idf_sys::ble_gap_conn_desc,
  pub reject: bool,
}
