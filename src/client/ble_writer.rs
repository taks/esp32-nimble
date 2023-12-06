use crate::{ble, BLEReturnCode, Signal, utilities::voidp_to_ref};

pub struct BLEWriter {
  conn_handle: u16,
  handle: u16,
  signal: Signal<u32>,
}

impl BLEWriter {
  pub fn new(conn_handle: u16, handle: u16) -> Self {
    Self {
      conn_handle,
      handle,
      signal: Signal::new(),
    }
  }

  pub async fn write_value(&mut self, data: &[u8], response: bool) -> Result<(), BLEReturnCode> {
    unsafe {
      let mtu = { esp_idf_sys::ble_att_mtu(self.conn_handle) - 3 } as usize;

      if !response && data.len() <= mtu {
        return ble!(esp_idf_sys::ble_gattc_write_no_rsp_flat(
          self.conn_handle,
          self.handle,
          data.as_ptr() as _,
          data.len() as _
        ));
      }

      if data.len() <= mtu {
        ble!(esp_idf_sys::ble_gattc_write_flat(
          self.conn_handle,
          self.handle,
          data.as_ptr() as _,
          data.len() as _,
          Some(Self::on_write_cb),
          self as *mut Self as _,
        ))?;
      } else {
        let om = esp_idf_sys::ble_hs_mbuf_from_flat(data.as_ptr() as _, data.len() as _);
        ble!(esp_idf_sys::ble_gattc_write_long(
          self.conn_handle,
          self.handle,
          0,
          om,
          Some(Self::on_write_cb),
          self as *mut Self as _
        ))?;
      }

      ble!(self.signal.wait().await)?;
    }

    Ok(())
  }

  extern "C" fn on_write_cb(
    conn_handle: u16,
    error: *const esp_idf_sys::ble_gatt_error,
    _service: *mut esp_idf_sys::ble_gatt_attr,
    arg: *mut core::ffi::c_void,
  ) -> i32 {
    let writer = unsafe { voidp_to_ref::<Self>(arg) };
    if writer.conn_handle != conn_handle {
      return 0;
    }

    writer.signal.signal(unsafe { (*error).status as _ });
    0
  }
}
