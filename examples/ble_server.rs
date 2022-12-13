#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;
use esp32_nimble::{utilities::BleUuid, BLEDevice, NimbleProperties};
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

  let ble_device = BLEDevice::take();

  let server = ble_device.get_server();
  let service = server.create_service(BleUuid::from_uuid128_string(
    "fafafafa-fafa-fafa-fafa-fafafafafafa",
  ));

  // A static characteristic.
  let static_characteristic = service.lock().create_characteristic(
    BleUuid::from_uuid128_string("d4e0e0d0-1a2b-11e9-ab14-d663bd873d93"),
    NimbleProperties::Read,
  );
  static_characteristic
    .lock()
    .set_value("Hello, world!".as_bytes());

  // A characteristic that notifies every second.
  let notifying_characteristic = service.lock().create_characteristic(
    BleUuid::from_uuid128_string("a3c87500-8ed3-4bdf-8a39-a01bebede295"),
    NimbleProperties::Read | NimbleProperties::Notify,
  );
  notifying_characteristic.lock().set_value(b"Initial value.");

  let ble_advertising = ble_device.get_advertising();
  ble_advertising
    .name("ESP32-GATT-Server")
    .add_service_uuid(BleUuid::from_uuid128_string(
      "fafafafa-fafa-fafa-fafa-fafafafafafa",
    ));

  ble_advertising.start(None).unwrap();

  let mut counter = 0;
  loop {
    esp_idf_hal::delay::Ets::delay_ms(1000);
    notifying_characteristic
      .lock()
      .set_value(format!("Counter: {counter}").as_bytes())
      .notify();

    counter += 1;
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
