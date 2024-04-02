use esp32_nimble::{
  l2cap_server::L2capServer, utilities::L2cap, uuid128, BLEAdvertisementData, BLEDevice,
  NimbleProperties,
};
use esp_idf_sys as _;
use std::format;

fn main() {
  esp_idf_sys::link_patches();
  esp_idf_svc::log::EspLogger::initialize_default();

  let ble_device = BLEDevice::take();
  let ble_advertising = ble_device.get_advertising();

  let server = ble_device.get_server();

  ble_advertising
    .lock()
    .set_data(BLEAdvertisementData::new().name("ESP32-GATT-Server"))
    .unwrap();
  ble_advertising.lock().start().unwrap();

  L2capServer::create(0x1001, 512, |data| {
    ::log::info!("Data received(0x1001): {:X?}", data);
  })
  .unwrap();
  L2capServer::create(0x1002, 512, |data| {
    ::log::info!("Data received(0x1002): {:X?}", data);
  })
  .unwrap();

  loop {
    esp_idf_hal::delay::FreeRtos::delay_ms(1000);
  }
}
