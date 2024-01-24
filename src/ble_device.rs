use alloc::{ffi::CString, vec::Vec};
use core::ffi::c_void;
use esp_idf_sys::esp_nofail;
use once_cell::sync::Lazy;

use crate::{ble, client::BLEScan, enums::*, BLEAddress, BLEReturnCode, BLESecurity, BLEServer};

#[cfg(not(esp_idf_bt_nimble_ext_adv))]
type BLEAdvertising = crate::BLEAdvertising;
#[cfg(esp_idf_bt_nimble_ext_adv)]
type BLEAdvertising = crate::BLEExtAdvertising;

extern "C" {
  fn ble_store_config_init();
}

#[cfg(esp32)]
extern "C" {
  fn ble_hs_pvcy_rpa_config(enable: u8) -> core::ffi::c_int;
}
#[cfg(esp32)]
const NIMBLE_HOST_DISABLE_PRIVACY: u8 = 0x00;
#[cfg(esp32)]
const NIMBLE_HOST_ENABLE_RPA: u8 = 0x01;
#[cfg(esp32)]
const NIMBLE_HOST_ENABLE_NRPA: u8 = 0x02;

static mut BLE_DEVICE: Lazy<BLEDevice> = Lazy::new(|| {
  BLEDevice::init();
  BLEDevice {
    security: BLESecurity::new(),
  }
});
static mut BLE_SCAN: Lazy<BLEScan> = Lazy::new(BLEScan::new);
pub static mut BLE_SERVER: Lazy<BLEServer> = Lazy::new(BLEServer::new);
static mut BLE_ADVERTISING: Lazy<BLEAdvertising> = Lazy::new(BLEAdvertising::new);

pub static mut OWN_ADDR_TYPE: OwnAddrType = OwnAddrType::Public;
static mut INITIALIZED: bool = false;
static mut SYNCED: bool = false;

pub struct BLEDevice {
  security: BLESecurity,
}

impl BLEDevice {
  pub fn init() {
    // NVS initialisation.
    unsafe {
      if !INITIALIZED {
        let result = esp_idf_sys::nvs_flash_init();
        if result == esp_idf_sys::ESP_ERR_NVS_NO_FREE_PAGES
          || result == esp_idf_sys::ESP_ERR_NVS_NEW_VERSION_FOUND
        {
          ::log::warn!("NVS initialisation failed. Erasing NVS.");
          esp_nofail!(esp_idf_sys::nvs_flash_erase());
          esp_nofail!(esp_idf_sys::nvs_flash_init());
        }

        esp_idf_sys::esp_bt_controller_mem_release(
          esp_idf_sys::esp_bt_mode_t_ESP_BT_MODE_CLASSIC_BT,
        );

        #[cfg(esp_idf_version_major = "4")]
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
      }

      while !SYNCED {
        esp_idf_sys::vPortYield();
      }

      INITIALIZED = true;
    }
  }

  pub fn take() -> &'static mut Self {
    unsafe { Lazy::force_mut(&mut BLE_DEVICE) }
  }

  /// Shutdown the NimBLE stack/controller
  pub fn deinit() {
    unsafe {
      let ret = esp_idf_sys::nimble_port_stop();
      if ret == 0 {
        esp_idf_sys::nimble_port_deinit();
        #[cfg(esp_idf_version_major = "4")]
        {
          let ret = esp_idf_sys::esp_nimble_hci_and_controller_deinit();
          if ret != esp_idf_sys::ESP_OK {
            ::log::warn!(
              "esp_nimble_hci_and_controller_deinit() failed with error: {}",
              ret
            );
          }
        }
        INITIALIZED = false;
        SYNCED = false;

        if let Some(server) = Lazy::get_mut(&mut BLE_SERVER) {
          server.started = false;
        }
      }
    };
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

  pub fn get_power(&self, power_type: PowerType) -> PowerLevel {
    unsafe { core::mem::transmute(esp_idf_sys::esp_ble_tx_power_get(power_type as _)) }
  }

  /// Get the addresses of all bonded peer device.
  pub fn bonded_addresses(&self) -> Result<Vec<BLEAddress>, BLEReturnCode> {
    let mut peer_id_addrs =
      [esp_idf_sys::ble_addr_t::default(); esp_idf_sys::MYNEWT_VAL_BLE_STORE_MAX_BONDS as _];
    let mut num_peers: core::ffi::c_int = 0;

    unsafe {
      ble!(esp_idf_sys::ble_store_util_bonded_peers(
        peer_id_addrs.as_mut_ptr(),
        &mut num_peers,
        esp_idf_sys::MYNEWT_VAL_BLE_STORE_MAX_BONDS as _,
      ))?
    };

    let mut result = Vec::with_capacity(esp_idf_sys::MYNEWT_VAL_BLE_STORE_MAX_BONDS as _);
    for addr in peer_id_addrs.iter().take(num_peers as _) {
      result.push(BLEAddress::from(*addr));
    }

    Ok(result)
  }

  /// Deletes all bonding information.
  pub fn delete_all_bonds(&self) -> Result<(), BLEReturnCode> {
    unsafe { ble!(esp_idf_sys::ble_store_clear()) }
  }

  /// Deletes a peer bond.
  ///
  /// * `address`: The address of the peer with which to delete bond info.
  pub fn delete_bond(&self, address: &BLEAddress) -> Result<(), BLEReturnCode> {
    unsafe { ble!(esp_idf_sys::ble_gap_unpair(&address.value)) }
  }

  pub fn set_white_list(&mut self, white_list: &[BLEAddress]) -> Result<(), BLEReturnCode> {
    unsafe {
      ble!(esp_idf_sys::ble_gap_wl_set(
        white_list.as_ptr() as _,
        white_list.len() as _
      ))
    }
  }

  pub fn security(&mut self) -> &mut BLESecurity {
    &mut self.security
  }

  /// Set the own address type.
  pub fn set_own_addr_type(&mut self, own_addr_type: OwnAddrType) {
    self._set_own_addr_type(own_addr_type, false);
  }

  /// Set the own address type to non-resolvable random address.
  #[cfg(esp32)]
  pub fn set_own_addr_type_to_non_resolvable_random(&mut self) {
    self._set_own_addr_type(OwnAddrType::Random, true);
  }

  #[allow(unused_variables)]
  fn _set_own_addr_type(&mut self, own_addr_type: OwnAddrType, use_nrpa: bool) {
    unsafe {
      OWN_ADDR_TYPE = own_addr_type;
      match own_addr_type {
        OwnAddrType::Public => {
          #[cfg(esp32)]
          ble_hs_pvcy_rpa_config(NIMBLE_HOST_DISABLE_PRIVACY);
        }
        OwnAddrType::Random => {
          self.security().resolve_rpa();
          #[cfg(esp32)]
          ble_hs_pvcy_rpa_config(if use_nrpa {
            NIMBLE_HOST_ENABLE_NRPA
          } else {
            NIMBLE_HOST_ENABLE_RPA
          });
        }
        OwnAddrType::RpaPublicDefault | OwnAddrType::RpaRandomDefault => {
          self.security().resolve_rpa();
          #[cfg(esp32)]
          ble_hs_pvcy_rpa_config(NIMBLE_HOST_ENABLE_RPA);
        }
      }
    }
  }

  /// Set the own address to be used when the address type is random.
  pub fn set_rnd_addr(&mut self, mut addr: [u8; 6]) -> Result<(), BLEReturnCode> {
    addr.reverse();
    unsafe { ble!(esp_idf_sys::ble_hs_id_set_rnd(addr.as_ptr())) }
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
        core::mem::transmute::<&mut OwnAddrType, &mut u8>(&mut OWN_ADDR_TYPE) as *mut _
      ));

      let mut addr = [0; 6];
      esp_nofail!(esp_idf_sys::ble_hs_id_copy_addr(
        OWN_ADDR_TYPE as _,
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
