use core::sync::atomic::AtomicU16;

use alloc::boxed::Box;
use alloc::vec::Vec;
use esp_idf_sys::*;
use log::*;
use once_cell::sync::Lazy;

use crate::ble_client::BLEClient;
use crate::ble_scan::BLEScan;
use crate::leaky_box_raw;
use crate::utilities::extend_lifetime_mut;

static mut BLE_DEVICE: Lazy<BLEDevice> = Lazy::new(|| {
  BLEDevice::init();
  BLEDevice {}
});
static mut BLE_SCAN: Lazy<BLEScan> = Lazy::new(BLEScan::new);
static mut CONNECTED_CLIENTS: Lazy<Vec<&mut BLEClient>> = Lazy::new(Vec::new);
static APP_ID_COUNTER: AtomicU16 = AtomicU16::new(0);

pub struct BLEDevice {}

impl BLEDevice {
  fn init() {
    // NVS initialisation.
    unsafe {
      let result = nvs_flash_init();
      if result == ESP_ERR_NVS_NO_FREE_PAGES || result == ESP_ERR_NVS_NEW_VERSION_FOUND {
        warn!("NVS initialisation failed. Erasing NVS.");
        esp_nofail!(nvs_flash_erase());
        esp_nofail!(nvs_flash_init());
      }
    }

    let default_controller_configuration = esp_bt_controller_config_t {
      controller_task_stack_size: ESP_TASK_BT_CONTROLLER_STACK as _,
      controller_task_prio: ESP_TASK_BT_CONTROLLER_PRIO as _,
      hci_uart_no: BT_HCI_UART_NO_DEFAULT as _,
      hci_uart_baudrate: BT_HCI_UART_BAUDRATE_DEFAULT,
      scan_duplicate_mode: SCAN_DUPLICATE_MODE as _,
      scan_duplicate_type: SCAN_DUPLICATE_TYPE_VALUE as _,
      normal_adv_size: NORMAL_SCAN_DUPLICATE_CACHE_SIZE as _,
      mesh_adv_size: MESH_DUPLICATE_SCAN_CACHE_SIZE as _,
      send_adv_reserved_size: SCAN_SEND_ADV_RESERVED_SIZE as _,
      controller_debug_flag: CONTROLLER_ADV_LOST_DEBUG_BIT,
      mode: esp_bt_mode_t_ESP_BT_MODE_BLE as _,
      ble_max_conn: CONFIG_BTDM_CTRL_BLE_MAX_CONN_EFF as _,
      bt_max_acl_conn: CONFIG_BTDM_CTRL_BR_EDR_MAX_ACL_CONN_EFF as _,
      bt_sco_datapath: CONFIG_BTDM_CTRL_BR_EDR_SCO_DATA_PATH_EFF as _,
      auto_latency: BTDM_CTRL_AUTO_LATENCY_EFF != 0,
      bt_legacy_auth_vs_evt: BTDM_CTRL_LEGACY_AUTH_VENDOR_EVT_EFF != 0,
      bt_max_sync_conn: CONFIG_BTDM_CTRL_BR_EDR_MAX_SYNC_CONN_EFF as _,
      ble_sca: CONFIG_BTDM_BLE_SLEEP_CLOCK_ACCURACY_INDEX_EFF as _,
      pcm_role: CONFIG_BTDM_CTRL_PCM_ROLE_EFF as _,
      pcm_polar: CONFIG_BTDM_CTRL_PCM_POLAR_EFF as _,
      hli: BTDM_CTRL_HLI != 0,
      magic: ESP_BT_CONTROLLER_CONFIG_MAGIC_VAL,
    };

    unsafe {
      esp_nofail!(esp_bt_controller_mem_release(
        esp_bt_mode_t_ESP_BT_MODE_CLASSIC_BT
      ));
      esp_nofail!(esp_bt_controller_init(leaky_box_raw!(
        default_controller_configuration
      )));
      esp_nofail!(esp_bt_controller_enable(esp_bt_mode_t_ESP_BT_MODE_BLE));
      esp_nofail!(esp_bluedroid_init());
      esp_nofail!(esp_bluedroid_enable());
      esp_nofail!(esp_ble_gap_register_callback(Some(Self::esp_gap_cb)));
      esp_nofail!(esp_ble_gattc_register_callback(Some(Self::esp_gattc_cb)));
      // client
      // esp_nofail!(esp_ble_gattc_app_register(PROFILE_A_APP_ID));
    }
  }

  pub fn take() -> &'static Self {
    unsafe { Lazy::force(&BLE_DEVICE) }
  }

  pub fn set_device_name(device_name: &str) -> Result<(), EspError> {
    unsafe {
      esp!(esp_ble_gap_set_device_name(
        device_name.as_ptr().cast::<i8>()
      ))
    }
  }

  pub fn get_scan(&self) -> &'static mut BLEScan {
    unsafe { Lazy::force_mut(&mut BLE_SCAN) }
  }

  pub(crate) fn add_device(ble_client: &mut BLEClient) -> u16 {
    let clients = unsafe { Lazy::force_mut(&mut CONNECTED_CLIENTS) };

    clients.push(unsafe { extend_lifetime_mut(ble_client) });

    APP_ID_COUNTER.fetch_add(1, core::sync::atomic::Ordering::SeqCst)
  }

  pub(crate) fn remove_device(ble_client: &mut BLEClient) {
    let clients = unsafe { Lazy::force_mut(&mut CONNECTED_CLIENTS) };
    if let Some(index) = clients.iter().position(|x| core::ptr::eq(*x, ble_client)) {
      clients.swap_remove(index);
    }
  }

  extern "C" fn esp_gap_cb(event: esp_gap_ble_cb_event_t, param: *mut esp_ble_gap_cb_param_t) {
    if let Some(ble_scan) = unsafe { Lazy::get_mut(&mut BLE_SCAN) } {
      ble_scan.handle_gap_event(event, param);
    }
  }
  extern "C" fn esp_gattc_cb(
    event: esp_gattc_cb_event_t,
    gattc_if: esp_gatt_if_t,
    param: *mut esp_ble_gattc_cb_param_t,
  ) {
    let clients = unsafe { Lazy::force_mut(&mut CONNECTED_CLIENTS) };

    for c in clients.iter_mut() {
      c.handle_gattc_event(event, gattc_if, param);
    }
  }
}
