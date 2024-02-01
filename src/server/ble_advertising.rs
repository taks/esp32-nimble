use core::ffi::{c_int, c_void};

use crate::{
  ble, enums::*, utilities::voidp_to_ref, BLEAdvertisementData, BLEReturnCode, BLEServer,
};
use alloc::boxed::Box;
use once_cell::sync::Lazy;

const BLE_HS_ADV_MAX_SZ: u8 = esp_idf_sys::BLE_HS_ADV_MAX_SZ as u8;

// Copied from ble_hs.h, for some reason esp_idf_sys didn't pick this up.
const BLE_HS_FOREVER: i32 = i32::MAX;

pub struct BLEAdvertising {
  adv_params: esp_idf_sys::ble_gap_adv_params,
  scan_response: bool,
  on_complete: Option<Box<dyn FnMut(c_int) + Send + Sync>>,
}

impl BLEAdvertising {
  #[allow(dead_code)]
  pub(crate) fn new() -> Self {
    let mut ret = Self {
      adv_params: esp_idf_sys::ble_gap_adv_params::default(),
      scan_response: true,
      on_complete: None,
    };

    ret.reset().unwrap();
    ret
  }

  pub fn reset(&mut self) -> Result<(), BLEReturnCode> {
    if self.is_advertising() {
      self.stop()?;
    }

    self.adv_params.conn_mode = esp_idf_sys::BLE_GAP_CONN_MODE_UND as _;
    self.adv_params.disc_mode = esp_idf_sys::BLE_GAP_DISC_MODE_GEN as _;
    self.scan_response = true;

    Ok(())
  }

  pub fn set_data(&mut self, data: &BLEAdvertisementData) -> Result<(), BLEReturnCode> {
    let mut adv_data = data.adv_data;
    let mut scan_data = esp_idf_sys::ble_hs_adv_fields::default();

    if self.adv_params.conn_mode == (ConnMode::Non as _) && !self.scan_response {
      adv_data.flags = 0;
    } else {
      adv_data.flags =
        (esp_idf_sys::BLE_HS_ADV_F_DISC_GEN | esp_idf_sys::BLE_HS_ADV_F_BREDR_UNSUP) as _;
    }

    let mut payload_len: u8 = if adv_data.flags > 0 { 2 + 1 } else { 0 };

    if adv_data.mfg_data_len > 0 {
      payload_len += 2 + adv_data.mfg_data_len;
    }

    if adv_data.svc_data_uuid16_len > 0 {
      payload_len += 2 + adv_data.svc_data_uuid16_len;
    }

    if adv_data.svc_data_uuid32_len > 0 {
      payload_len += 2 + adv_data.svc_data_uuid32_len;
    }

    if adv_data.svc_data_uuid128_len > 0 {
      payload_len += 2 + adv_data.svc_data_uuid128_len;
    }

    if adv_data.uri_len > 0 {
      payload_len += 2 + adv_data.uri_len;
    }

    if adv_data.appearance_is_present() > 0 {
      payload_len += 2 + (esp_idf_sys::BLE_HS_ADV_APPEARANCE_LEN as u8);
    }

    if adv_data.tx_pwr_lvl_is_present() > 0 {
      payload_len += 2 + (esp_idf_sys::BLE_HS_ADV_TX_PWR_LVL_LEN as u8);
    }

    if !adv_data.slave_itvl_range.is_null() {
      payload_len += 2 + (esp_idf_sys::BLE_HS_ADV_SLAVE_ITVL_RANGE_LEN as u8);
    }

    if data.service_uuids_16.is_empty() {
      adv_data.set_uuids16_is_complete(0);
      adv_data.uuids16 = core::ptr::null();
      adv_data.num_uuids16 = 0;
    } else {
      adv_data.set_uuids16_is_complete(1);
      adv_data.uuids16 = data.service_uuids_16.as_ptr();
      adv_data.num_uuids16 = data.service_uuids_16.len() as _;
      payload_len += 2 + 4 * (data.service_uuids_16.len() - 1) as u8;
    }

    if data.service_uuids_32.is_empty() {
      adv_data.set_uuids32_is_complete(0);
      adv_data.uuids32 = core::ptr::null();
      adv_data.num_uuids32 = 0;
    } else {
      adv_data.set_uuids32_is_complete(1);
      adv_data.uuids32 = data.service_uuids_32.as_ptr();
      adv_data.num_uuids32 = data.service_uuids_32.len() as _;
      payload_len += 4 + 6 * (data.service_uuids_32.len() - 1) as u8;
    }

    if data.service_uuids_128.is_empty() {
      adv_data.set_uuids128_is_complete(0);
      adv_data.uuids128 = core::ptr::null();
      adv_data.num_uuids128 = 0;
    } else {
      adv_data.set_uuids128_is_complete(1);
      adv_data.uuids128 = data.service_uuids_128.as_ptr();
      adv_data.num_uuids128 = data.service_uuids_128.len() as _;
      payload_len += 16 + 18 * (data.service_uuids_128.len() - 1) as u8;
    }

    if payload_len + 2 + adv_data.name_len > BLE_HS_ADV_MAX_SZ {
      if self.scan_response {
        scan_data.name = adv_data.name;
        scan_data.name_len = adv_data.name_len;
        if scan_data.name_len > BLE_HS_ADV_MAX_SZ - 2 {
          scan_data.name_len = BLE_HS_ADV_MAX_SZ - 2;
          scan_data.set_name_is_complete(0);
        } else {
          scan_data.set_name_is_complete(1);
        }

        adv_data.name = core::ptr::null();
        adv_data.name_len = 0;
        adv_data.set_name_is_complete(0);
      } else {
        if adv_data.tx_pwr_lvl_is_present() > 0 {
          adv_data.set_tx_pwr_lvl_is_present(0);
          payload_len -= 2 + 1;
        }
        if adv_data.name_len > (BLE_HS_ADV_MAX_SZ - payload_len - 2) {
          adv_data.name_len = BLE_HS_ADV_MAX_SZ - payload_len - 2;
          adv_data.set_name_is_complete(0);
        }
      }
    }

    unsafe {
      if self.scan_response {
        ble!(esp_idf_sys::ble_gap_adv_rsp_set_fields(&scan_data))?;
      }

      ble!(esp_idf_sys::ble_gap_adv_set_fields(&adv_data))
    }
  }

  pub fn set_raw_data(&mut self, data: &[u8]) -> Result<(), BLEReturnCode> {
    unsafe {
      ble!(esp_idf_sys::ble_gap_adv_set_data(
        data.as_ptr(),
        data.len() as i32
      ))
    }
  }

  pub fn set_raw_scan_response_data(&mut self, data: &[u8]) -> Result<(), BLEReturnCode> {
    unsafe {
      ble!(esp_idf_sys::ble_gap_adv_rsp_set_data(
        data.as_ptr(),
        data.len() as i32
      ))
    }
  }

  /// Set the type of advertisment to use.
  pub fn advertisement_type(&mut self, adv_type: ConnMode) -> &mut Self {
    self.adv_params.conn_mode = adv_type as _;
    self
  }

  /// Set discoverable mode.
  pub fn disc_mode(&mut self, mode: DiscMode) -> &mut Self {
    self.adv_params.disc_mode = mode as _;
    self
  }

  /// Set the duty cycle for advertisement_type.
  ///
  /// Valid only if advertisement_type is directed-connectable.
  pub fn high_duty_cycle(&mut self, val: bool) -> &mut Self {
    self.adv_params.set_high_duty_cycle(val as _);
    self
  }

  /// Set the minimum advertising interval.
  ///
  /// * `interval`: advertising interval in 0.625ms units, 0 = use default.
  pub fn min_interval(&mut self, interval: u16) -> &mut Self {
    self.adv_params.itvl_min = interval;
    self
  }

  /// Set the maximum advertising interval.
  ///
  /// * `interval`: advertising interval in 0.625ms units, 0 = use default.
  pub fn max_interval(&mut self, interval: u16) -> &mut Self {
    self.adv_params.itvl_max = interval;
    self
  }

  /// Set if scan response is available.
  pub fn scan_response(&mut self, value: bool) -> &mut Self {
    self.scan_response = value;
    self
  }

  /// Set the filtering for the scan filter.
  pub fn filter_policy(&mut self, value: AdvFilterPolicy) -> &mut Self {
    self.adv_params.filter_policy = value.into();
    self
  }

  /// Start advertising.
  /// Advertising not stop until it is manually stopped.
  pub fn start(&mut self) -> Result<(), BLEReturnCode> {
    self.start_with_duration(BLE_HS_FOREVER)
  }

  /// Start advertising.
  pub fn start_with_duration(&mut self, duration_ms: i32) -> Result<(), BLEReturnCode> {
    let mut server = unsafe { Lazy::get_mut(&mut crate::ble_device::BLE_SERVER) };
    if let Some(server) = server.as_mut() {
      if !server.started {
        server.start()?;
      }
    }

    if self.adv_params.conn_mode == (ConnMode::Non as _) && !self.scan_response {
      self.adv_params.disc_mode = esp_idf_sys::BLE_GAP_DISC_MODE_NON as _;
    } else {
      self.adv_params.disc_mode = esp_idf_sys::BLE_GAP_DISC_MODE_GEN as _;
    }

    let handle_gap_event = if server.is_some() {
      BLEServer::handle_gap_event
    } else {
      Self::handle_gap_event
    };
    unsafe {
      ble!(esp_idf_sys::ble_gap_adv_start(
        crate::ble_device::OWN_ADDR_TYPE as _,
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

  pub fn on_complete(&mut self, callback: impl FnMut(c_int) + Send + Sync + 'static) -> &mut Self {
    self.on_complete = Some(Box::new(callback));
    self
  }

  pub(crate) extern "C" fn handle_gap_event(
    event: *mut esp_idf_sys::ble_gap_event,
    arg: *mut c_void,
  ) -> i32 {
    let event = unsafe { &*event };
    let adv = unsafe { voidp_to_ref::<Self>(arg) };

    if event.type_ == esp_idf_sys::BLE_GAP_EVENT_ADV_COMPLETE as _ {
      if let Some(callback) = adv.on_complete.as_mut() {
        callback(unsafe { event.__bindgen_anon_1.adv_complete.reason });
      }
    }

    0
  }
}

unsafe impl Send for BLEAdvertising {}
