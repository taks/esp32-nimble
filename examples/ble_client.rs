use esp32_nimble::{uuid128, BLEClient, BLEDevice};
use bstr::ByteSlice;
use esp_idf_hal::prelude::Peripherals;
use esp_idf_hal::task::block_on;
use esp_idf_hal::timer::{TimerConfig, TimerDriver};
use esp_idf_sys as _;

fn main() {
  esp_idf_sys::link_patches();
  esp_idf_svc::log::EspLogger::initialize_default();

  let peripherals = Peripherals::take().unwrap();
  let mut timer = TimerDriver::new(peripherals.timer00, &TimerConfig::new()).unwrap();

  block_on(async {
    let ble_device = BLEDevice::take();
    let ble_scan = ble_device.get_scan();
     let device = ble_scan
      .active_scan(true)
      .interval(100)
      .window(99)
      .find_device(10000, |device| device.name().contains_str("ESP32"))
      .await
      .unwrap();

    if let Some(device) = device {
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

      if !characteristic.can_notify() {
        return ::log::error!("characteristic can't notify: {:?}", uuid);
      }

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
