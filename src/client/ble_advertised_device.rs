use alloc::boxed::Box;
use alloc::vec::Vec;
use bstr::{BStr, BString};

use crate::enums::{AdvFlag, AdvType};
use crate::utilities::BleUuid;
use crate::BLEAddress;

#[derive(Debug, Clone)]
pub struct BLEServiceData {
  uuid: BleUuid,
  data: Box<[u8]>,
}

impl BLEServiceData {
  pub fn uuid(&self) -> BleUuid {
    self.uuid
  }
  pub fn data(&self) -> &[u8] {
    &self.data
  }
}

#[derive(Debug, Clone)]
pub struct BLEAdvertisedDevice {
  addr: BLEAddress,
  adv_type: AdvType,
  adv_flags: Option<AdvFlag>,
  appearance: Option<u16>,
  name: BString,
  rssi: i32,
  service_uuids: Vec<BleUuid>,
  service_data_list: Vec<BLEServiceData>,
  tx_power: Option<u8>,
  manufacture_data: Option<Vec<u8>>,
}

impl BLEAdvertisedDevice {
  pub(crate) fn new(param: &esp_idf_sys::ble_gap_disc_desc) -> Self {
    Self {
      addr: param.addr.into(),
      adv_type: AdvType::try_from(param.event_type).unwrap(),
      adv_flags: None,
      appearance: None,
      name: BString::default(),
      rssi: param.rssi as _,
      service_uuids: Vec::new(),
      service_data_list: Vec::new(),
      tx_power: None,
      manufacture_data: None,
    }
  }

  pub fn name(&self) -> &BStr {
    self.name.as_ref()
  }

  /// Get the address of the advertising device.
  pub fn addr(&self) -> &BLEAddress {
    &self.addr
  }

  /// Get the advertisement type.
  pub fn adv_type(&self) -> AdvType {
    self.adv_type
  }

  /// Get the advertisement flags.
  pub fn adv_flags(&self) -> Option<AdvFlag> {
    self.adv_flags
  }

  pub fn rssi(&self) -> i32 {
    self.rssi
  }

  pub(crate) fn update_rssi(&mut self, rssi: i8) {
    self.rssi = rssi as i32;
  }

  pub fn get_service_uuids(&self) -> core::slice::Iter<'_, BleUuid> {
    self.service_uuids.iter()
  }

  pub fn is_advertising_service(&self, uuid: &BleUuid) -> bool {
    self.get_service_uuids().any(|x| x == uuid)
  }

  pub fn get_service_data_list(&self) -> core::slice::Iter<'_, BLEServiceData> {
    self.service_data_list.iter()
  }

  pub fn get_service_data(&self, uuid: BleUuid) -> Option<&BLEServiceData> {
    self.get_service_data_list().find(|x| x.uuid == uuid)
  }

  pub fn get_manufacture_data(&self) -> Option<&[u8]> {
    self.manufacture_data.as_deref()
  }

  pub(crate) fn parse_advertisement(&mut self, payload: &[u8]) {
    let mut payload = payload;

    loop {
      let Some(length) = payload.first() else {
        return;
      };
      let length = *length as usize;

      if length != 0 {
        let Some(type_) = payload.get(1) else { return };
        let type_ = *type_ as u32;

        let Some(data) = payload.get(2..(length + 1)) else {
          return;
        };

        match type_ {
          esp_idf_sys::BLE_HS_ADV_TYPE_FLAGS => {
            let Some(ad_flag) = data.first() else { return };
            self.adv_flags = AdvFlag::from_bits(*ad_flag);
          }
          esp_idf_sys::BLE_HS_ADV_TYPE_INCOMP_NAME | esp_idf_sys::BLE_HS_ADV_TYPE_COMP_NAME => {
            self.name = BString::new(data.to_vec());
          }
          esp_idf_sys::BLE_HS_ADV_TYPE_TX_PWR_LVL => {
            let Some(tx_power) = data.first() else { return };
            self.tx_power = Some(*tx_power);
          }

          esp_idf_sys::BLE_HS_ADV_TYPE_INCOMP_UUIDS16
          | esp_idf_sys::BLE_HS_ADV_TYPE_COMP_UUIDS16 => {
            let mut data = data;
            while data.len() >= 2 {
              let (uuid, data_) = data.split_at(2);
              self.push_service_uuid(BleUuid::from_uuid16(u16::from_le_bytes(
                uuid.try_into().unwrap(),
              )));
              data = data_;
            }
          }
          esp_idf_sys::BLE_HS_ADV_TYPE_INCOMP_UUIDS32
          | esp_idf_sys::BLE_HS_ADV_TYPE_COMP_UUIDS32 => {
            let mut data = data;
            while data.len() >= 4 {
              let (uuid, data_) = data.split_at(4);
              self.push_service_uuid(BleUuid::from_uuid32(u32::from_le_bytes(
                uuid.try_into().unwrap(),
              )));
              data = data_;
            }
          }
          esp_idf_sys::BLE_HS_ADV_TYPE_INCOMP_UUIDS128
          | esp_idf_sys::BLE_HS_ADV_TYPE_COMP_UUIDS128 => {
            if let Ok(data) = data.try_into() {
              self.push_service_uuid(BleUuid::Uuid128(data));
            }
          }
          esp_idf_sys::BLE_HS_ADV_TYPE_SVC_DATA_UUID16 => {
            if length < 2 {
              ::log::error!("Length too small for BLE_HS_ADV_TYPE_SVC_DATA_UUID16");
            } else {
              let (uuid, service_data) = data.split_at(2);
              let uuid = BleUuid::from_uuid16(u16::from_le_bytes(uuid.try_into().unwrap()));
              self.push_service_data(uuid, service_data);
            }
          }
          esp_idf_sys::BLE_HS_ADV_TYPE_SVC_DATA_UUID32 => {
            if length < 4 {
              ::log::error!("Length too small for BLE_HS_ADV_TYPE_SVC_DATA_UUID32");
            } else {
              let (uuid, service_data) = data.split_at(4);
              let uuid = BleUuid::from_uuid32(u32::from_le_bytes(uuid.try_into().unwrap()));
              self.push_service_data(uuid, service_data);
            }
          }
          esp_idf_sys::BLE_HS_ADV_TYPE_SVC_DATA_UUID128 => {
            if length < 16 {
              ::log::error!("Length too small for BLE_HS_ADV_TYPE_SVC_DATA_UUID128");
            } else {
              let (uuid, service_data) = data.split_at(16);
              let uuid = BleUuid::from_uuid128(uuid.try_into().unwrap());
              self.push_service_data(uuid, service_data);
            }
          }
          esp_idf_sys::BLE_HS_ADV_TYPE_APPEARANCE => {
            if let Ok(appearance) = data.try_into() {
              self.appearance = Some(u16::from_le_bytes(appearance));
            }
          }
          esp_idf_sys::BLE_HS_ADV_TYPE_MFG_DATA => {
            self.manufacture_data = Some(data.to_vec());
          }
          esp_idf_sys::BLE_HS_ADV_TYPE_SLAVE_ITVL_RANGE => {
            // DO NOTHING
          }
          _ => {
            ::log::info!("Unhandled type: adType: 0x{:X}", type_);
          }
        }
      }
      payload = &payload[(1 + length)..];
    }
  }

  fn push_service_uuid(&mut self, uuid: BleUuid) {
    let is_present = self.service_uuids.iter().any(|x| x == &uuid);
    if !is_present {
      self.service_uuids.push(uuid);
    }
  }

  fn push_service_data(&mut self, uuid: BleUuid, data: &[u8]) {
    let service_data = self.service_data_list.iter_mut().find(|x| x.uuid == uuid);
    match service_data {
      Some(x) => {
        x.data = data.into();
      }
      None => self.service_data_list.push(BLEServiceData {
        uuid,
        data: data.into(),
      }),
    }
  }
}
