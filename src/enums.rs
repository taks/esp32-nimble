use bitflags::bitflags;
use esp_idf_sys::*;

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum SecurityIOCap {
  /// DisplayOnly IO capability
  DisplayOnly = BLE_HS_IO_DISPLAY_ONLY as _,
  /// DisplayYesNo IO capability
  DisplayYesNo = BLE_HS_IO_DISPLAY_YESNO as _,
  /// KeyboardOnly IO capability
  KeyboardOnly = BLE_HS_IO_KEYBOARD_ONLY as _,
  /// NoInputNoOutput IO capability
  NoInputNoOutput = BLE_HS_IO_NO_INPUT_OUTPUT as _,
  /// KeyboardDisplay Only IO capability
  KeyboardDisplay = BLE_HS_IO_KEYBOARD_DISPLAY as _,
}

#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PowerLevel {
  /// Corresponding to -12dbm
  N12 = esp_power_level_t_ESP_PWR_LVL_N12 as _,
  /// Corresponding to  -9dbm
  N9 = esp_power_level_t_ESP_PWR_LVL_N9 as _,
  /// Corresponding to  -6dbm
  N6 = esp_power_level_t_ESP_PWR_LVL_N6 as _,
  /// Corresponding to  -3dbm
  N3 = esp_power_level_t_ESP_PWR_LVL_N3 as _,
  /// Corresponding to   0dbm
  N0 = esp_power_level_t_ESP_PWR_LVL_N0 as _,
  /// Corresponding to  +3dbm
  P3 = esp_power_level_t_ESP_PWR_LVL_P3 as _,
  /// Corresponding to  +6dbm
  P6 = esp_power_level_t_ESP_PWR_LVL_P6 as _,
  /// Corresponding to  +9dbm
  P9 = esp_power_level_t_ESP_PWR_LVL_P9 as _,
}

impl PowerLevel {
  pub fn to_dbm(&self) -> i8 {
    match self {
      PowerLevel::N12 => -12,
      PowerLevel::N9 => -9,
      PowerLevel::N6 => -6,
      PowerLevel::N3 => -3,
      PowerLevel::N0 => 0,
      PowerLevel::P3 => 3,
      PowerLevel::P6 => 6,
      PowerLevel::P9 => 9,
    }
  }
}

#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PowerType {
  /// For connection handle 0
  ConnHdl0 = esp_ble_power_type_t_ESP_BLE_PWR_TYPE_CONN_HDL0 as _,
  /// For connection handle 1
  ConnHdl1 = esp_ble_power_type_t_ESP_BLE_PWR_TYPE_CONN_HDL1 as _,
  /// For connection handle 2
  ConnHdl2 = esp_ble_power_type_t_ESP_BLE_PWR_TYPE_CONN_HDL2 as _,
  /// For connection handle 3
  ConnHdl3 = esp_ble_power_type_t_ESP_BLE_PWR_TYPE_CONN_HDL3 as _,
  /// For connection handle 4
  ConnHdl4 = esp_ble_power_type_t_ESP_BLE_PWR_TYPE_CONN_HDL4 as _,
  /// For connection handle 5
  ConnHdl5 = esp_ble_power_type_t_ESP_BLE_PWR_TYPE_CONN_HDL5 as _,
  /// For connection handle 6
  ConnHdl6 = esp_ble_power_type_t_ESP_BLE_PWR_TYPE_CONN_HDL6 as _,
  /// For connection handle 7
  ConnHdl7 = esp_ble_power_type_t_ESP_BLE_PWR_TYPE_CONN_HDL7 as _,
  /// For connection handle 8
  ConnHdl8 = esp_ble_power_type_t_ESP_BLE_PWR_TYPE_CONN_HDL8 as _,
  /// For advertising
  Advertising = esp_ble_power_type_t_ESP_BLE_PWR_TYPE_ADV as _,
  /// For scan
  Scan = esp_ble_power_type_t_ESP_BLE_PWR_TYPE_SCAN as _,
  /// For default, if not set other, it will use default value
  Default = esp_ble_power_type_t_ESP_BLE_PWR_TYPE_DEFAULT as _,
}

bitflags! {
  #[repr(transparent)]
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub struct PairKeyDist: u8 {
    /// Accept/Distribute the encryption key.
    const ENC = BLE_SM_PAIR_KEY_DIST_ENC as _;
    /// Accept/Distribute the ID key (IRK).
    const ID = BLE_SM_PAIR_KEY_DIST_ID as _;
    const SIGN = BLE_SM_PAIR_KEY_DIST_SIGN as _;
    const LINK = BLE_SM_PAIR_KEY_DIST_LINK as _;
  }
}

bitflags! {
  #[repr(transparent)]
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub struct AuthReq: u8 {
    /// allow bounding
    const Bond = 0b001;
    /// man in the middle protection
    const Mitm = 0b010;
    /// secure connection pairing
    const Sc = 0b100;
  }
}
