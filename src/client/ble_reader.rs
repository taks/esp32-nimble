use alloc::vec::Vec;
use core::ffi::c_void;
use esp_idf_svc::sys as esp_idf_sys;

use crate::{ble, utilities::voidp_to_ref, BLEError, Signal};

pub struct BLEReader {
  conn_handle: u16,
  handle: u16,
  signal: Signal<u32>,
}

impl BLEReader {
  pub fn new(conn_handle: u16, handle: u16) -> Self {
    Self {
      conn_handle,
      handle,
      signal: Signal::new(),
    }
  }

  pub async fn read_value(&mut self) -> Result<Vec<u8>, BLEError> {
    let data = Vec::<u8>::new();
    let mut arg = (self, data);

    unsafe {
      ble!(esp_idf_sys::ble_gattc_read_long(
        arg.0.conn_handle,
        arg.0.handle,
        0,
        Some(Self::on_read_cb),
        core::ptr::addr_of_mut!(arg) as _,
      ))?;
    }

    ble!(arg.0.signal.wait().await)?;
    Ok(arg.1)
  }

  extern "C" fn on_read_cb(
    conn_handle: u16,
    error: *const esp_idf_sys::ble_gatt_error,
    attr: *mut esp_idf_sys::ble_gatt_attr,
    arg: *mut c_void,
  ) -> i32 {
    let (reader, data) = unsafe { voidp_to_ref::<(&mut Self, Vec<u8>)>(arg) };
    if conn_handle != reader.conn_handle {
      return 0;
    }

    let error = unsafe { &*error };

    if error.status == 0 {
      if let Some(attr) = unsafe { attr.as_ref() } {
        let mut om = attr.om;
        while !om.is_null() {
          let slice = unsafe { core::slice::from_raw_parts((*om).om_data, (*om).om_len as _) };
          data.extend_from_slice(slice);
          om = unsafe { (*om).om_next.sle_next };
        }

        return 0;
      }
    }

    reader.signal.signal(error.status as _);
    error.status as _
  }
}
