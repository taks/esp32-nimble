#![no_std]
#![no_main]

extern crate alloc;

use alloc::sync::Arc;
use embassy_time::{Duration, Timer};
use esp32_nimble::{utilities::mutex::Mutex, utilities::BleUuid, *};
use esp_idf_hal::task::executor::{EspExecutor, Local};
use esp_idf_sys as _;
use log::*;

#[no_mangle]
fn main() {
  esp_idf_sys::link_patches();

  esp_idf_svc::log::EspLogger::initialize_default();
  esp_idf_svc::log::EspLogger.set_target_level("NimBLE", log::LevelFilter::Warn);

  log::set_max_level(log::LevelFilter::Debug);

  // WDT OFF
  unsafe {
    esp_idf_sys::esp_task_wdt_delete(esp_idf_sys::xTaskGetIdleTaskHandleForCPU(
      esp_idf_hal::cpu::core() as u32,
    ));
  };

  esp_idf_svc::timer::embassy_time::driver::link();

  let executor = EspExecutor::<16, Local>::new();
  let _task = executor
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
          if device.name().contains("ESP32") {
            BLEDevice::take().get_scan().stop().unwrap();
            (*device0.lock()) = Some(device.clone());
          }
        });
      ble_scan.start(10000).await.unwrap();

      let device = &*connect_device.lock();
      if let Some(device) = device {
        info!("Advertised Device: {:?}", device);

        let mut client = BLEClient::new();
        client.connect(device.addr()).await.unwrap();

        let service = client
          .get_service(BleUuid::from_uuid128_string(
            "fafafafa-fafa-fafa-fafa-fafafafafafa",
          ))
          .await
          .unwrap();

        let uuid = BleUuid::from_uuid128_string("d4e0e0d0-1a2b-11e9-ab14-d663bd873d93");
        let characteristic = service.get_characteristic(uuid).await.unwrap();
        let value = characteristic.read_value().await.unwrap();
        ::log::info!(
          "{:?} value: {}",
          uuid,
          core::str::from_utf8(&value).unwrap()
        );

        let uuid = BleUuid::from_uuid128_string("a3c87500-8ed3-4bdf-8a39-a01bebede295");
        let characteristic = service.get_characteristic(uuid).await.unwrap();
        ::log::info!("subscribe {:?}", uuid);
        characteristic
          .on_notify(|data| {
            ::log::info!("{}", core::str::from_utf8(&data).unwrap());
          })
          .subscribe(true, false)
          .await
          .unwrap();

        Timer::after(Duration::from_secs(10)).await;

        client.disconnect().unwrap();
      }
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
