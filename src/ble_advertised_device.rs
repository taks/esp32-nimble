use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use crate::utilities::BleUuid;
use crate::BLEAddress;

#[derive(Debug, Clone)]
pub struct BLEAdvertisedDevice {
  pub addr: BLEAddress,
  pub ad_flag: Option<u8>,
  pub name: String,
  pub rssi: i32,
  pub service_uuids: Vec<BleUuid>,
  pub service_data: Vec<(BleUuid, Box<[u8]>)>,
  pub tx_power: Option<u8>,
  pub manufacture_data: Option<Vec<u8>>,
}

impl BLEAdvertisedDevice {
  pub(crate) fn new(param: &esp_idf_sys::ble_gap_disc_desc) -> Self {
    let mut ret = Self {
      addr: param.addr,
      ad_flag: None,
      name: String::new(),
      rssi: param.rssi as _,
      service_uuids: Vec::new(),
      service_data: Vec::new(),
      tx_power: None,
      manufacture_data: None,
    };

    let data = unsafe { core::slice::from_raw_parts(param.data, param.length_data as _) };
    ::log::debug!("DATA: {:X?}", data);
    ret.parse_advertisement(data);

    ret
  }

  fn parse_advertisement(&mut self, payload: &[u8]) {
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
          esp_idf_sys::BLE_HS_ADV_TYPE_MFG_DATA => {
            self.manufacture_data = Some(data.to_vec());
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
