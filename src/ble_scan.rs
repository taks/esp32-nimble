use crate::{ble, BLEAdvertisedDevice, BLEReturnCode, Signal};
use alloc::boxed::Box;
use esp_idf_sys::{c_types::c_void, *};

pub struct BLEScan {
  #[allow(clippy::type_complexity)]
  on_result: Option<Box<dyn FnMut(&BLEAdvertisedDevice) + Send + Sync>>,
  on_completed: Option<Box<dyn FnMut() + Send + Sync>>,
  scan_params: esp_idf_sys::ble_gap_disc_params,
  stopped: bool,
  signal: Signal<()>,
}

impl BLEScan {
  pub(crate) fn new() -> Self {
    let mut ret = Self {
      on_result: None,
      on_completed: None,
      scan_params: esp_idf_sys::ble_gap_disc_params {
        itvl: 0,
        window: 0,
        filter_policy: esp_idf_sys::BLE_HCI_SCAN_FILT_NO_WL as _,
        _bitfield_align_1: [0; 0],
        _bitfield_1: __BindgenBitfieldUnit::new([0; 1]),
      },
      stopped: true,
      signal: Signal::new(),
    };
    ret.scan_params.set_limited(0);
    ret.scan_params.set_filter_duplicates(true as _);
    ret.active_scan(false).interval(100).window(100);
    ret
  }

  pub fn active_scan(&mut self, active: bool) -> &mut Self {
    self.scan_params.set_passive((!active) as _);
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
    self.scan_params.itvl = ((interval_msecs as f32) / 0.625) as u16;
    self
  }

  pub fn window(&mut self, window_msecs: u16) -> &mut Self {
    self.scan_params.window = ((window_msecs as f32) / 0.625) as u16;
    self
  }

  pub async fn start(&mut self, duration_ms: i32) -> Result<(), BLEReturnCode> {
    unsafe {
      ble!(esp_idf_sys::ble_gap_disc(
        crate::ble_device::OWN_ADDR_TYPE,
        duration_ms,
        &self.scan_params,
        Some(Self::handle_gap_event),
        self as *mut Self as _,
      ))?;
    }
    self.stopped = false;

    self.signal.wait().await;
    Ok(())
  }

  pub fn stop(&mut self) -> Result<(), EspError> {
    self.stopped = true;
    let rc = unsafe { esp_idf_sys::ble_gap_disc_cancel() };
    if rc != 0 && rc != (esp_idf_sys::BLE_HS_EALREADY as _) {
      return EspError::convert(esp_idf_sys::ESP_FAIL);
    }

    if let Some(callback) = self.on_completed.as_mut() {
      callback();
    }
    self.signal.signal(());

    Ok(())
  }

  extern "C" fn handle_gap_event(event: *mut esp_idf_sys::ble_gap_event, arg: *mut c_void) -> i32 {
    let event = unsafe { &*event };
    let scan = unsafe { &mut *(arg as *mut Self) };

    match event.type_ as u32 {
      esp_idf_sys::BLE_GAP_EVENT_EXT_DISC | esp_idf_sys::BLE_GAP_EVENT_DISC => {
        let disc = unsafe { &event.__bindgen_anon_1.disc };

        let advertised_device = BLEAdvertisedDevice::new(disc);
        if let Some(callback) = scan.on_result.as_mut() {
          callback(&advertised_device);
        }
      }
      esp_idf_sys::BLE_GAP_EVENT_DISC_COMPLETE => {
        if let Some(callback) = scan.on_completed.as_mut() {
          callback();
        }
        scan.signal.signal(());
      }
      _ => {}
    }
    0
  }
}
