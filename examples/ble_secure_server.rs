#![no_std]
#![no_main]

extern crate alloc;

use esp32_nimble::{enums::*, utilities::BleUuid, BLEDevice, NimbleProperties};
use esp_idf_sys as _;

#[no_mangle]
fn main() {
  esp_idf_sys::link_patches();
  esp_idf_svc::log::EspLogger::initialize_default();

  let device = BLEDevice::take();
  device
    .security()
    .set_auth(true, true, true)
    .set_passkey(123456)
    .set_io_cap(SecurityIOCap::DisplayOnly);

  let server = device.get_server();
  server.on_connect(|server, desc| {
    ::log::info!("Client connected");
  });
  server.on_disconnect(|desc, reason| {
    ::log::info!("Client disconnected ({:X})", reason);
  });

  let service = server.create_service(BleUuid::Uuid16(0xABCD));

  let non_secure_characteristic = service
    .lock()
    .create_characteristic(BleUuid::Uuid16(0x1234), NimbleProperties::READ);
  non_secure_characteristic
    .lock()
    .set_value("non_secure_characteristic".as_bytes());

  let secure_characteristic = service.lock().create_characteristic(
    BleUuid::Uuid16(0x1235),
    NimbleProperties::READ | NimbleProperties::READ_ENC | NimbleProperties::READ_AUTHEN,
  );
  secure_characteristic
    .lock()
    .set_value("secure_characteristic".as_bytes());

  let ble_advertising = device.get_advertising();
  ble_advertising
    .name("ESP32-GATT-Server")
    .add_service_uuid(BleUuid::Uuid16(0xABCD))
    .start()
    .unwrap();

  ::log::info!("bonded_addresses: {:?}", device.bonded_addresses().unwrap());

  loop {
    esp_idf_hal::delay::FreeRtos::delay_ms(1000);
  }
}

#[panic_handler]
#[allow(dead_code)]
fn panic(info: &core::panic::PanicInfo) -> ! {
  ::log::error!("{:?}", info);
  unsafe { esp_idf_sys::abort() }
}
