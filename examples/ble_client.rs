#![no_std]
#![no_main]

extern crate alloc;

use alloc::sync::Arc;
use esp32_nimble::{utilities::mutex::Mutex, uuid128, BLEClient, BLEDevice};
use esp_idf_hal::prelude::Peripherals;
use esp_idf_hal::task::block_on;
use esp_idf_hal::timer::{TimerConfig, TimerDriver};
use esp_idf_sys as _;
use log::*;

#[no_mangle]
fn main() {
  esp_idf_sys::link_patches();
  esp_idf_svc::log::EspLogger::initialize_default();

  let peripherals = Peripherals::take().unwrap();
  let mut timer = TimerDriver::new(peripherals.timer00, &TimerConfig::new()).unwrap();

  block_on(async {
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
      client.on_connect(|client| {
        client.update_conn_params(120, 120, 0, 60).unwrap();
      });
      client.connect(device.addr()).await.unwrap();

      let service = client
        .get_service(uuid128!("fafafafa-fafa-fafa-fafa-fafafafafafa"))
        .await
        .unwrap();

      let uuid = uuid128!("d4e0e0d0-1a2b-11e9-ab14-d663bd873d93");
      let characteristic = service.get_characteristic(uuid).await.unwrap();
      let value = characteristic.read_value().await.unwrap();
      ::log::info!(
        "{:?} value: {}",
        uuid,
        core::str::from_utf8(&value).unwrap()
      );

      let uuid = uuid128!("a3c87500-8ed3-4bdf-8a39-a01bebede295");
      let characteristic = service.get_characteristic(uuid).await.unwrap();
      ::log::info!("subscribe {:?}", uuid);
      characteristic
        .on_notify(|data| {
          ::log::info!("{}", core::str::from_utf8(data).unwrap());
        })
        .subscribe_notify(false)
        .await
        .unwrap();

      timer.delay(timer.tick_hz()).await.unwrap();

      // Timer::after(core::time::Duration::from_secs(10)).await;

      client.disconnect().unwrap();
    }
  });
}

#[panic_handler]
#[allow(dead_code)]
fn panic(info: &core::panic::PanicInfo) -> ! {
  ::log::error!("{:?}", info);
  unsafe { esp_idf_sys::abort() }
}
