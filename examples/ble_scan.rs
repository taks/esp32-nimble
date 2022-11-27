#![no_std]
#![no_main]

extern crate alloc;

use ble_client::ble_device::BLEDevice;

use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use embedded_hal::delay::DelayUs;

use log::*;

#[no_mangle]
fn main() {
  // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
  // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
  esp_idf_sys::link_patches();

  // Bind the log crate to the ESP Logging facilities
  esp_idf_svc::log::EspLogger::initialize_default();

  let ble_device = BLEDevice::init("");
  let ble_scan = ble_device.get_scan();
  ble_scan
    .active_scan(true)
    .interval(100)
    .window(99)
    .on_result(|param| {
      info!("Advertised Device: {:X?}", param.bda);
    })
    .on_completed(|| {
      info!("Scan end.")
    });
  ble_scan.start(5);

  let mut delay = esp_idf_hal::delay::Ets {};

  loop {
    delay.delay_ms(100).unwrap();
  }
}
