#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;
use esp32_nimble::{uuid128, BLEDevice, NimbleProperties};
use esp_idf_sys as _;

#[no_mangle]
fn main() {
  esp_idf_sys::link_patches();
  esp_idf_svc::log::EspLogger::initialize_default();

  let ble_device = BLEDevice::take();

  let server = ble_device.get_server();
  server.on_connect(|server, desc| {
    ::log::info!("Client connected");

    server
      .update_conn_params(desc.conn_handle, 24, 48, 0, 60)
      .unwrap();

    ::log::info!("Multi-connect support: start advertising");
    ble_device.get_advertising().start().unwrap();
  });
  server.on_disconnect(|desc, reason| {
    ::log::info!("Client disconnected ({:X})", reason);
  });
  let service = server.create_service(uuid128!("fafafafa-fafa-fafa-fafa-fafafafafafa"));

  // A static characteristic.
  let static_characteristic = service.lock().create_characteristic(
    uuid128!("d4e0e0d0-1a2b-11e9-ab14-d663bd873d93"),
    NimbleProperties::READ,
  );
  static_characteristic
    .lock()
    .set_value("Hello, world!".as_bytes());

  // A characteristic that notifies every second.
  let notifying_characteristic = service.lock().create_characteristic(
    uuid128!("a3c87500-8ed3-4bdf-8a39-a01bebede295"),
    NimbleProperties::READ | NimbleProperties::NOTIFY,
  );
  notifying_characteristic.lock().set_value(b"Initial value.");

  // A writable characteristic.
  let writable_characteristic = service.lock().create_characteristic(
    uuid128!("3c9a3f00-8ed3-4bdf-8a39-a01bebede295"),
    NimbleProperties::READ | NimbleProperties::WRITE,
  );
  writable_characteristic
    .lock()
    .on_read(move |_, _| {
      ::log::info!("Read from writable characteristic.");
    })
    .on_write(move |args| {
      ::log::info!("Wrote to writable characteristic: {:?}", args.recv_data);
    });

  let ble_advertising = ble_device.get_advertising();
  ble_advertising
    .name("ESP32-GATT-Server")
    .add_service_uuid(uuid128!("fafafafa-fafa-fafa-fafa-fafafafafafa"));

  ble_advertising.start().unwrap();

  let mut counter = 0;
  loop {
    esp_idf_hal::delay::FreeRtos::delay_ms(1000);
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
  unsafe { esp_idf_sys::abort() }
}
