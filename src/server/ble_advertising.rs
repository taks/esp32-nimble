use core::ffi::c_void;

use crate::{ble, enums::PowerType, utilities::BleUuid, BLEDevice, BLEReturnCode, BLEServer};
use alloc::{ffi::CString, vec::Vec};
use once_cell::sync::Lazy;

const BLE_HS_ADV_MAX_SZ: u8 = esp_idf_sys::BLE_HS_ADV_MAX_SZ as u8;

pub struct BLEAdvertising {
  adv_data: esp_idf_sys::ble_hs_adv_fields,
  scan_data: esp_idf_sys::ble_hs_adv_fields,
  adv_params: esp_idf_sys::ble_gap_adv_params,
  service_uuids_16: Vec<esp_idf_sys::ble_uuid16_t>,
  service_uuids_32: Vec<esp_idf_sys::ble_uuid32_t>,
  service_uuids_128: Vec<esp_idf_sys::ble_uuid128_t>,
  service_data16: Vec<u8>,
  service_data32: Vec<u8>,
  service_data128: Vec<u8>,
  adv_data_set: bool,
  custom_adv_data: bool,
  custom_scan_response_data: bool,
  name: Option<CString>,
  mfg_data: Vec<u8>,
  scan_response: bool,
}

impl BLEAdvertising {
  pub(crate) fn new() -> Self {
    let mut ret = Self {
      adv_data: esp_idf_sys::ble_hs_adv_fields::default(),
      scan_data: esp_idf_sys::ble_hs_adv_fields::default(),
      adv_params: esp_idf_sys::ble_gap_adv_params::default(),
      service_uuids_16: Vec::new(),
      service_uuids_32: Vec::new(),
      service_uuids_128: Vec::new(),
      service_data16: Vec::new(),
      service_data32: Vec::new(),
      service_data128: Vec::new(),
      adv_data_set: false,
      custom_adv_data: false,
      custom_scan_response_data: false,
      name: None,
      mfg_data: Vec::new(),
      scan_response: true,
    };

    ret.reset().unwrap();
    ret
  }

  pub fn reset(&mut self) -> Result<(), BLEReturnCode> {
    if self.is_advertising() {
      self.stop()?;
    }

    self.adv_data = esp_idf_sys::ble_hs_adv_fields::default();
    self.scan_data = esp_idf_sys::ble_hs_adv_fields::default();
    self.adv_params = esp_idf_sys::ble_gap_adv_params::default();
    self.service_uuids_16.clear();
    self.service_uuids_32.clear();
    self.service_uuids_128.clear();
    self.service_data16.clear();
    self.service_data32.clear();
    self.service_data128.clear();

    let ble_device = BLEDevice::take();
    self.adv_data.tx_pwr_lvl = ble_device.get_power(PowerType::Advertising).to_dbm();

    self.adv_data.flags =
      (esp_idf_sys::BLE_HS_ADV_F_DISC_GEN | esp_idf_sys::BLE_HS_ADV_F_BREDR_UNSUP) as _;
    self.adv_params.conn_mode = esp_idf_sys::BLE_GAP_CONN_MODE_UND as _;
    self.adv_params.disc_mode = esp_idf_sys::BLE_GAP_DISC_MODE_GEN as _;
    self.scan_response = true;

    self.adv_data_set = false;
    self.custom_adv_data = false;
    self.custom_scan_response_data = false;

    Ok(())
  }

  pub fn add_service_uuid(&mut self, uuid: BleUuid) -> &mut Self {
    let x = esp_idf_sys::ble_uuid_any_t::from(uuid);
    match uuid {
      BleUuid::Uuid16(_) => {
        self.service_uuids_16.push(unsafe { x.u16_ });
      }
      BleUuid::Uuid32(_) => {
        self.service_uuids_32.push(unsafe { x.u32_ });
      }
      BleUuid::Uuid128(_) => {
        self.service_uuids_128.push(unsafe { x.u128_ });
      }
    }
    self.adv_data_set = false;

    self
  }

  /// Set the device appearance in the advertising data.
  pub fn appearance(&mut self, appearance: u16) -> &mut Self {
    self.adv_data.appearance = appearance;
    self.adv_data.set_appearance_is_present(1);
    self.adv_data_set = false;

    self
  }

  /// Add the transmission power level to the advertisement packet.
  pub fn add_tx_power(&mut self) -> &mut Self {
    self.adv_data.set_tx_pwr_lvl_is_present(1);
    self.adv_data_set = false;

    self
  }

  /// Set the advertised name of the device.
  pub fn name(&mut self, name: &str) -> &mut Self {
    self.adv_data.name_len = name.len() as _;

    self.name = Some(CString::new(name).unwrap());
    self.adv_data.name = self.name.as_mut().unwrap().as_ptr().cast();
    self.adv_data.set_name_is_complete(1);
    self.adv_data_set = false;

    self
  }

  pub fn custom_adv_data(&mut self, data: &[u8]) -> Result<(), BLEReturnCode> {
    unsafe {
      ble!(esp_idf_sys::ble_gap_adv_set_data(
        data.as_ptr(),
        data.len() as i32
      ))?
    }

    self.custom_adv_data = true;

    Ok(())
  }

  pub fn custom_scan_response_data(&mut self, data: &[u8]) -> Result<(), BLEReturnCode> {
    unsafe {
      ble!(esp_idf_sys::ble_gap_adv_rsp_set_data(
        data.as_ptr(),
        data.len() as i32
      ))?
    }

    self.custom_scan_response_data = true;

    Ok(())
  }

  pub fn manufacturer_data(&mut self, data: &[u8]) -> &mut Self {
    self.mfg_data.clear();
    self.mfg_data.extend_from_slice(data);
    self.adv_data.mfg_data = self.mfg_data.as_ptr();
    self.adv_data.mfg_data_len = data.len() as _;
    self.adv_data_set = false;

    self
  }

  pub fn service_data(&mut self, uuid: BleUuid, data: &[u8]) {
    match uuid {
      BleUuid::Uuid16(uuid) => {
        self.service_data16.clear();
        self.service_data16.extend_from_slice(&uuid.to_ne_bytes());
        self.service_data16.extend_from_slice(data);
        self.adv_data.svc_data_uuid16 = self.service_data16.as_ptr();
        self.adv_data.svc_data_uuid16_len = if data.is_empty() {
          0
        } else {
          self.service_data16.len() as _
        }
      }
      BleUuid::Uuid32(uuid) => {
        self.service_data32.clear();
        self.service_data32.extend_from_slice(&uuid.to_ne_bytes());
        self.service_data32.extend_from_slice(data);
        self.adv_data.svc_data_uuid32 = self.service_data32.as_ptr();
        self.adv_data.svc_data_uuid32_len = if data.is_empty() {
          0
        } else {
          self.service_data32.len() as _
        }
      }
      BleUuid::Uuid128(uuid) => {
        self.service_data128.clear();
        self.service_data128.extend_from_slice(&uuid);
        self.service_data128.extend_from_slice(data);
        self.adv_data.svc_data_uuid128 = self.service_data128.as_ptr();
        self.adv_data.svc_data_uuid128_len = if data.is_empty() {
          0
        } else {
          self.service_data128.len() as _
        }
      }
    }
  }

  pub fn scan_response(&mut self, value: bool) -> &mut Self {
    self.scan_response = value;
    self.adv_data_set = false;

    self
  }

  pub fn start(&mut self) -> Result<(), BLEReturnCode> {
    self.start_with_duration(i32::MAX)
  }

  fn start_with_duration(&mut self, duration_ms: i32) -> Result<(), BLEReturnCode> {
    let mut server = unsafe { Lazy::get_mut(&mut crate::ble_device::BLE_SERVER) };
    if let Some(server) = server.as_mut() {
      if !server.started {
        server.start()?;
      }
    }

    self.adv_params.disc_mode = esp_idf_sys::BLE_GAP_DISC_MODE_GEN as _;
    self.adv_data.flags =
      (esp_idf_sys::BLE_HS_ADV_F_DISC_GEN | esp_idf_sys::BLE_HS_ADV_F_BREDR_UNSUP) as _;

    if !self.custom_adv_data && !self.adv_data_set {
      let mut payload_len: u8 = if self.adv_data.flags > 0 { 2 + 1 } else { 0 };

      if self.adv_data.mfg_data_len > 0 {
        payload_len += 2 + self.adv_data.mfg_data_len;
      }

      if self.adv_data.svc_data_uuid16_len > 0 {
        payload_len += 2 + self.adv_data.svc_data_uuid16_len;
      }

      if self.adv_data.svc_data_uuid32_len > 0 {
        payload_len += 2 + self.adv_data.svc_data_uuid32_len;
      }

      if self.adv_data.svc_data_uuid128_len > 0 {
        payload_len += 2 + self.adv_data.svc_data_uuid128_len;
      }

      if self.adv_data.uri_len > 0 {
        payload_len += 2 + self.adv_data.uri_len;
      }

      if self.adv_data.appearance_is_present() > 0 {
        payload_len += 2 + (esp_idf_sys::BLE_HS_ADV_APPEARANCE_LEN as u8);
      }

      if self.adv_data.tx_pwr_lvl_is_present() > 0 {
        payload_len += 2 + (esp_idf_sys::BLE_HS_ADV_TX_PWR_LVL_LEN as u8);
      }

      if !self.adv_data.slave_itvl_range.is_null() {
        payload_len += 2 + (esp_idf_sys::BLE_HS_ADV_SLAVE_ITVL_RANGE_LEN as u8);
      }

      if self.service_uuids_16.is_empty() {
        self.adv_data.set_uuids16_is_complete(0);
        self.adv_data.uuids16 = core::ptr::null();
        self.adv_data.num_uuids16 = 0;
      } else {
        self.adv_data.set_uuids16_is_complete(1);
        self.adv_data.uuids16 = self.service_uuids_16.as_ptr();
        self.adv_data.num_uuids16 = self.service_uuids_16.len() as _;
        payload_len += 2 + 4 * (self.service_uuids_16.len() - 1) as u8;
      }

      if self.service_uuids_32.is_empty() {
        self.adv_data.set_uuids32_is_complete(0);
        self.adv_data.uuids32 = core::ptr::null();
        self.adv_data.num_uuids32 = 0;
      } else {
        self.adv_data.set_uuids32_is_complete(1);
        self.adv_data.uuids32 = self.service_uuids_32.as_ptr();
        self.adv_data.num_uuids32 = self.service_uuids_32.len() as _;
        payload_len += 4 + 6 * (self.service_uuids_32.len() - 1) as u8;
      }

      if self.service_uuids_128.is_empty() {
        self.adv_data.set_uuids128_is_complete(0);
        self.adv_data.uuids128 = core::ptr::null();
        self.adv_data.num_uuids128 = 0;
      } else {
        self.adv_data.set_uuids128_is_complete(1);
        self.adv_data.uuids128 = self.service_uuids_128.as_ptr();
        self.adv_data.num_uuids128 = self.service_uuids_128.len() as _;
        payload_len += 16 + 18 * (self.service_uuids_128.len() - 1) as u8;
      }

      if payload_len + 2 + self.adv_data.name_len > BLE_HS_ADV_MAX_SZ {
        if self.scan_response && !self.custom_scan_response_data {
          self.scan_data.name = self.adv_data.name;
          self.scan_data.name_len = self.adv_data.name_len;
          if self.scan_data.name_len > BLE_HS_ADV_MAX_SZ - 2 {
            self.scan_data.name_len = BLE_HS_ADV_MAX_SZ - 2;
            self.scan_data.set_name_is_complete(0);
          } else {
            self.scan_data.set_name_is_complete(1);
          }
          self.adv_data.name = core::ptr::null();
          self.adv_data.name_len = 0;
          self.adv_data.set_name_is_complete(0);
        } else {
          if self.adv_data.tx_pwr_lvl_is_present() > 0 {
            self.adv_data.set_tx_pwr_lvl_is_present(0);
            payload_len -= 2 + 1;
          }
          if self.adv_data.name_len > (BLE_HS_ADV_MAX_SZ - payload_len - 2) {
            self.adv_data.name_len = BLE_HS_ADV_MAX_SZ - payload_len - 2;
            self.adv_data.set_name_is_complete(0);
          }
        }
      }

      unsafe {
        if self.scan_response && !self.custom_scan_response_data {
          ble!(esp_idf_sys::ble_gap_adv_rsp_set_fields(&self.scan_data))?;
        }

        ble!(esp_idf_sys::ble_gap_adv_set_fields(&self.adv_data))?;
      }
    }

    let handle_gap_event = if server.is_some() {
      BLEServer::handle_gap_event
    } else {
      Self::handle_gap_event
    };
    unsafe {
      ble!(esp_idf_sys::ble_gap_adv_start(
        crate::ble_device::OWN_ADDR_TYPE,
        core::ptr::null(),
        duration_ms,
        &self.adv_params,
        Some(handle_gap_event),
        self as *mut Self as _,
      ))?;
    }

    Ok(())
  }

  pub fn stop(&self) -> Result<(), BLEReturnCode> {
    unsafe { ble!(esp_idf_sys::ble_gap_adv_stop()) }
  }

  pub fn is_advertising(&self) -> bool {
    unsafe { esp_idf_sys::ble_gap_adv_active() != 0 }
  }

  extern "C" fn handle_gap_event(event: *mut esp_idf_sys::ble_gap_event, arg: *mut c_void) -> i32 {
    let _event = unsafe { &*event };
    let _adv = unsafe { &mut *(arg as *mut Self) };

    0
  }
}
