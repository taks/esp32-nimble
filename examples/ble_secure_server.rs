#![no_std]
#![no_main]

extern crate alloc;

use esp32_nimble::{enums::*, utilities::BleUuid, uuid128, BLEDevice, NimbleProperties};
use esp_idf_sys as _;

#[no_mangle]
fn main() {
  esp_idf_sys::link_patches();
  esp_idf_svc::log::EspLogger::initialize_default();

  // WDT OFF
  unsafe {
    esp_idf_sys::esp_task_wdt_delete(esp_idf_sys::xTaskGetIdleTaskHandleForCPU(
      esp_idf_hal::cpu::core() as u32,
    ));
  };

  let device = BLEDevice::take();
  device
    .security()
    .set_auth(true, true, true)
    .set_passkey(123456)
    .set_io_cap(SecurityIOCap::DisplayOnly);

  let server = device.get_server();
  let service = server.create_service(uuid128!("fafafafa-fafa-fafa-fafa-fafafafafafa"));

  let non_secure_characteristic = service
    .lock()
    .create_characteristic(BleUuid::Uuid16(0x1234), NimbleProperties::Read);
  non_secure_characteristic
    .lock()
    .set_value("non_secure_characteristic".as_bytes());

  let secure_characteristic = service.lock().create_characteristic(
    BleUuid::Uuid16(0x1235),
    NimbleProperties::Read | NimbleProperties::ReadEnc | NimbleProperties::ReadAuthen,
  );
  secure_characteristic
    .lock()
    .set_value("non_secure_characteristic".as_bytes());

  let ble_advertising = device.get_advertising();
  ble_advertising
    .name("ESP32-GATT-Server")
    .add_service_uuid(uuid128!("fafafafa-fafa-fafa-fafa-fafafafafafa"))
    .start()
    .unwrap();

  loop {
    esp_idf_hal::delay::Ets::delay_ms(1000);
  }
}

#[panic_handler]
#[allow(dead_code)]
fn panic(info: &core::panic::PanicInfo) -> ! {
  ::log::error!("{:?}", info);
  unsafe {
    esp_idf_sys::abort();
  }
}
