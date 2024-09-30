use bitflags::bitflags;
use esp_idf_svc::sys::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};

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

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum OwnAddrType {
  Public = BLE_OWN_ADDR_PUBLIC as _,
  Random = BLE_OWN_ADDR_RANDOM as _,
  RpaPublicDefault = BLE_OWN_ADDR_RPA_PUBLIC_DEFAULT as _,
  RpaRandomDefault = BLE_OWN_ADDR_RPA_RANDOM_DEFAULT as _,
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ConnMode {
  /// non-connectable (3.C.9.3.2)
  Non = BLE_GAP_CONN_MODE_NON as _,
  /// directed-connectable (3.C.9.3.3)
  Dir = BLE_GAP_CONN_MODE_DIR as _,
  /// undirected-connectable (3.C.9.3.4)
  Und = BLE_GAP_CONN_MODE_UND as _,
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum DiscMode {
  /// non-discoverable; 3.C.9.2.2
  Non = BLE_GAP_DISC_MODE_NON as _,
  /// limited-discoverable; 3.C.9.2.3
  Ltd = BLE_GAP_DISC_MODE_LTD as _,
  /// general-discoverable; 3.C.9.2.4
  Gen = BLE_GAP_DISC_MODE_GEN as _,
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

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum AdvType {
  /// indirect advertising
  Ind,
  /// direct advertising
  DirectInd,
  /// indirect scan response
  ScanInd,
  /// indirect advertising - not connectable
  NonconnInd,
  ScanResponse,
  #[cfg(esp_idf_bt_nimble_ext_adv)]
  Extended(u8),
}

impl AdvType {
  pub(crate) fn from_event_type(event_type: u8) -> Self {
    match event_type as u32 {
      BLE_HCI_ADV_RPT_EVTYPE_ADV_IND => AdvType::Ind,
      BLE_HCI_ADV_RPT_EVTYPE_DIR_IND => AdvType::DirectInd,
      BLE_HCI_ADV_RPT_EVTYPE_SCAN_IND => AdvType::ScanInd,
      BLE_HCI_ADV_RPT_EVTYPE_NONCONN_IND => AdvType::NonconnInd,
      BLE_HCI_ADV_RPT_EVTYPE_SCAN_RSP => AdvType::ScanResponse,
      5.. => unreachable!(),
    }
  }
}

bitflags! {
  #[repr(transparent)]
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub struct AdvFlag: u8 {
    /// LE Limited Discoverable Mode
    const DiscLimited = BLE_HS_ADV_F_DISC_LTD as _;
    /// LE General Discoverable Mode
    const DiscGeneral = BLE_HS_ADV_F_DISC_GEN as _;
    /// BR/EDR Not Supported
    const BrEdrUnsupported = BLE_HS_ADV_F_BREDR_UNSUP as _;
    /// Simultaneous LE and BR/EDR to Same Device Capable (Controller)
    const SimultaneousController = 0b01000;
    /// Simultaneous LE and BR/EDR to Same Device Capable (Host)
    const SimultaneousHost       = 0b10000;
  }
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug, IntoPrimitive)]
pub enum ScanFilterPolicy {
  /// Scanner processes all advertising packets (white list not used)
  /// except directed, connectable advertising packets not sent to the scanner.
  NoWl = BLE_HCI_SCAN_FILT_NO_WL as _,
  /// Scanner processes advertisements from white list only.
  /// A connectable, directed advertisement is ignored unless it contains scanners address.
  UseWl = BLE_HCI_SCAN_FILT_USE_WL as _,
  /// Scanner process all advertising packets (white list not used).
  /// A connectable, directed advertisement shall not be ignored if the InitA is a resolvable private address.
  NoWlInitA = BLE_HCI_SCAN_FILT_NO_WL_INITA as _,
  /// Scanner process advertisements from white list only.
  /// A connectable, directed advertisement shall not be ignored if the InitA is a resolvable private address.
  UseWlInitA = BLE_HCI_SCAN_FILT_USE_WL_INITA as _,
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug, IntoPrimitive)]
pub enum AdvFilterPolicy {
  /// No filtering
  None = BLE_HCI_ADV_FILT_NONE as _,
  /// only allow scan requests from those on the white list.
  Scan = BLE_HCI_ADV_FILT_SCAN as _,
  /// only allow connections from those on the white list.
  Connect = BLE_HCI_ADV_FILT_CONN as _,
  /// only allow scan/connections from those on the white list.
  Both = BLE_HCI_ADV_FILT_BOTH as _,
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug, TryFromPrimitive, IntoPrimitive)]
pub enum PrimPhy {
  /// 1Mbps phy
  Phy1M = BLE_HCI_LE_PHY_1M as _,
  /// Coded phy
  Coded = BLE_HCI_LE_PHY_CODED as _,
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug, TryFromPrimitive, IntoPrimitive)]
pub enum SecPhy {
  /// 1Mbps phy
  Phy1M = BLE_HCI_LE_PHY_1M as _,
  /// 2Mbps phy
  Phy2M = BLE_HCI_LE_PHY_2M as _,
  /// Coded phy
  Coded = BLE_HCI_LE_PHY_CODED as _,
}
