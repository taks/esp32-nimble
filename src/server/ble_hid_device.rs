use alloc::sync::Arc;

use crate::{
  utilities::{mutex::Mutex, BleUuid},
  BLECharacteristic, BLEServer, BLEService, NimbleProperties,
};

#[allow(dead_code)]
pub struct BLEHIDDevice {
  device_info_service: Arc<Mutex<BLEService>>,
  pnp_characteristic: Arc<Mutex<BLECharacteristic>>,
  hid_service: Arc<Mutex<BLEService>>,
  hid_info_characteristic: Arc<Mutex<BLECharacteristic>>,
  report_map_characteristic: Arc<Mutex<BLECharacteristic>>,
  hid_control_characteristic: Arc<Mutex<BLECharacteristic>>,
  protocol_mode_characteristic: Arc<Mutex<BLECharacteristic>>,
}

impl BLEHIDDevice {
  pub fn new(server: &mut BLEServer) -> Self {
    let device_info_service = server.create_service(BleUuid::from_uuid16(0x180a));

    let pnp_characteristic = device_info_service
      .lock()
      .create_characteristic(BleUuid::from_uuid16(0x2a50), NimbleProperties::READ);

    let hid_service = server.create_service(BleUuid::from_uuid16(0x1812));

    let hid_info_characteristic = hid_service
      .lock()
      .create_characteristic(BleUuid::Uuid16(0x2a4a), NimbleProperties::READ);
    let report_map_characteristic = hid_service
      .lock()
      .create_characteristic(BleUuid::Uuid16(0x2a4b), NimbleProperties::READ);
    let hid_control_characteristic = hid_service
      .lock()
      .create_characteristic(BleUuid::Uuid16(0x2a4c), NimbleProperties::WRITE_NO_RSP);
    let protocol_mode_characteristic = hid_service.lock().create_characteristic(
      BleUuid::Uuid16(0x2a4e),
      NimbleProperties::WRITE_NO_RSP | NimbleProperties::READ,
    );

    Self {
      device_info_service,
      pnp_characteristic,
      hid_service,
      hid_info_characteristic,
      report_map_characteristic,
      hid_control_characteristic,
      protocol_mode_characteristic,
    }
  }

  /// Sets the Plug n Play characteristic value.
  pub fn report_map(&mut self, map: &[u8]) {
    self.report_map_characteristic.lock().set_value(map);
  }

  pub fn pnp(&mut self, sig: u8, vid: u16, pid: u16, version: u16) {
    let mut pnp_characteristic = self.pnp_characteristic.lock();
    let value = pnp_characteristic.value_mut();
    value.clear();
    value.extend(&[sig]);
    value.extend(&vid.to_be_bytes());
    value.extend(&pid.to_be_bytes());
    value.extend(&version.to_be_bytes());
  }

  /// Sets the HID Information characteristic value.1
  pub fn hid_info(&mut self, country: u8, flags: u8) {
    let info = [0x11, 0x1, country, flags];
    self.hid_info_characteristic.lock().set_value(&info);
  }

  /// Create input report characteristic
  pub fn input_report(&mut self, report_id: u8) -> Arc<Mutex<BLECharacteristic>> {
    let input_report_characteristic = self.hid_service.lock().create_characteristic(
      BleUuid::from_uuid16(0x2a4d),
      NimbleProperties::READ | NimbleProperties::NOTIFY | NimbleProperties::READ_ENC,
    );
    let input_report_descriptor = input_report_characteristic.lock().create_descriptor(
      BleUuid::Uuid16(0x2908),
      NimbleProperties::READ | NimbleProperties::READ_ENC,
    );

    let desc1_val = [report_id, 0x01];
    input_report_descriptor.lock().set_value(&desc1_val);

    input_report_characteristic
  }

  pub fn output_report(&mut self, report_id: u8) -> Arc<Mutex<BLECharacteristic>> {
    let output_report_characteristic = self.hid_service.lock().create_characteristic(
      BleUuid::from_uuid16(0x2a4d),
      NimbleProperties::READ
        | NimbleProperties::WRITE
        | NimbleProperties::WRITE_NO_RSP
        | NimbleProperties::READ_ENC
        | NimbleProperties::WRITE_ENC,
    );
    let output_report_descriptor = output_report_characteristic.lock().create_descriptor(
      BleUuid::Uuid16(0x2908),
      NimbleProperties::READ
        | NimbleProperties::WRITE
        | NimbleProperties::READ_ENC
        | NimbleProperties::WRITE_ENC,
    );

    let desc1_val = [report_id, 0x02];
    output_report_descriptor.lock().set_value(&desc1_val);

    output_report_characteristic
  }

  pub fn feature_report(&mut self, report_id: u8) -> Arc<Mutex<BLECharacteristic>> {
    let feature_report_characteristic = self.hid_service.lock().create_characteristic(
      BleUuid::from_uuid16(0x2a4d),
      NimbleProperties::READ
        | NimbleProperties::WRITE
        | NimbleProperties::READ_ENC
        | NimbleProperties::WRITE_ENC,
    );
    let feature_report_descriptor = feature_report_characteristic.lock().create_descriptor(
      BleUuid::Uuid16(0x2908),
      NimbleProperties::READ
        | NimbleProperties::WRITE
        | NimbleProperties::READ_ENC
        | NimbleProperties::WRITE_ENC,
    );

    let desc1_val = [report_id, 0x03];
    feature_report_descriptor.lock().set_value(&desc1_val);

    feature_report_characteristic
  }

  /// Creates a keyboard boot input report characteristic
  pub fn boot_input(&self) -> Arc<Mutex<BLECharacteristic>> {
    self
      .hid_service
      .lock()
      .create_characteristic(BleUuid::from_uuid16(0x2a22), NimbleProperties::NOTIFY)
  }

  /// Creates a keyboard boot input report characteristic
  pub fn boot_output(&self) -> Arc<Mutex<BLECharacteristic>> {
    self.hid_service.lock().create_characteristic(
      BleUuid::from_uuid16(0x2a32),
      NimbleProperties::READ | NimbleProperties::WRITE | NimbleProperties::WRITE_NO_RSP,
    )
  }

  /// Returns the HID control point characteristic.
  pub fn hid_control(&self) -> &Arc<Mutex<BLECharacteristic>> {
    &self.hid_control_characteristic
  }

  /// Returns the protocol mode characteristic.
  pub fn protocol_mode(&self) -> &Arc<Mutex<BLECharacteristic>> {
    &self.protocol_mode_characteristic
  }

  /// Returns a pointer to the HID service.
  pub fn hid_service(&self) -> &Arc<Mutex<BLEService>> {
    &self.hid_service
  }
}
