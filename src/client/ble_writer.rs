use crate::{
  ble,
  utilities::{as_void_ptr, voidp_to_ref, OsMBuf},
  BLEError, Signal,
};
use esp_idf_svc::sys as esp_idf_sys;

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

  pub async fn write_value(&mut self, data: &[u8], response: bool) -> Result<(), BLEError> {
    unsafe {
      // ble_att_mtu() returns 0 for a closed connection
      let mtu = esp_idf_sys::ble_att_mtu(self.conn_handle);
      if mtu == 0 {
        return BLEError::convert(esp_idf_sys::BLE_HS_ENOTCONN as _);
      }
      let mtu = { mtu - 3 } as usize;

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
          as_void_ptr(self),
        ))?;
      } else {
        let om = OsMBuf::from_flat(data);
        ble!(esp_idf_sys::ble_gattc_write_long(
          self.conn_handle,
          self.handle,
          0,
          om.0,
          Some(Self::on_write_cb),
          as_void_ptr(self),
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
