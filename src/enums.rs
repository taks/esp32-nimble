use esp_idf_sys::*;

#[repr(u8)]
pub enum SecurityIOCap {
  DisplayOnly = BLE_HS_IO_DISPLAY_ONLY as _,
  DisplayYesNo = BLE_HS_IO_DISPLAY_YESNO as _,
  KeyboardOnly = BLE_HS_IO_KEYBOARD_ONLY as _,
  InputOutput = BLE_HS_IO_NO_INPUT_OUTPUT as _,
  KeyboardDisplay = BLE_HS_IO_KEYBOARD_DISPLAY as _,
}
