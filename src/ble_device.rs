use alloc::{ffi::CString, vec::Vec};
use core::{
  ffi::c_void,
  sync::atomic::{AtomicBool, Ordering},
};
use esp_idf_svc::sys as esp_idf_sys;
use esp_idf_sys::{esp, esp_nofail, EspError};
use once_cell::sync::Lazy;

use crate::{ble, enums::*, utilities::mutex::Mutex, BLEAddress, BLEError, BLESecurity, BLEServer};

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
pub static mut BLE_SERVER: Lazy<BLEServer> = Lazy::new(BLEServer::new);
static BLE_ADVERTISING: Lazy<Mutex<BLEAdvertising>> =
  Lazy::new(|| Mutex::new(BLEAdvertising::new()));

pub static mut OWN_ADDR_TYPE: OwnAddrType = OwnAddrType::Public;
static INITIALIZED: AtomicBool = AtomicBool::new(false);
static SYNCED: AtomicBool = AtomicBool::new(false);

pub struct BLEDevice {
  security: BLESecurity,
}

impl BLEDevice {
  pub fn init() {
    // NVS initialisation.
    unsafe {
      let initialized = INITIALIZED.load(Ordering::Acquire);
      if !initialized {
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

      loop {
        let syncd = SYNCED.load(Ordering::Acquire);
        if syncd {
          break;
        }
        esp_idf_sys::vPortYield();
      }

      INITIALIZED.store(true, Ordering::Release);
    }
  }

  pub fn take() -> &'static mut Self {
    unsafe { Lazy::force_mut(&mut BLE_DEVICE) }
  }

  /// Shutdown the NimBLE stack/controller
  pub fn deinit() -> Result<(), EspError> {
    unsafe {
      esp!(esp_idf_sys::nimble_port_stop())?;

      #[cfg(esp_idf_version_major = "4")]
      {
        esp_idf_sys::nimble_port_deinit();
        esp!(esp_idf_sys::esp_nimble_hci_and_controller_deinit())?;
      }

      #[cfg(not(esp_idf_version_major = "4"))]
      esp!(esp_idf_sys::nimble_port_deinit())?;

      INITIALIZED.store(false, Ordering::Release);
      SYNCED.store(false, Ordering::Release);

      if let Some(server) = Lazy::get_mut(&mut BLE_SERVER) {
        server.started = false;
      }
    };

    Ok(())
  }

  /// Shutdown the NimBLE stack/controller
  /// server/advertising/scan will be reset.
  pub fn deinit_full() -> Result<(), EspError> {
    Self::deinit()?;
    unsafe {
      #[cfg(not(esp_idf_bt_nimble_ext_adv))]
      if let Some(advertising) = Lazy::get(&BLE_ADVERTISING) {
        advertising.lock().reset().unwrap();
      }

      if let Some(server) = Lazy::get_mut(&mut BLE_SERVER) {
        server.reset();
      }
    }
    Ok(())
  }

  pub fn get_server(&self) -> &'static mut BLEServer {
    unsafe { Lazy::force_mut(&mut BLE_SERVER) }
  }

  pub fn get_advertising(&self) -> &'static Mutex<BLEAdvertising> {
    Lazy::force(&BLE_ADVERTISING)
  }

  pub fn set_power(
    &mut self,
    power_type: PowerType,
    power_level: PowerLevel,
  ) -> Result<(), BLEError> {
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

  /// Sets the preferred ATT MTU; the device will indicate this value in all subsequent ATT MTU exchanges.
  /// The ATT MTU of a connection is equal to the lower of the two peers’preferred MTU values.
  /// The ATT MTU is what dictates the maximum size of any message sent during a GATT procedure.
  ///
  /// The specified MTU must be within the following range: [23, BLE_ATT_MTU_MAX].
  /// 23 is a minimum imposed by the Bluetooth specification;
  /// BLE_ATT_MTU_MAX is a NimBLE compile-time setting.
  pub fn set_preferred_mtu(&self, mtu: u16) -> Result<(), BLEError> {
    unsafe { ble!(esp_idf_sys::ble_att_set_preferred_mtu(mtu)) }
  }

  /// Retrieves the preferred ATT MTU.
  /// This is the value indicated by the device during an ATT MTU exchange.
  pub fn get_preferred_mtu(&self) -> u16 {
    unsafe { esp_idf_sys::ble_att_preferred_mtu() }
  }

  /// Get the addresses of all bonded peer device.
  pub fn bonded_addresses(&self) -> Result<Vec<BLEAddress>, BLEError> {
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
  pub fn delete_all_bonds(&self) -> Result<(), BLEError> {
    unsafe { ble!(esp_idf_sys::ble_store_clear()) }
  }

  /// Deletes a peer bond.
  ///
  /// * `address`: The address of the peer with which to delete bond info.
  pub fn delete_bond(&self, address: &BLEAddress) -> Result<(), BLEError> {
    unsafe { ble!(esp_idf_sys::ble_gap_unpair(&address.value)) }
  }

  pub fn set_white_list(&mut self, white_list: &[BLEAddress]) -> Result<(), BLEError> {
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
  pub fn set_rnd_addr(&mut self, mut addr: [u8; 6]) -> Result<(), BLEError> {
    addr.reverse();
    unsafe { ble!(esp_idf_sys::ble_hs_id_set_rnd(addr.as_ptr())) }
  }

  #[allow(temporary_cstring_as_ptr)]
  pub fn set_device_name(device_name: &str) -> Result<(), BLEError> {
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

      SYNCED.store(true, Ordering::Release);
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
