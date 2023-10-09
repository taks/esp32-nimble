#![no_std]
#![no_main]

extern crate alloc;
use esp_idf_sys::{BLE_HCI_LE_PHY_1M, BLE_HCI_LE_PHY_CODED};

use esp32_nimble::{
  utilities::BleUuid, BLEAddress, BLEAddressType, BLEDevice, BLEExtAdvertisement, NimbleProperties,
};

const SERVICE_UUID: BleUuid = BleUuid::Uuid16(0xABCD);

#[no_mangle]
fn main() {
  esp_idf_sys::link_patches();
  esp_idf_svc::log::EspLogger::initialize_default();

  let ble_device = BLEDevice::take();

  let server = ble_device.get_server();

  let service = server.create_service(SERVICE_UUID);
  let characteristic = service
    .lock()
    .create_characteristic(BleUuid::Uuid16(0x1234), NimbleProperties::READ);
  characteristic.lock().set_value("Hello, world!".as_bytes());

  let mut ext_scannable =
    BLEExtAdvertisement::new(BLE_HCI_LE_PHY_CODED as _, BLE_HCI_LE_PHY_1M as _);
  ext_scannable.scannable(true);
  ext_scannable.connectable(false);

  ext_scannable.service_data(SERVICE_UUID, "Scan me!".as_bytes());
  ext_scannable.enable_scan_request_callback(true);

  let mut legacy_connectable =
    BLEExtAdvertisement::new(BLE_HCI_LE_PHY_1M as _, BLE_HCI_LE_PHY_1M as _);
  legacy_connectable.address(&BLEAddress::new(
    [0xDE, 0xAD, 0xBE, 0xEF, 0xBA, 0xAD],
    BLEAddressType::Random,
  ));

  legacy_connectable.name("Legacy");
  legacy_connectable.complete_service(&SERVICE_UUID);

  legacy_connectable.legacy_advertising(true);
  legacy_connectable.connectable(true);

  let mut legacy_scan_response =
    BLEExtAdvertisement::new(BLE_HCI_LE_PHY_1M as _, BLE_HCI_LE_PHY_1M as _);
  legacy_scan_response.service_data(SERVICE_UUID, "Legacy SR".as_bytes());

  let advertising = ble_device.get_advertising();
  advertising
    .set_instance_data(0, &mut ext_scannable)
    .unwrap();
  advertising
    .set_instance_data(1, &mut legacy_connectable)
    .unwrap();
  advertising
    .set_scan_response_data(1, &mut legacy_scan_response)
    .unwrap();

  advertising.start(0).unwrap();
  advertising.start(1).unwrap();

  loop {
    esp_idf_hal::delay::FreeRtos::delay_ms(5000);
  }
}

#[panic_handler]
#[allow(dead_code)]
fn panic(info: &core::panic::PanicInfo) -> ! {
  ::log::error!("{:?}", info);
  unsafe { esp_idf_sys::abort() }
}
