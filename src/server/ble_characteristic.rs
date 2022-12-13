use core::{cell::UnsafeCell, ffi::c_void};

use alloc::{boxed::Box, vec::Vec};
use bitflags::bitflags;
use esp_idf_sys::{ble_uuid_any_t, ble_uuid_cmp, os_mbuf_append};

use crate::{
  utilities::{ble_npl_hw_enter_critical, ble_npl_hw_exit_critical, mutex::Mutex, BleUuid},
  BLEDevice,
};

use super::att_value::AttValue;

const NULL_HANDLE: u16 = 0xFFFF;

bitflags! {
  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
  pub struct NimbleProperties: u16 {
    const Read = esp_idf_sys::BLE_GATT_CHR_F_READ as _;
    const ReadEnc = esp_idf_sys::BLE_GATT_CHR_F_READ_ENC as _;
    const ReadAuthen = esp_idf_sys::BLE_GATT_CHR_F_READ_AUTHEN as _;
    const ReadAuthor = esp_idf_sys::BLE_GATT_CHR_F_READ_AUTHOR as _;
    const Write = esp_idf_sys::BLE_GATT_CHR_F_WRITE as _;
    const WriteNoRsp = esp_idf_sys::BLE_GATT_CHR_F_WRITE_NO_RSP as _;
    const WriteEnc = esp_idf_sys::BLE_GATT_CHR_F_WRITE_ENC as _;
    const WriteAuthen = esp_idf_sys::BLE_GATT_CHR_F_WRITE_AUTHEN as _;
    const WriteAuthor = esp_idf_sys::BLE_GATT_CHR_F_WRITE_AUTHOR as _;
    const Broadcast = esp_idf_sys::BLE_GATT_CHR_F_BROADCAST as _;
    const Notify = esp_idf_sys::BLE_GATT_CHR_F_NOTIFY as _;
    const Indicate = esp_idf_sys::BLE_GATT_CHR_F_INDICATE as _;
  }
}

bitflags! {
  #[derive(PartialEq, PartialOrd)]
  struct NimbleSub: u16 {
    const Notify = 0x0001;
    const Indicate = 0x0002;
  }
}

#[allow(clippy::type_complexity)]
pub struct BLECharacteristic {
  pub(crate) uuid: ble_uuid_any_t,
  pub(crate) handle: u16,
  pub(crate) properties: NimbleProperties,
  value: AttValue,
  on_read: Option<Box<dyn FnMut(&mut AttValue) + Send + Sync>>,
  subscribed_list: Vec<(u16, NimbleSub)>,
}

impl BLECharacteristic {
  pub(crate) fn new(uuid: BleUuid, properties: NimbleProperties) -> Self {
    Self {
      uuid: ble_uuid_any_t::from(uuid),
      handle: NULL_HANDLE,
      properties,
      value: AttValue::new(),
      on_read: None,
      subscribed_list: Vec::new(),
    }
  }

  pub fn set_value(&mut self, value: &[u8]) -> &mut Self {
    self.value.set_value(value);
    self
  }

  pub fn on_read(
    &mut self,
    callback: impl FnMut(&mut AttValue) + Send + Sync + 'static,
  ) -> &mut Self {
    self.on_read = Some(Box::new(callback));
    self
  }

  pub fn notify(&self) {
    if self.subscribed_list.is_empty() {
      return;
    }

    let server = BLEDevice::take().get_server();

    for it in &self.subscribed_list {
      let _mtu = unsafe { esp_idf_sys::ble_att_mtu(it.0) - 3 };
      if _mtu == 0 || it.1.is_empty() {
        continue;
      }

      if it.1.contains(NimbleSub::Indicate) && self.properties.contains(NimbleProperties::Indicate)
      {
        if !server.set_indicate_wait(it.0) {
          ::log::error!("prior Indication in progress");
          continue;
        }

        let om = unsafe {
          esp_idf_sys::ble_hs_mbuf_from_flat(
            self.value.value().as_ptr() as _,
            self.value.len() as _,
          )
        };

        let rc = unsafe { esp_idf_sys::ble_gattc_indicate_custom(it.0, self.handle, om) };
        if rc != 0 {
          server.clear_indicate_wait(it.0);
        }
      } else if it.1.contains(NimbleSub::Indicate)
        && self.properties.contains(NimbleProperties::Indicate)
      {
        let om = unsafe {
          esp_idf_sys::ble_hs_mbuf_from_flat(
            self.value.value().as_ptr() as _,
            self.value.len() as _,
          )
        };
        unsafe { esp_idf_sys::ble_gattc_notify_custom(it.0, self.handle, om) };
      }
    }
  }

  pub(super) extern "C" fn handle_gap_event(
    conn_handle: u16,
    _attr_handle: u16,
    ctxt: *mut esp_idf_sys::ble_gatt_access_ctxt,
    arg: *mut c_void,
  ) -> i32 {
    let ctxt = unsafe { &*ctxt };

    let mutex = unsafe { &mut *(arg as *mut Mutex<Self>) };
    let mut characteristic = mutex.lock();

    if unsafe { ble_uuid_cmp((*ctxt.__bindgen_anon_1.chr).uuid, &characteristic.uuid.u) != 0 } {
      return esp_idf_sys::BLE_ATT_ERR_UNLIKELY as _;
    }

    match ctxt.op as _ {
      esp_idf_sys::BLE_GATT_ACCESS_OP_READ_CHR => {
        let mut desc = esp_idf_sys::ble_gap_conn_desc::default();
        let rc = unsafe { esp_idf_sys::ble_gap_conn_find(conn_handle, &mut desc) };
        assert_eq!(rc, 0);

        unsafe {
          if (*(ctxt.om)).om_pkthdr_len > 8
            || characteristic.value.len() <= (esp_idf_sys::ble_att_mtu(desc.conn_handle) - 3) as _
          {
            let characteristic = UnsafeCell::new(&mut characteristic);
            if let Some(callback) = &mut (*characteristic.get()).on_read {
              callback(&mut (*characteristic.get()).value);
            }
          }
        }

        ble_npl_hw_enter_critical();
        let value = characteristic.value.value();
        let rc = unsafe { os_mbuf_append(ctxt.om, value.as_ptr() as _, value.len() as _) };
        ble_npl_hw_exit_critical();
        if rc == 0 {
          0
        } else {
          esp_idf_sys::BLE_ATT_ERR_INSUFFICIENT_RES as _
        }
      }
      esp_idf_sys::BLE_GATT_ACCESS_OP_WRITE_CHR => 0,
      _ => esp_idf_sys::BLE_ATT_ERR_UNLIKELY as _,
    }
  }

  pub(super) fn subscribe(
    &mut self,
    subscribe: &esp_idf_sys::ble_gap_event__bindgen_ty_1__bindgen_ty_12,
  ) {
    let mut desc = esp_idf_sys::ble_gap_conn_desc::default();
    if unsafe { esp_idf_sys::ble_gap_conn_find(subscribe.conn_handle, &mut desc) != 0 } {
      return;
    }

    let mut sub_val = NimbleSub::empty();
    if subscribe.cur_notify() > 0 && (self.properties.contains(NimbleProperties::Notify)) {
      sub_val.insert(NimbleSub::Notify);
    }
    if subscribe.cur_indicate() > 0 && (self.properties.contains(NimbleProperties::Indicate)) {
      sub_val.insert(NimbleSub::Indicate);
    }

    if let Some(idx) = self
      .subscribed_list
      .iter()
      .position(|x| x.0 == subscribe.conn_handle)
    {
      if !sub_val.is_empty() {
        self.subscribed_list[idx].1 = sub_val;
      } else {
        self.subscribed_list.swap_remove(idx);
      }
    } else if !sub_val.is_empty() {
      self.subscribed_list.push((subscribe.conn_handle, sub_val));
    }
  }
}
