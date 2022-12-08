use alloc::ffi::CString;
use esp_idf_sys::c_types::c_void;
use esp_idf_sys::{esp, esp_nofail, EspError};
use once_cell::sync::Lazy;

use crate::ble_scan::BLEScan;

static mut BLE_DEVICE: Lazy<BLEDevice> = Lazy::new(|| {
  BLEDevice::init();
  BLEDevice {}
});
static mut BLE_SCAN: Lazy<BLEScan> = Lazy::new(BLEScan::new);
// static mut CONNECTED_CLIENTS: Lazy<Vec<&mut BLEClient>> = Lazy::new(Vec::new);
pub(crate) static mut OWN_ADDR_TYPE: u8 = esp_idf_sys::BLE_OWN_ADDR_PUBLIC as _;
static mut SYNCED: bool = false;

pub struct BLEDevice {}

impl BLEDevice {
  fn init() {
    // NVS initialisation.
    unsafe {
      let result = esp_idf_sys::nvs_flash_init();
      if result == esp_idf_sys::ESP_ERR_NVS_NO_FREE_PAGES
        || result == esp_idf_sys::ESP_ERR_NVS_NEW_VERSION_FOUND
      {
        ::log::warn!("NVS initialisation failed. Erasing NVS.");
        esp_nofail!(esp_idf_sys::nvs_flash_erase());
        esp_nofail!(esp_idf_sys::nvs_flash_init());
      }

      esp_nofail!(esp_idf_sys::esp_bt_controller_mem_release(
        esp_idf_sys::esp_bt_mode_t_ESP_BT_MODE_CLASSIC_BT
      ));

      esp_nofail!(esp_idf_sys::esp_nimble_hci_and_controller_init());

      esp_idf_sys::nimble_port_init();

      esp_idf_sys::ble_hs_cfg.sync_cb = Some(Self::on_sync);
      esp_idf_sys::ble_hs_cfg.reset_cb = Some(Self::on_reset);

      esp_idf_sys::nimble_port_freertos_init(Some(Self::blecent_host_task));

      while !SYNCED {
        esp_idf_sys::vPortYield();
      }
    }
  }

  pub fn take() -> &'static Self {
    unsafe { Lazy::force(&BLE_DEVICE) }
  }

  pub fn get_scan(&self) -> &'static mut BLEScan {
    unsafe { Lazy::force_mut(&mut BLE_SCAN) }
  }

  #[allow(temporary_cstring_as_ptr)]
  pub fn set_device_name(device_name: &str) -> Result<(), EspError> {
    unsafe {
      esp!(esp_idf_sys::ble_svc_gap_device_name_set(
        CString::new(device_name).unwrap().as_ptr()
      ))
    }
  }

  extern "C" fn on_sync() {
    unsafe {
      esp_nofail!(esp_idf_sys::ble_hs_id_infer_auto(
        0,
        &mut OWN_ADDR_TYPE as *mut _
      ));

      let mut addr = [0; 6];
      esp_nofail!(esp_idf_sys::ble_hs_id_copy_addr(
        OWN_ADDR_TYPE,
        addr.as_mut_ptr(),
        core::ptr::null_mut()
      ));
      ::log::info!("Device Address: {:X?}", addr);

      SYNCED = true;
    }
  }

  extern "C" fn on_reset(reason: i32) {
    ::log::info!("Resetting state; reason={}", reason);
  }

  extern "C" fn blecent_host_task(_: *mut c_void) {
    unsafe {
      ::log::info!("BLE Host Task Started");
      esp_idf_sys::nimble_port_run();
      esp_idf_sys::nimble_port_freertos_deinit();
    }
  }
}
