use esp32_nimble::{l2cap::L2capServer, BLEAdvertisementData, BLEDevice};
use esp_idf_sys as _;

fn main() {
  esp_idf_sys::link_patches();
  esp_idf_svc::log::EspLogger::initialize_default();

  let ble_device = BLEDevice::take();
  let ble_advertising = ble_device.get_advertising();

  // let server = ble_device.get_server();

  ble_advertising
    .lock()
    .set_data(BLEAdvertisementData::new().name("ESP32-GATT-Server"))
    .unwrap();
  ble_advertising.lock().start().unwrap();

  L2capServer::create(0x1001, 512, |data| {
    ::log::info!("Data received(0x1001): {:X?}", data.sdu_rx());
  })
  .unwrap();
  L2capServer::create(0x1002, 512, |data| {
    ::log::info!("Data received(0x1002): {:X?}", data.sdu_rx());
  })
  .unwrap();

  loop {
    esp_idf_hal::delay::FreeRtos::delay_ms(1000);
  }
}
