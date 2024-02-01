use crate::{enums::*, utilities::BleUuid, BLEDevice};
use alloc::{ffi::CString, vec::Vec};

pub struct BLEAdvertisementData {
  pub(crate) adv_data: esp_idf_sys::ble_hs_adv_fields,
  name: Option<CString>,
  mfg_data: Vec<u8>,
  pub(crate) service_uuids_16: Vec<esp_idf_sys::ble_uuid16_t>,
  pub(crate) service_uuids_32: Vec<esp_idf_sys::ble_uuid32_t>,
  pub(crate) service_uuids_128: Vec<esp_idf_sys::ble_uuid128_t>,
  pub(crate) service_data16: Vec<u8>,
  pub(crate) service_data32: Vec<u8>,
  pub(crate) service_data128: Vec<u8>,
}

impl BLEAdvertisementData {
  pub fn new() -> Self {
    let mut ret = Self {
      adv_data: esp_idf_sys::ble_hs_adv_fields::default(),
      name: None,
      mfg_data: Vec::new(),
      service_uuids_16: Vec::new(),
      service_uuids_32: Vec::new(),
      service_uuids_128: Vec::new(),
      service_data16: Vec::new(),
      service_data32: Vec::new(),
      service_data128: Vec::new(),
    };

    let ble_device = BLEDevice::take();
    ret.adv_data.tx_pwr_lvl = ble_device.get_power(PowerType::Advertising).to_dbm();

    ret.adv_data.flags =
      (esp_idf_sys::BLE_HS_ADV_F_DISC_GEN | esp_idf_sys::BLE_HS_ADV_F_BREDR_UNSUP) as _;

    ret
  }

  /// Set the advertised name of the device.
  pub fn name(&mut self, name: &str) -> &mut Self {
    self.adv_data.name_len = name.len() as _;

    self.name = Some(CString::new(name).unwrap());
    self.adv_data.name = self.name.as_mut().unwrap().as_ptr().cast();
    self.adv_data.set_name_is_complete(1);

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
        self.service_data16.clear();
        self.service_data16.extend_from_slice(&uuid.to_ne_bytes());
        self.service_data16.extend_from_slice(data);
        self.adv_data.svc_data_uuid16 = self.service_data16.as_ptr();
        self.adv_data.svc_data_uuid16_len = if data.is_empty() {
          0
        } else {
          self.service_data16.len() as _
        }
      }
      BleUuid::Uuid32(uuid) => {
        self.service_data32.clear();
        self.service_data32.extend_from_slice(&uuid.to_ne_bytes());
        self.service_data32.extend_from_slice(data);
        self.adv_data.svc_data_uuid32 = self.service_data32.as_ptr();
        self.adv_data.svc_data_uuid32_len = if data.is_empty() {
          0
        } else {
          self.service_data32.len() as _
        }
      }
      BleUuid::Uuid128(uuid) => {
        self.service_data128.clear();
        self.service_data128.extend_from_slice(&uuid);
        self.service_data128.extend_from_slice(data);
        self.adv_data.svc_data_uuid128 = self.service_data128.as_ptr();
        self.adv_data.svc_data_uuid128_len = if data.is_empty() {
          0
        } else {
          self.service_data128.len() as _
        }
      }
    }
  }

  /// Set the device appearance in the advertising data.
  pub fn appearance(&mut self, appearance: u16) -> &mut Self {
    self.adv_data.appearance = appearance;
    self.adv_data.set_appearance_is_present(1);

    self
  }

  /// Add the transmission power level to the advertisement packet.
  pub fn add_tx_power(&mut self) -> &mut Self {
    self.adv_data.set_tx_pwr_lvl_is_present(1);

    self
  }

  pub fn manufacturer_data(&mut self, data: &[u8]) -> &mut Self {
    self.mfg_data.clear();
    self.mfg_data.extend_from_slice(data);
    self.adv_data.mfg_data = self.mfg_data.as_ptr();
    self.adv_data.mfg_data_len = data.len() as _;

    self
  }
}
