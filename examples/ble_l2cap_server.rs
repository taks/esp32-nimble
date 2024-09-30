#![feature(future_join)]

use esp32_nimble::{l2cap::L2capServer, BLEAdvertisementData, BLEDevice};
use esp_idf_svc::hal::task::block_on;
use std::future::join;

fn main() {
  esp_idf_svc::sys::link_patches();
  esp_idf_svc::log::EspLogger::initialize_default();

  let ble_device = BLEDevice::take();
  let ble_advertising = ble_device.get_advertising();

  // let server = ble_device.get_server();

  ble_advertising
    .lock()
    .set_data(BLEAdvertisementData::new().name("ESP32-GATT-Server"))
    .unwrap();
  ble_advertising.lock().start().unwrap();

  let l2cap1 = L2capServer::create(0x1001, 512).unwrap();
  let l2cap2 = L2capServer::create(0x1002, 512).unwrap();

  block_on(async {
    join!(run_callback(l2cap1), run_callback(l2cap2)).await;
  });
}

async fn run_callback(server: &mut L2capServer) {
  loop {
    let recv = server.rx().await;
    ::log::info!("< {:?}", recv.data());
    server.tx(recv.data()).unwrap();
  }
}
