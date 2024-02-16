use crate::{enums::PowerType, utilities::BleUuid, BLEDevice};
use alloc::{ffi::CString, vec::Vec};

pub struct BLEAdvertisementData {
  // 0x01 - Flags
  pub(crate) flags: u8,
  // 0x02,0x03 - 16-bit service class UUIDs
  service_uuids_16: Vec<esp_idf_sys::ble_uuid16_t>,
  uuids16_is_complete: bool,
  // 0x04,0x05 - 32-bit service class UUIDs
  service_uuids_32: Vec<esp_idf_sys::ble_uuid32_t>,
  uuids32_is_complete: bool,
  // 0x06,0x07 - 128-bit service class UUIDs.
  service_uuids_128: Vec<esp_idf_sys::ble_uuid128_t>,
  uuids128_is_complete: bool,
  // 0x08,0x09 - Local name
  name: Option<CString>,
  name_is_complete: bool,
  // 0x0a - Tx power level
  tx_pwr_lvl_is_present: bool,
  // Not Implemented: 0x0d - Slave connection interval range

  // 0x16 - Service data - 16-bit UUID
  svc_data_uuid16: Vec<u8>,
  // Not Implemented: 0x17 - Public target address

  // 0x19 - Appearance
  appearance: Option<u16>,
  // Not Implemented: 0x1a - Advertising interval

  // 0x20 - Service data - 32-bit UUID
  svc_data_uuid32: Vec<u8>,
  // 0x21 - Service data - 128-bit UUID
  svc_data_uuid128: Vec<u8>,
  // Not Implemented: 0x24 - URI
  // 0xff - Manufacturer specific data.
  mfg_data: Vec<u8>,
}

impl BLEAdvertisementData {
  pub fn new() -> Self {
    Self {
      flags: (esp_idf_sys::BLE_HS_ADV_F_DISC_GEN | esp_idf_sys::BLE_HS_ADV_F_BREDR_UNSUP) as _,
      service_uuids_16: Vec::new(),
      uuids16_is_complete: true,
      service_uuids_32: Vec::new(),
      uuids32_is_complete: true,
      service_uuids_128: Vec::new(),
      uuids128_is_complete: true,
      name: None,
      name_is_complete: true,
      tx_pwr_lvl_is_present: false,
      svc_data_uuid16: Vec::new(),
      appearance: None,
      svc_data_uuid32: Vec::new(),
      svc_data_uuid128: Vec::new(),
      mfg_data: Vec::new(),
    }
  }

  /// Set the advertised name of the device.
  pub fn name(&mut self, name: &str) -> &mut Self {
    self.name = Some(CString::new(name).unwrap());

    self
  }

  pub fn add_service_uuid(&mut self, uuid: BleUuid) -> &mut Self {
    let x = esp_idf_sys::ble_uuid_any_t::from(uuid);
    match uuid {
      BleUuid::Uuid16(_) => {
        self.service_uuids_16.push(unsafe { x.u16_ });
      }
      BleUuid::Uuid32(_) => {
        self.service_uuids_32.push(unsafe { x.u32_ });
      }
      BleUuid::Uuid128(_) => {
        self.service_uuids_128.push(unsafe { x.u128_ });
      }
    }

    self
  }

  pub fn service_data(&mut self, uuid: BleUuid, data: &[u8]) {
    match uuid {
      BleUuid::Uuid16(uuid) => {
        self.svc_data_uuid16.clear();
        self.svc_data_uuid16.extend_from_slice(&uuid.to_ne_bytes());
        self.svc_data_uuid16.extend_from_slice(data);
      }
      BleUuid::Uuid32(uuid) => {
        self.svc_data_uuid32.clear();
        self.svc_data_uuid32.extend_from_slice(&uuid.to_ne_bytes());
        self.svc_data_uuid32.extend_from_slice(data);
      }
      BleUuid::Uuid128(uuid) => {
        self.svc_data_uuid128.clear();
        self.svc_data_uuid128.extend_from_slice(&uuid);
        self.svc_data_uuid128.extend_from_slice(data);
      }
    }
  }

  /// Set the device appearance in the advertising data.
  pub fn appearance(&mut self, appearance: u16) -> &mut Self {
    self.appearance = Some(appearance);

    self
  }

  /// Add the transmission power level to the advertisement packet.
  pub fn add_tx_power(&mut self) -> &mut Self {
    self.tx_pwr_lvl_is_present = true;

    self
  }

  pub fn manufacturer_data(&mut self, data: &[u8]) -> &mut Self {
    self.mfg_data.clear();
    self.mfg_data.extend_from_slice(data);

    self
  }

  pub(crate) fn payload_len(&self) -> usize {
    let mut payload_len: usize = if self.flags > 0 { 2 + 1 } else { 0 };

    if !self.service_uuids_16.is_empty() {
      payload_len += 2 + 2 * self.service_uuids_16.len();
    }
    if !self.service_uuids_32.is_empty() {
      payload_len += 2 + 4 * self.service_uuids_32.len();
    }
    if !self.service_uuids_128.is_empty() {
      payload_len += 2 + 16 * self.service_uuids_128.len();
    }

    if let Some(name) = &self.name {
      payload_len += 2 + name.to_bytes().len();
    }

    if self.tx_pwr_lvl_is_present {
      payload_len += 2 + (esp_idf_sys::BLE_HS_ADV_TX_PWR_LVL_LEN as usize);
    }

    if !self.svc_data_uuid16.is_empty() {
      payload_len += 2 + self.svc_data_uuid16.len();
    }

    if !self.svc_data_uuid32.is_empty() {
      payload_len += 2 + self.svc_data_uuid32.len();
    }

    if !self.svc_data_uuid128.is_empty() {
      payload_len += 2 + self.svc_data_uuid128.len();
    }

    if self.appearance.is_some() {
      payload_len += 2 + (esp_idf_sys::BLE_HS_ADV_APPEARANCE_LEN as usize);
    }

    if !self.mfg_data.is_empty() {
      payload_len += 2 + self.mfg_data.len();
    }

    // if self.uri_len > 0 {
    //   payload_len += 2 + adv_data.uri_len;
    // }

    // if !adv_data.slave_itvl_range.is_null() {
    //   payload_len += 2 + (esp_idf_sys::BLE_HS_ADV_SLAVE_ITVL_RANGE_LEN as _);
    // }

    payload_len
  }

  pub(crate) fn as_ble_hs_adv_fields(&self) -> esp_idf_sys::ble_hs_adv_fields {
    let mut ret = esp_idf_sys::ble_hs_adv_fields {
      flags: self.flags,
      ..Default::default()
    };

    if !self.service_uuids_16.is_empty() {
      ret.set_uuids16_is_complete(self.uuids16_is_complete as _);
      ret.uuids16 = self.service_uuids_16.as_ptr();
      ret.num_uuids16 = self.service_uuids_16.len() as _;
    }
    if !self.service_uuids_32.is_empty() {
      ret.set_uuids32_is_complete(self.uuids32_is_complete as _);
      ret.uuids32 = self.service_uuids_32.as_ptr();
      ret.num_uuids32 = self.service_uuids_32.len() as _;
    }
    if !self.service_uuids_128.is_empty() {
      ret.set_uuids128_is_complete(self.uuids128_is_complete as _);
      ret.uuids128 = self.service_uuids_128.as_ptr();
      ret.num_uuids128 = self.service_uuids_128.len() as _;
    }

    if let Some(name) = &self.name {
      ret.name = name.as_ptr().cast();
      ret.name_len = name.to_bytes().len() as _;
      ret.set_name_is_complete(self.name_is_complete as _);
    }

    if self.tx_pwr_lvl_is_present {
      ret.set_tx_pwr_lvl_is_present(1);
      let ble_device = BLEDevice::take();
      ret.tx_pwr_lvl = ble_device.get_power(PowerType::Advertising).to_dbm();
    }

    if !self.svc_data_uuid16.is_empty() {
      ret.svc_data_uuid16 = self.svc_data_uuid16.as_ptr();
      ret.svc_data_uuid16_len = self.svc_data_uuid16.len() as _;
    }

    if !self.svc_data_uuid32.is_empty() {
      ret.svc_data_uuid32 = self.svc_data_uuid32.as_ptr();
      ret.svc_data_uuid32_len = self.svc_data_uuid32.len() as _;
    }

    if !self.svc_data_uuid128.is_empty() {
      ret.svc_data_uuid128 = self.svc_data_uuid128.as_ptr();
      ret.svc_data_uuid128_len = self.svc_data_uuid128.len() as _;
    }

    if let Some(appearance) = self.appearance {
      ret.set_appearance_is_present(1);
      ret.appearance = appearance;
    }

    if !self.mfg_data.is_empty() {
      ret.mfg_data = self.mfg_data.as_ptr();
      ret.mfg_data_len = self.mfg_data.len() as _;
    }

    ret
  }
}
