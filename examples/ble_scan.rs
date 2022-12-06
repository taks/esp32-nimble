#![no_std]
#![no_main]

extern crate alloc;

use ble_client::BLEDevice;
use esp_idf_hal::task::executor::{EspExecutor, Local};
use esp_idf_sys as _;
use log::*;

#[no_mangle]
fn main() {
  // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
  // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
  esp_idf_sys::link_patches();

  // Bind the log crate to the ESP Logging facilities
  esp_idf_svc::log::EspLogger::initialize_default();
  log::set_max_level(log::LevelFilter::Debug);

  // WDT OFF
  unsafe {
    esp_idf_sys::esp_task_wdt_delete(esp_idf_sys::xTaskGetIdleTaskHandleForCPU(
      esp_idf_hal::cpu::core() as u32,
    ));
  };

  let executor = EspExecutor::<16, Local>::new();
  let _task = executor
    .spawn_local(async {
      let ble_device = BLEDevice::take();
      let ble_scan = ble_device.get_scan();
      ble_scan
        .active_scan(true)
        .interval(100)
        .window(99)
        .on_result(|param| {
          info!("Advertised Device: {:?}", param);
        });
      ble_scan.start(5000).await.unwrap();
      info!("Scan end");
    })
    .unwrap();

  executor.run(|| true);
}

#[panic_handler]
#[allow(dead_code)]
fn panic(info: &core::panic::PanicInfo) -> ! {
  ::log::error!("{:?}", info);
  unsafe {
    esp_idf_sys::abort();
    core::hint::unreachable_unchecked();
  }
}
