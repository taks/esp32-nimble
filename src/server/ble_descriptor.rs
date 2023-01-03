use core::{cell::UnsafeCell, ffi::c_void};

use alloc::boxed::Box;
use bitflags::bitflags;
use esp_idf_sys::{ble_uuid_any_t, ble_uuid_cmp, os_mbuf_append};

use crate::{
  utilities::{ble_npl_hw_enter_critical, ble_npl_hw_exit_critical, mutex::Mutex, BleUuid},
  AttValue,
};

bitflags! {
  #[repr(transparent)]
  pub struct DescriptorProperties: u8 {
    const READ = esp_idf_sys::BLE_ATT_F_READ as _;
    const READ_ENC = esp_idf_sys::BLE_ATT_F_READ_ENC as _;
    const READ_AUTHEN = esp_idf_sys::BLE_ATT_F_READ_AUTHEN as _;
    const READ_AUTHOR = esp_idf_sys::BLE_ATT_F_READ_AUTHOR  as _;
    const WRITE = esp_idf_sys::BLE_ATT_F_WRITE  as _;
    const WRITE_ENC = esp_idf_sys::BLE_ATT_F_WRITE_ENC as _;
    const WRITE_AUTHEN = esp_idf_sys::BLE_ATT_F_WRITE_AUTHEN  as _;
    const WRITE_AUTHOR = esp_idf_sys::BLE_ATT_F_WRITE_AUTHOR  as _;
  }
}

#[allow(clippy::type_complexity)]
pub struct BLEDescriptor {
  pub(crate) uuid: ble_uuid_any_t,
  pub(crate) properties: DescriptorProperties,
  value: AttValue,
  on_read: Option<Box<dyn FnMut(&mut AttValue, &esp_idf_sys::ble_gap_conn_desc) + Send + Sync>>,
  on_write: Option<Box<dyn FnMut(&[u8], &esp_idf_sys::ble_gap_conn_desc) + Send + Sync>>,
}

impl BLEDescriptor {
  pub(super) fn new(uuid: BleUuid, properties: DescriptorProperties) -> Self {
    Self {
      uuid: ble_uuid_any_t::from(uuid),
      properties,
      value: AttValue::new(),
      on_read: None,
      on_write: None,
    }
  }

  pub fn set_value(&mut self, value: &[u8]) -> &mut Self {
    self.value.set_value(value);
    self
  }

  pub fn set_from<T: Sized>(&mut self, value: &T) -> &mut Self {
    self.value.set_from(value);
    self
  }

  pub fn value_mut(&mut self) -> &mut AttValue {
    &mut self.value
  }

  pub fn on_read(
    &mut self,
    callback: impl FnMut(&mut AttValue, &esp_idf_sys::ble_gap_conn_desc) + Send + Sync + 'static,
  ) -> &mut Self {
    self.on_read = Some(Box::new(callback));
    self
  }

  pub fn on_write(
    &mut self,
    callback: impl FnMut(&[u8], &esp_idf_sys::ble_gap_conn_desc) + Send + Sync + 'static,
  ) -> &mut Self {
    self.on_write = Some(Box::new(callback));
    self
  }

  pub(super) extern "C" fn handle_gap_event(
    conn_handle: u16,
    _attr_handle: u16,
    ctxt: *mut esp_idf_sys::ble_gatt_access_ctxt,
    arg: *mut c_void,
  ) -> i32 {
    let ctxt = unsafe { &*ctxt };

    let mutex = unsafe { &mut *(arg as *mut Mutex<Self>) };
    let mut descriptor = mutex.lock();

    if unsafe { ble_uuid_cmp((*ctxt.__bindgen_anon_1.chr).uuid, &descriptor.uuid.u) != 0 } {
      return esp_idf_sys::BLE_ATT_ERR_UNLIKELY as _;
    }

    match ctxt.op as _ {
      esp_idf_sys::BLE_GATT_ACCESS_OP_READ_DSC => {
        let desc = crate::utilities::ble_gap_conn_find(conn_handle).unwrap();

        unsafe {
          if (*(ctxt.om)).om_pkthdr_len > 8
            || descriptor.value.len() <= (esp_idf_sys::ble_att_mtu(desc.conn_handle) - 3) as _
          {
            let descriptor = UnsafeCell::new(&mut descriptor);
            if let Some(callback) = &mut (*descriptor.get()).on_read {
              callback(&mut (*descriptor.get()).value, &desc);
            }
          }
        }

        ble_npl_hw_enter_critical();
        let value = descriptor.value.value();
        let rc = unsafe { os_mbuf_append(ctxt.om, value.as_ptr() as _, value.len() as _) };
        ble_npl_hw_exit_critical();
        if rc == 0 {
          0
        } else {
          esp_idf_sys::BLE_ATT_ERR_INSUFFICIENT_RES as _
        }
      }
      esp_idf_sys::BLE_GATT_ACCESS_OP_WRITE_DSC => {
        descriptor.value.clear();
        let mut om = ctxt.om;
        while !om.is_null() {
          let slice = unsafe { core::slice::from_raw_parts((*om).om_data, (*om).om_len as _) };
          descriptor.value.extend(slice);
          om = unsafe { (*om).om_next.sle_next };
        }

        let desc = crate::utilities::ble_gap_conn_find(conn_handle).unwrap();

        unsafe {
          let descriptor = UnsafeCell::new(&mut descriptor);
          if let Some(callback) = &mut (*descriptor.get()).on_write {
            callback((*descriptor.get()).value.value(), &desc);
          }
        }
        0
      }
      _ => esp_idf_sys::BLE_ATT_ERR_UNLIKELY as _,
    }
  }
}
