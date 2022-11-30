use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use esp_idf_sys::*;
use log::*;

use crate::utilities::BleUuid;
use crate::BLEAddress;

#[derive(Debug, Clone)]
pub struct BLEAdvertisedDevice {
  pub addr: BLEAddress,
  pub addr_type: u32,
  pub ad_flag: Option<u8>,
  pub name: String,
  pub rssi: i32,
  pub service_uuids: Vec<BleUuid>,
  pub service_data: Vec<(BleUuid, Box<[u8]>)>,
  pub tx_power: Option<u8>,
}

impl BLEAdvertisedDevice {
  pub(crate) fn new(param: &esp_ble_gap_cb_param_t_ble_scan_result_evt_param) -> Self {
    let mut ret = Self {
      addr: param.bda,
      addr_type: param.ble_addr_type,
      ad_flag: None,
      name: String::new(),
      rssi: param.rssi,
      service_uuids: Vec::new(),
      service_data: Vec::new(),
      tx_power: None,
    };
    ret.parse_advertisement(&param.ble_adv[..(param.adv_data_len + param.scan_rsp_len) as usize]);

    ret
  }

  fn parse_advertisement(&mut self, payload: &[u8]) {
    let mut payload = payload;

    loop {
      let length = payload[0] as usize;
      if length != 0 {
        let data = &payload[2..(length + 1)];

        match payload[1] as u32 {
          esp_idf_sys::esp_ble_adv_data_type_ESP_BLE_AD_TYPE_NAME_CMPL => {
            self.name = String::from_utf8(data.to_vec()).unwrap();
          }
          esp_idf_sys::esp_ble_adv_data_type_ESP_BLE_AD_TYPE_TX_PWR => {
            self.tx_power = Some(data[0]);
          }
          esp_idf_sys::esp_ble_adv_data_type_ESP_BLE_AD_TYPE_FLAG => {
            self.ad_flag = Some(data[0]);
          }
          esp_idf_sys::esp_ble_adv_data_type_ESP_BLE_AD_TYPE_16SRV_CMPL
          | esp_idf_sys::esp_ble_adv_data_type_ESP_BLE_AD_TYPE_16SRV_PART => {
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
          esp_idf_sys::esp_ble_adv_data_type_ESP_BLE_AD_MANUFACTURER_SPECIFIC_TYPE => {
            // TODO:
          }
          esp_idf_sys::esp_ble_adv_data_type_ESP_BLE_AD_TYPE_SERVICE_DATA => {
            // Adv Data Type: 0x16 (Service Data) - 2 byte UUID
            if length < 2 {
              error!("Length too small for ESP_BLE_AD_TYPE_SERVICE_DATA");
            }
            let (uuid, service_data) = data.split_at(2);
            let uuid = BleUuid::from_uuid16(u16::from_le_bytes(uuid.try_into().unwrap()));
            self.service_data.push((uuid, service_data.into()));
          }
          _ => {
            debug!("Unhandled type: adType: {}", payload[1]);
          }
        }
      }
      payload = &payload[(1 + length)..];

      if payload.is_empty() {
        return;
      }
    }
  }
}
