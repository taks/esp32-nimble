#![no_std]
#![no_main]

extern crate alloc;

use alloc::sync::Arc;
use esp32_nimble::{
  enums::*,
  utilities::{mutex::Mutex, BleUuid},
  BLEClient, BLEDevice,
};
use esp_idf_hal::task::executor::{EspExecutor, Local};
use esp_idf_sys as _;
use log::*;

const SERVICE_UUID: BleUuid = BleUuid::Uuid16(0xABCD);

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

  esp_idf_svc::timer::embassy_time::driver::link();

  let executor = EspExecutor::<16, Local>::new();
  let _task = executor
    .spawn_local(async {
      let device = BLEDevice::take();
      device
        .set_power(PowerType::Default, PowerLevel::P9)
        .unwrap();
      device
        .security()
        .set_auth(true, true, true)
        .set_io_cap(SecurityIOCap::KeyboardOnly);

      let ble_scan = device.get_scan();
      let connect_device = Arc::new(Mutex::new(None));

      let device0 = connect_device.clone();
      ble_scan
        .active_scan(true)
        .interval(100)
        .window(99)
        .on_result(move |device| {
          if device.is_advertising_service(&SERVICE_UUID) {
            BLEDevice::take().get_scan().stop().unwrap();
            (*device0.lock()) = Some(device.clone());
          }
        });
      ble_scan.start(10000).await.unwrap();

      let device = &*connect_device.lock();

      let Some(device) = device else {
        ::log::warn!("device not found");
        return;
      };

      info!("Advertised Device: {:?}", device);

      let mut client = BLEClient::new();
      client.connect(device.addr()).await.unwrap();
      client.on_passkey_request(|| 123456);
      client.secure_connection().await.unwrap();

      let service = client.get_service(SERVICE_UUID).await.unwrap();

      let non_secure_characteristic = service
        .get_characteristic(BleUuid::Uuid16(0x1234))
        .await
        .unwrap();
      let value = non_secure_characteristic.read_value().await.unwrap();
      ::log::info!(
        "{:?} value: {}",
        non_secure_characteristic.uuid(),
        core::str::from_utf8(&value).unwrap()
      );

      let secure_characteristic = service
        .get_characteristic(BleUuid::Uuid16(0x1235))
        .await
        .unwrap();
      let value = secure_characteristic.read_value().await.unwrap();
      ::log::info!(
        "{:?} value: {}",
        secure_characteristic.uuid(),
        core::str::from_utf8(&value).unwrap()
      );

      client.disconnect().unwrap();
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
  }
}
