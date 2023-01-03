use alloc::sync::Arc;

use crate::{
  utilities::{mutex::Mutex, BleUuid},
  BLE2904Format, BLECharacteristic, BLEServer, BLEService, DescriptorProperties, NimbleProperties,
  BLE2904,
};

const BLE_SVC_DIS_CHR_UUID16_MANUFACTURER_NAME: BleUuid = BleUuid::from_uuid16(0x2A29);
const BLE_SVC_BAS_UUID16: BleUuid = BleUuid::from_uuid16(0x180F);
const BLE_SVC_BAS_CHR_UUID16_BATTERY_LEVEL: BleUuid = BleUuid::from_uuid16(0x2A19);

#[allow(dead_code)]
pub struct BLEHIDDevice {
  device_info_service: Arc<Mutex<BLEService>>,
  pnp_characteristic: Arc<Mutex<BLECharacteristic>>,
  manufacturer_characteristic: Option<Arc<Mutex<BLECharacteristic>>>,

  hid_service: Arc<Mutex<BLEService>>,
  hid_info_characteristic: Arc<Mutex<BLECharacteristic>>,
  report_map_characteristic: Arc<Mutex<BLECharacteristic>>,
  hid_control_characteristic: Arc<Mutex<BLECharacteristic>>,
  protocol_mode_characteristic: Arc<Mutex<BLECharacteristic>>,

  battery_service: Arc<Mutex<BLEService>>,
  battery_level_characteristic: Arc<Mutex<BLECharacteristic>>,
  battery_level_descriptor: BLE2904,
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

    let battery_service = server.create_service(BLE_SVC_BAS_UUID16);
    let battery_level_characteristic = battery_service.lock().create_characteristic(
      BLE_SVC_BAS_CHR_UUID16_BATTERY_LEVEL,
      NimbleProperties::READ | NimbleProperties::NOTIFY,
    );
    let mut battery_level_descriptor = battery_level_characteristic.lock().create_2904_descriptor();
    battery_level_descriptor
      .format(BLE2904Format::UINT8)
      .namespace(1)
      .unit(0x27ad);

    Self {
      device_info_service,
      pnp_characteristic,
      manufacturer_characteristic: None,
      hid_service,
      hid_info_characteristic,
      report_map_characteristic,
      hid_control_characteristic,
      protocol_mode_characteristic,
      battery_service,
      battery_level_characteristic,
      battery_level_descriptor,
    }
  }

  /// Sets the Plug n Play characteristic value.
  pub fn report_map(&mut self, map: &[u8]) {
    self.report_map_characteristic.lock().set_value(map);
  }

  pub fn manufacturer(&mut self, name: &str) {
    let chr = self.manufacturer_characteristic.get_or_insert_with(|| {
      self.device_info_service.lock().create_characteristic(
        BLE_SVC_DIS_CHR_UUID16_MANUFACTURER_NAME,
        NimbleProperties::READ,
      )
    });
    chr.lock().set_value(name.as_bytes());
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
      DescriptorProperties::READ | DescriptorProperties::READ_ENC,
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
      DescriptorProperties::READ
        | DescriptorProperties::WRITE
        | DescriptorProperties::READ_ENC
        | DescriptorProperties::WRITE_ENC,
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
      DescriptorProperties::READ
        | DescriptorProperties::WRITE
        | DescriptorProperties::READ_ENC
        | DescriptorProperties::WRITE_ENC,
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

  /// Set the battery level characteristic value.
  pub fn set_battery_level(&mut self, level: u8) {
    self.battery_level_characteristic.lock().set_value(&[level]);
  }

  /// Returns a pointer to the HID service.
  pub fn hid_service(&self) -> &Arc<Mutex<BLEService>> {
    &self.hid_service
  }
}
