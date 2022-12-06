use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use crate::utilities::BleUuid;
use crate::BLEAddress;

#[derive(Debug, Clone)]
pub struct BLEAdvertisedDevice {
  addr: BLEAddress,
  adv_type: u8,
  ad_flag: Option<u8>,
  appearance: Option<u16>,
  name: String,
  rssi: i32,
  service_uuids: Vec<BleUuid>,
  service_data: Vec<(BleUuid, Box<[u8]>)>,
  tx_power: Option<u8>,
  manufacture_data: Option<Vec<u8>>,
}

impl BLEAdvertisedDevice {
  pub(crate) fn new(param: &esp_idf_sys::ble_gap_disc_desc) -> Self {
    Self {
      addr: param.addr,
      adv_type: param.event_type,
      ad_flag: None,
      appearance: None,
      name: String::new(),
      rssi: param.rssi as _,
      service_uuids: Vec::new(),
      service_data: Vec::new(),
      tx_power: None,
      manufacture_data: None,
    }
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn addr(&self) -> &BLEAddress {
    &self.addr
  }

  pub fn rssi(&self) -> i32 {
    self.rssi
  }

  pub(crate) fn adv_type(&self) -> u8 {
    self.adv_type
  }

  pub(crate) fn parse_advertisement(&mut self, payload: &[u8]) {
    let mut payload = payload;

    loop {
      if payload.is_empty() {
        return;
      }

      let length = payload[0] as usize;
      if length != 0 {
        let type_ = payload[1] as u32;
        let data = &payload[2..(length + 1)];

        match type_ {
          esp_idf_sys::BLE_HS_ADV_TYPE_FLAGS => {
            self.ad_flag = Some(data[0]);
          }
          esp_idf_sys::BLE_HS_ADV_TYPE_INCOMP_NAME | esp_idf_sys::BLE_HS_ADV_TYPE_COMP_NAME => {
            self.name = String::from_utf8(data.to_vec()).unwrap();
          }
          esp_idf_sys::BLE_HS_ADV_TYPE_TX_PWR_LVL => {
            self.tx_power = Some(data[0]);
          }

          esp_idf_sys::BLE_HS_ADV_TYPE_INCOMP_UUIDS16
          | esp_idf_sys::BLE_HS_ADV_TYPE_COMP_UUIDS16 => {
            let mut data = data;
            while !data.is_empty() {
              let (uuid, data_) = data.split_at(2);
              self
                .service_uuids
                .push(BleUuid::from_uuid16(u16::from_le_bytes(
                  uuid.try_into().unwrap(),
                )));
              data = data_;
            }
          }
          esp_idf_sys::BLE_HS_ADV_TYPE_INCOMP_UUIDS128
          | esp_idf_sys::BLE_HS_ADV_TYPE_COMP_UUIDS128 => {
            self
              .service_uuids
              .push(BleUuid::Uuid128(data.try_into().unwrap()));
          }

          esp_idf_sys::BLE_HS_ADV_TYPE_SVC_DATA_UUID16 => {
            // Adv Data Type: 0x16 (Service Data) - 2 byte UUID
            if length < 2 {
              ::log::error!("Length too small for BLE_HS_ADV_TYPE_SVC_DATA_UUID16");
            } else {
              let (uuid, service_data) = data.split_at(2);
              let uuid = BleUuid::from_uuid16(u16::from_le_bytes(uuid.try_into().unwrap()));
              self.service_data.push((uuid, service_data.into()));
            }
          }
          esp_idf_sys::BLE_HS_ADV_TYPE_APPEARANCE => {
            self.appearance = Some(u16::from_le_bytes(data.try_into().unwrap()));
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
}
