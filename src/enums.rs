use esp_idf_sys::*;

#[repr(u8)]
pub enum SecurityIOCap {
  DisplayOnly = BLE_HS_IO_DISPLAY_ONLY as _,
  DisplayYesNo = BLE_HS_IO_DISPLAY_YESNO as _,
  KeyboardOnly = BLE_HS_IO_KEYBOARD_ONLY as _,
  NoInputOutput = BLE_HS_IO_NO_INPUT_OUTPUT as _,
  KeyboardDisplay = BLE_HS_IO_KEYBOARD_DISPLAY as _,
}

#[repr(u32)]
pub enum PowerLevel {
  N12 = esp_power_level_t_ESP_PWR_LVL_N12 as _,
  N9 = esp_power_level_t_ESP_PWR_LVL_N9 as _,
  N6 = esp_power_level_t_ESP_PWR_LVL_N6 as _,
  N3 = esp_power_level_t_ESP_PWR_LVL_N3 as _,
  N0 = esp_power_level_t_ESP_PWR_LVL_N0 as _,
  P3 = esp_power_level_t_ESP_PWR_LVL_P3 as _,
  P6 = esp_power_level_t_ESP_PWR_LVL_P6 as _,
  P9 = esp_power_level_t_ESP_PWR_LVL_P9 as _,
}

#[repr(u32)]
pub enum PowerType {
  ConnHdl0 = esp_ble_power_type_t_ESP_BLE_PWR_TYPE_CONN_HDL0 as _,
  Default = esp_ble_power_type_t_ESP_BLE_PWR_TYPE_DEFAULT as _,
}
