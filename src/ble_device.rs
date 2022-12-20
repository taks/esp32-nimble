use alloc::ffi::CString;
use core::ffi::c_void;
use esp_idf_sys::esp_nofail;
use once_cell::sync::Lazy;

use crate::{
  ble, client::BLEScan, enums::*, BLEAdvertising, BLEReturnCode, BLESecurity, BLEServer,
};

extern "C" {
  fn ble_store_config_init();
}

static mut BLE_DEVICE: Lazy<BLEDevice> = Lazy::new(|| {
  BLEDevice::init();
  BLEDevice {
    security: BLESecurity::new(),
  }
});
static mut BLE_SCAN: Lazy<BLEScan> = Lazy::new(BLEScan::new);
pub static mut BLE_SERVER: Lazy<BLEServer> = Lazy::new(BLEServer::new);
static mut BLE_ADVERTISING: Lazy<BLEAdvertising> = Lazy::new(BLEAdvertising::new);

pub static mut OWN_ADDR_TYPE: u8 = esp_idf_sys::BLE_OWN_ADDR_PUBLIC as _;
static mut SYNCED: bool = false;

pub struct BLEDevice {
  security: BLESecurity,
}

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

      // Set initial security capabilities
      esp_idf_sys::ble_hs_cfg.sm_io_cap = esp_idf_sys::BLE_HS_IO_NO_INPUT_OUTPUT as _;
      esp_idf_sys::ble_hs_cfg.set_sm_bonding(0);
      esp_idf_sys::ble_hs_cfg.set_sm_mitm(0);
      esp_idf_sys::ble_hs_cfg.set_sm_sc(1);
      esp_idf_sys::ble_hs_cfg.sm_our_key_dist = 1;
      esp_idf_sys::ble_hs_cfg.sm_their_key_dist = 3;
      esp_idf_sys::ble_hs_cfg.store_status_cb = Some(esp_idf_sys::ble_store_util_status_rr);

      ble_store_config_init();

      esp_idf_sys::nimble_port_freertos_init(Some(Self::blecent_host_task));

      while !SYNCED {
        esp_idf_sys::vPortYield();
      }
    }
  }

  pub fn take() -> &'static mut Self {
    unsafe { Lazy::force_mut(&mut BLE_DEVICE) }
  }

  pub fn get_scan(&self) -> &'static mut BLEScan {
    unsafe { Lazy::force_mut(&mut BLE_SCAN) }
  }

  pub fn get_server(&self) -> &'static mut BLEServer {
    unsafe { Lazy::force_mut(&mut BLE_SERVER) }
  }

  pub fn get_advertising(&self) -> &'static mut BLEAdvertising {
    unsafe { Lazy::force_mut(&mut BLE_ADVERTISING) }
  }

  pub fn set_power(
    &mut self,
    power_type: PowerType,
    power_level: PowerLevel,
  ) -> Result<(), BLEReturnCode> {
    unsafe {
      ble!(esp_idf_sys::esp_ble_tx_power_set(
        power_type as _,
        power_level as _
      ))
    }
  }

  pub fn security(&mut self) -> &mut BLESecurity {
    &mut self.security
  }

  #[allow(temporary_cstring_as_ptr)]
  pub fn set_device_name(device_name: &str) -> Result<(), BLEReturnCode> {
    unsafe {
      ble!(esp_idf_sys::ble_svc_gap_device_name_set(
        CString::new(device_name).unwrap().as_ptr().cast()
      ))
    }
  }

  extern "C" fn on_sync() {
    unsafe {
      esp_idf_sys::ble_hs_util_ensure_addr(0);

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
      ::log::info!(
        "Device Address: {:X}:{:X}:{:X}:{:X}:{:X}:{:X}",
        addr[5],
        addr[4],
        addr[3],
        addr[2],
        addr[1],
        addr[0]
      );

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
