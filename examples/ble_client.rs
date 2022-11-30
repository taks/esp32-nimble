#![no_std]
#![no_main]

extern crate alloc;

use alloc::sync::Arc;
use ble_client::{utilities::mutex::Mutex, *};
use embedded_hal::delay::DelayUs;
use esp_idf_hal::task::executor::{EspExecutor, Local};
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
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
  let task = executor
    .spawn_local(async {
      let ble_device = BLEDevice::take();
      let ble_scan = ble_device.get_scan();
      let connect_device = Arc::new(Mutex::new(None));

      let device0 = connect_device.clone();
      ble_scan
        .active_scan(true)
        .interval(100)
        .window(99)
        .on_result(move |device| {
          if device.name.len() > 0 {
            BLEDevice::take().get_scan().stop();
            (*device0.lock()) = Some(device.clone());
          }
        });
      ble_scan.start(10).await;
      info!("Scan end");

      let device = &*connect_device.lock();
      info!("Advertised Device: {:?}", device);
      if let Some(device) = device {
        let mut client = BLEClient::new();
        client.connect(device.addr, device.addr_type).await.unwrap();
        info!("Connected");
        for s in client.get_services().await.unwrap() {
          info!(" {:?}", s);
        }
        client.disconnect().unwrap();
        info!("Disconnected");
      }
    })
    .unwrap();

  executor.run(|| true);
}
