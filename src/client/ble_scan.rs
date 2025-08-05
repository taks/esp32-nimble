use crate::{ble, enums::*, utilities::voidp_to_ref, BLEAdvertisedDevice, BLEError, Signal};
use crate::{BLEAdvertisedData, BLEDevice};
use core::ffi::c_void;
use esp_idf_svc::sys;

/// Scan for ble devices.
///
/// # Examples
///
/// ```
/// let ble_device = BLEDevice::take();
/// let ble_scan = BLEScan::new();
/// let name = "Device Name To Be Found";
/// let device = ble_scan
///   .start(ble_device, 10000, |device, data| {
///     if let Some(device_name) = data.name() {
///       if device_name == name {
///         return Some(*device);
///       }
///     }
///     None
///   })
///   .await
///   .unwrap();
/// ```
pub struct BLEScan {
  scan_params: sys::ble_gap_disc_params,
  signal: Signal<()>,
}

type CbArgType<'a> = (
  &'a mut BLEScan,
  &'a mut (dyn FnMut(&mut BLEScan, &BLEAdvertisedDevice, BLEAdvertisedData<&[u8]>)),
);

impl BLEScan {
  pub fn new() -> Self {
    let mut ret = Self {
      scan_params: sys::ble_gap_disc_params {
        itvl: 0,
        window: 0,
        filter_policy: sys::BLE_HCI_SCAN_FILT_NO_WL as _,
        ..Default::default()
      },
      signal: Signal::new(),
    };
    ret.limited(false);
    ret.filter_duplicates(true);
    ret.active_scan(false).interval(100).window(100);
    ret
  }

  pub fn active_scan(&mut self, active: bool) -> &mut Self {
    self.scan_params.set_passive((!active) as _);
    self
  }

  pub fn filter_duplicates(&mut self, val: bool) -> &mut Self {
    self.scan_params.set_filter_duplicates(val as _);
    self
  }

  /// Set whether or not the BLE controller only report scan results
  /// from devices advertising in limited discovery mode, i.e. directed advertising.
  pub fn limited(&mut self, val: bool) -> &mut Self {
    self.scan_params.set_limited(val as _);
    self
  }

  /// Sets the scan filter policy.
  pub fn filter_policy(&mut self, val: ScanFilterPolicy) -> &mut Self {
    self.scan_params.filter_policy = val.into();
    self
  }

  /// Set the interval to scan.
  pub fn interval(&mut self, interval_msecs: u16) -> &mut Self {
    self.scan_params.itvl = ((interval_msecs as f32) / 0.625) as u16;
    self
  }

  /// Set the window to actively scan.
  pub fn window(&mut self, window_msecs: u16) -> &mut Self {
    self.scan_params.window = ((window_msecs as f32) / 0.625) as u16;
    self
  }

  /// The callback function must return Option type.
  /// If it returns None, the scan continues.
  /// If Some(r) is returned, the scan stops and the start function returns the return value of the callback.
  pub async fn start<F, R>(
    &mut self,
    _ble_device: &BLEDevice,
    duration_ms: i32,
    mut callback: F,
  ) -> Result<Option<R>, BLEError>
  where
    F: FnMut(&BLEAdvertisedDevice, BLEAdvertisedData<&[u8]>) -> Option<R>,
  {
    let mut result: Option<R> = None;

    let mut on_result =
      |scan: &mut Self, device: &BLEAdvertisedDevice, data: BLEAdvertisedData<&[u8]>| {
        if let Some(res) = callback(device, data) {
          result = Some(res);

          if let Err(err) = Self::stop() {
            ::log::warn!("scan stop err: {err:?}");
          }
          scan.signal.signal(());
        }
      };

    let cb_arg: CbArgType = (self, &mut on_result);

    #[cfg(esp_idf_bt_nimble_ext_adv)]
    {
      let mut scan_params = sys::ble_gap_ext_disc_params {
        itvl: cb_arg.0.scan_params.itvl,
        window: cb_arg.0.scan_params.window,
        ..Default::default()
      };
      scan_params.set_passive(cb_arg.0.scan_params.passive());
      unsafe {
        ble!(sys::ble_gap_ext_disc(
          crate::ble_device::OWN_ADDR_TYPE as _,
          (duration_ms / 10) as _,
          0,
          cb_arg.0.scan_params.filter_duplicates(),
          cb_arg.0.scan_params.filter_policy,
          cb_arg.0.scan_params.limited(),
          &scan_params,
          &scan_params,
          Some(Self::handle_gap_event),
          core::ptr::addr_of!(cb_arg) as _,
        ))?;
      }
    }

    #[cfg(not(esp_idf_bt_nimble_ext_adv))]
    unsafe {
      ble!(sys::ble_gap_disc(
        crate::ble_device::OWN_ADDR_TYPE as _,
        duration_ms,
        &cb_arg.0.scan_params,
        Some(Self::handle_gap_event),
        core::ptr::addr_of!(cb_arg) as _,
      ))?;
    }

    cb_arg.0.signal.wait().await;

    Ok(result)
  }

  fn stop() -> Result<(), BLEError> {
    let rc = unsafe { sys::ble_gap_disc_cancel() };
    if rc != 0 && rc != (sys::BLE_HS_EALREADY as _) {
      return BLEError::convert(rc as _);
    }

    Ok(())
  }

  extern "C" fn handle_gap_event(event: *mut sys::ble_gap_event, arg: *mut c_void) -> i32 {
    let event = unsafe { &*event };
    let (scan, on_result) = unsafe { voidp_to_ref::<CbArgType>(arg) };

    match event.type_ as u32 {
      sys::BLE_GAP_EVENT_EXT_DISC | sys::BLE_GAP_EVENT_DISC => {
        #[cfg(esp_idf_bt_nimble_ext_adv)]
        let disc = unsafe { &event.__bindgen_anon_1.ext_disc };

        #[cfg(not(esp_idf_bt_nimble_ext_adv))]
        let disc = unsafe { &event.__bindgen_anon_1.disc };

        let data = BLEAdvertisedData::new(unsafe {
          core::slice::from_raw_parts(disc.data, disc.length_data as _)
        });

        let advertised_device: &BLEAdvertisedDevice = unsafe { core::mem::transmute(disc) };

        on_result(scan, advertised_device, data);
      }
      sys::BLE_GAP_EVENT_DISC_COMPLETE => {
        scan.signal.signal(());
      }
      _ => {}
    }
    0
  }
}
