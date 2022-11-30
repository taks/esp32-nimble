use crate::{ble_advertised_device::BLEAdvertisedDevice, Signal};
use alloc::boxed::Box;
use esp_idf_sys::*;

pub struct BLEScan {
  #[allow(clippy::type_complexity)]
  on_result: Option<Box<dyn FnMut(&BLEAdvertisedDevice) + Send + Sync>>,
  on_completed: Option<Box<dyn FnMut() + Send + Sync>>,
  scan_params: esp_idf_sys::esp_ble_scan_params_t,
  stopped: bool,
  signal: Signal<()>,
}

impl BLEScan {
  pub(crate) fn new() -> Self {
    let mut ret = Self {
      on_result: None,
      on_completed: None,
      scan_params: esp_ble_scan_params_t {
        scan_type: esp_ble_scan_type_t_BLE_SCAN_TYPE_PASSIVE,
        own_addr_type: esp_ble_addr_type_t_BLE_ADDR_TYPE_PUBLIC,
        scan_filter_policy: esp_ble_scan_filter_t_BLE_SCAN_FILTER_ALLOW_ALL,
        scan_interval: 0,
        scan_window: 0,
        scan_duplicate: esp_idf_sys::esp_ble_scan_duplicate_t_BLE_SCAN_DUPLICATE_DISABLE,
      },
      stopped: true,
      signal: Signal::new(),
    };
    ret.interval(100).window(100);
    ret
  }

  pub fn active_scan(&mut self, active: bool) -> &mut Self {
    self.scan_params.scan_type = if active {
      esp_ble_scan_type_t_BLE_SCAN_TYPE_ACTIVE
    } else {
      esp_ble_scan_type_t_BLE_SCAN_TYPE_PASSIVE
    };
    self
  }

  pub fn on_result(
    &mut self,
    callback: impl FnMut(&BLEAdvertisedDevice) + Send + Sync + 'static,
  ) -> &mut Self {
    self.on_result = Some(Box::new(callback));
    self
  }

  pub fn on_completed(&mut self, callback: impl FnMut() + Send + Sync + 'static) -> &mut Self {
    self.on_completed = Some(Box::new(callback));
    self
  }
  pub fn interval(&mut self, interval_msecs: u16) -> &mut Self {
    self.scan_params.scan_interval = ((interval_msecs as f32) / 0.625) as u16;
    self
  }

  pub fn window(&mut self, window_msecs: u16) -> &mut Self {
    self.scan_params.scan_window = ((window_msecs as f32) / 0.625) as u16;
    self
  }

  pub async fn start(&mut self, duration: u32) -> Result<(), EspError> {
    unsafe {
      esp!(esp_idf_sys::esp_ble_gap_set_scan_params(
        &mut self.scan_params
      ))?;
      esp!(esp_ble_gap_start_scanning(duration))?;
    }
    self.stopped = false;

    self.signal.wait().await;
    Ok(())
  }

  pub fn stop(&mut self) -> Result<(), EspError> {
    self.stopped = true;
    unsafe { esp!(esp_ble_gap_stop_scanning()) }
  }

  pub(crate) fn handle_gap_event(
    &mut self,
    event: esp_gap_ble_cb_event_t,
    param: *mut esp_ble_gap_cb_param_t,
  ) {
    #[allow(clippy::single_match)]
    match event {
      esp_idf_sys::esp_gap_ble_cb_event_t_ESP_GAP_BLE_SCAN_RESULT_EVT => {
        let param = unsafe { &(*param).scan_rst };
        match param.search_evt {
          esp_idf_sys::esp_gap_search_evt_t_ESP_GAP_SEARCH_INQ_CMPL_EVT => {
            self.stopped = true;
            if let Some(callback) = self.on_completed.as_mut() {
              callback();
            }
            self.signal.signal(());
          }
          esp_idf_sys::esp_gap_search_evt_t_ESP_GAP_SEARCH_INQ_RES_EVT => {
            if self.stopped {
              return;
            }

            if let Some(callback) = self.on_result.as_mut() {
              let advertised_device = BLEAdvertisedDevice::new(param);
              callback(&advertised_device);
            }
          }
          _ => {}
        }
      }
      esp_idf_sys::esp_gap_ble_cb_event_t_ESP_GAP_BLE_SCAN_STOP_COMPLETE_EVT => {
        // let param = unsafe { &(*param).scan_stop_cmpl };
        log::debug!("ESP_GAP_BLE_SCAN_STOP_COMPLETE_EVT");
        if let Some(callback) = self.on_completed.as_mut() {
          callback();
        }
        self.signal.signal(());
      }
      _ => {}
    }
  }
}
