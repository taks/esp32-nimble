use core::ffi::c_void;
use once_cell::sync::Lazy;

use alloc::vec;
use alloc::vec::Vec;

use crate::{
  ble,
  utilities::{os_mbuf_append, os_msys_get_pkthdr, BleUuid},
  BLEAddress, BLEReturnCode, BLEServer,
};

pub struct BLEExtAdvertisement {
  payload: Vec<u8>,
  params: esp_idf_sys::ble_gap_ext_adv_params,
  adv_address: Option<BLEAddress>,
}

impl BLEExtAdvertisement {
  pub fn new(primary_phy: u8, secondary_phy: u8) -> Self {
    Self {
      payload: Vec::new(),
      params: esp_idf_sys::ble_gap_ext_adv_params {
        own_addr_type: unsafe { crate::ble_device::OWN_ADDR_TYPE as _ },
        primary_phy,
        secondary_phy,
        tx_power: 127,
        ..Default::default()
      },
      adv_address: None,
    }
  }

  pub fn legacy_advertising(&mut self, val: bool) {
    self.params.set_legacy_pdu(val as _);
  }

  pub fn scannable(&mut self, val: bool) {
    self.params.set_scannable(val as _);
  }

  /// Sets the transmission power level for this advertisement.
  ///
  /// The allowable value range depends on device hardware.
  /// The ESP32C3 and ESP32S3 have a range of -27 to +18
  ///
  /// * `dbm`: the transmission power to use in dbm.
  pub fn tx_power(&mut self, dbm: i8) {
    self.params.tx_power = dbm;
  }

  /// Sets wether this advertisement should advertise as a connectable device.
  pub fn connectable(&mut self, val: bool) {
    self.params.set_connectable(val as _);
  }

  /// Set the address to use for this advertisement
  pub fn address(&mut self, addr: &BLEAddress) {
    self.adv_address = Some(*addr);
    self.params.own_addr_type = esp_idf_sys::BLE_OWN_ADDR_RANDOM as _;
  }

  /// Sets The primary channels to advertise on.
  pub fn primary_channels(&mut self, ch37: bool, ch38: bool, ch39: bool) {
    self.params.channel_map = (ch37 as u8) | ((ch38 as u8) << 1) | ((ch39 as u8) << 2);
  }

  /// Set the minimum advertising interval
  pub fn min_interval(&mut self, val: u32) {
    self.params.itvl_min = val;
  }

  /// Set the maximum advertising interval.
  pub fn max_interval(&mut self, val: u32) {
    self.params.itvl_max = val;
  }

  /// Sets whether the scan response request callback should be called.
  pub fn enable_scan_request_callback(&mut self, val: bool) {
    self.params.set_scan_req_notif(val as _);
  }

  /// Clears the data stored in this instance, does not change settings.
  pub fn clear(&mut self) {
    self.payload.clear();
  }

  /// Get the size of the current data.
  pub fn size(&self) -> usize {
    self.payload.len()
  }

  pub fn appearance(&mut self, appearance: u16) {
    self.add_data(
      esp_idf_sys::BLE_HS_ADV_TYPE_APPEARANCE as _,
      &appearance.to_le_bytes(),
    );
  }

  /// Set manufacturer specific data.
  pub fn manufacturer_data(&mut self, data: &[u8]) {
    self.add_data(esp_idf_sys::BLE_HS_ADV_TYPE_MFG_DATA as _, data);
  }

  /// Set the complete name of this device.
  pub fn name(&mut self, name: &str) {
    self.add_data(esp_idf_sys::BLE_HS_ADV_TYPE_COMP_NAME as _, name.as_bytes());
  }

  // Set a single service to advertise as a complete list of services.
  pub fn complete_service(&mut self, uuid: &BleUuid) {
    let bit_size = match uuid {
      BleUuid::Uuid16(_) => 16,
      BleUuid::Uuid32(_) => 32,
      BleUuid::Uuid128(_) => 128,
    };
    self.set_services(true, bit_size, &[*uuid]);
  }

  fn set_services(&mut self, complete: bool, size: u8, uuids: &[BleUuid]) {
    let data_type: u8 = match size {
      16 => {
        if complete {
          esp_idf_sys::BLE_HS_ADV_TYPE_SVC_DATA_UUID16 as _
        } else {
          esp_idf_sys::BLE_HS_ADV_TYPE_INCOMP_UUIDS16 as _
        }
      }
      32 => {
        if complete {
          esp_idf_sys::BLE_HS_ADV_TYPE_SVC_DATA_UUID32 as _
        } else {
          esp_idf_sys::BLE_HS_ADV_TYPE_INCOMP_UUIDS32 as _
        }
      }
      128 => {
        if complete {
          esp_idf_sys::BLE_HS_ADV_TYPE_SVC_DATA_UUID128 as _
        } else {
          esp_idf_sys::BLE_HS_ADV_TYPE_INCOMP_UUIDS128 as _
        }
      }
      _ => return,
    };
    self.payload.push((size / 8) * (uuids.len() as u8) + 1);
    self.payload.push(data_type);

    for uuid in uuids {
      match uuid {
        BleUuid::Uuid16(uuid) => {
          self.payload.extend_from_slice(&uuid.to_ne_bytes());
        }
        BleUuid::Uuid32(uuid) => {
          self.payload.extend_from_slice(&uuid.to_ne_bytes());
        }
        BleUuid::Uuid128(uuid) => {
          self.payload.extend_from_slice(uuid);
        }
      }
    }
  }

  /// Set the service data (UUID + data)
  pub fn service_data(&mut self, uuid: BleUuid, data: &[u8]) {
    match uuid {
      BleUuid::Uuid16(uuid) => {
        self.add_data2(
          esp_idf_sys::BLE_HS_ADV_TYPE_SVC_DATA_UUID16 as _,
          &uuid.to_ne_bytes(),
          data,
        );
      }
      BleUuid::Uuid32(uuid) => {
        self.add_data2(
          esp_idf_sys::BLE_HS_ADV_TYPE_SVC_DATA_UUID32 as _,
          &uuid.to_ne_bytes(),
          data,
        );
      }
      BleUuid::Uuid128(uuid) => {
        self.add_data2(
          esp_idf_sys::BLE_HS_ADV_TYPE_SVC_DATA_UUID128 as _,
          &uuid,
          data,
        );
      }
    }
  }

  fn add_data(&mut self, data_type: u8, data: &[u8]) {
    self.payload.push((data.len() as u8) + 1);
    self.payload.push(data_type);
    self.payload.extend_from_slice(data);
  }

  fn add_data2(&mut self, data_type: u8, data0: &[u8], data1: &[u8]) {
    self.payload.push((data0.len() + data1.len() + 1) as u8);
    self.payload.push(data_type);
    self.payload.extend_from_slice(data0);
    self.payload.extend_from_slice(data1);
  }
}

pub struct BLEExtAdvertising {
  adv_status: Vec<bool>,
}

impl BLEExtAdvertising {
  #[allow(dead_code)]
  pub(crate) fn new() -> Self {
    Self {
      adv_status: vec![false; (esp_idf_sys::CONFIG_BT_NIMBLE_MAX_EXT_ADV_INSTANCES + 1) as _],
    }
  }

  pub fn set_instance_data(
    &mut self,
    inst_id: u8,
    adv: &mut BLEExtAdvertisement,
  ) -> Result<(), BLEReturnCode> {
    adv.params.sid = inst_id;

    // Legacy advertising as connectable requires the scannable flag also.
    if adv.params.legacy_pdu() != 0 && adv.params.connectable() != 0 {
      adv.params.set_scannable(1);
    }

    // If connectable or not scannable disable the callback for scan response requests
    if adv.params.connectable() != 0 || adv.params.scannable() == 0 {
      adv.params.set_scan_req_notif(0);
    }

    let mut server = unsafe { Lazy::get_mut(&mut crate::ble_device::BLE_SERVER) };
    if let Some(server) = server.as_mut() {
      if !server.started {
        server.start()?;
      }
    }

    let handle_gap_event = if server.is_some() {
      BLEServer::handle_gap_event
    } else {
      Self::handle_gap_event
    };

    unsafe {
      ble!(esp_idf_sys::ble_gap_ext_adv_configure(
        inst_id,
        &adv.params,
        core::ptr::null_mut(),
        Some(handle_gap_event),
        self as *mut Self as _
      ))?;

      let buf = os_msys_get_pkthdr(adv.payload.len() as _, 0);
      if buf.is_null() {
        return BLEReturnCode::fail();
      }
      ble!(os_mbuf_append(buf, &adv.payload))?;

      if (adv.params.scannable() != 0) && (adv.params.legacy_pdu() == 0) {
        ble!(esp_idf_sys::ble_gap_ext_adv_rsp_set_data(inst_id, buf))?;
      } else {
        ble!(esp_idf_sys::ble_gap_ext_adv_set_data(inst_id, buf))?;
      }

      if let Some(addr) = adv.adv_address {
        ble!(esp_idf_sys::ble_gap_ext_adv_set_addr(inst_id, &addr.value))?;
      }
    }

    Ok(())
  }

  pub fn set_scan_response_data(
    &mut self,
    inst_id: u8,
    lsr: &BLEExtAdvertisement,
  ) -> Result<(), BLEReturnCode> {
    unsafe {
      let buf = os_msys_get_pkthdr(lsr.payload.len() as _, 0);
      if buf.is_null() {
        return BLEReturnCode::fail();
      }
      ble!(os_mbuf_append(buf, &lsr.payload))?;

      ble!(esp_idf_sys::ble_gap_ext_adv_rsp_set_data(inst_id, buf))
    }
  }

  pub fn start(&mut self, inst_id: u8) -> Result<(), BLEReturnCode> {
    self.start_with_duration(inst_id, 0, 0)
  }

  pub fn start_with_duration(
    &mut self,
    inst_id: u8,
    duration: i32,
    max_event: i32,
  ) -> Result<(), BLEReturnCode> {
    unsafe {
      ble!(esp_idf_sys::ble_gap_ext_adv_start(
        inst_id, duration, max_event
      ))
    }
  }

  pub(crate) extern "C" fn handle_gap_event(
    event: *mut esp_idf_sys::ble_gap_event,
    arg: *mut c_void,
  ) -> i32 {
    let event = unsafe { &*event };
    let adv = unsafe { &mut *(arg as *mut Self) };

    match event.type_ as _ {
      esp_idf_sys::BLE_GAP_EVENT_ADV_COMPLETE => {
        let adv_complete = unsafe { event.__bindgen_anon_1.adv_complete };
        adv.adv_status[adv_complete.instance as usize] = false;
      }
      _ => {}
    }

    0
  }
}
