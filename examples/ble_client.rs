use bstr::ByteSlice;
use esp32_nimble::{uuid128, BLEClient, BLEDevice};
use esp_idf_svc::hal::{
  prelude::Peripherals,
  task::block_on,
  timer::{TimerConfig, TimerDriver},
};

fn main() -> anyhow::Result<()> {
  esp_idf_svc::sys::link_patches();
  esp_idf_svc::log::EspLogger::initialize_default();

  let peripherals = Peripherals::take()?;
  let mut timer = TimerDriver::new(peripherals.timer00, &TimerConfig::new())?;

  block_on(async {
    let ble_device = BLEDevice::take();
    let ble_scan = ble_device.get_scan();
    let device = ble_scan
      .active_scan(true)
      .interval(100)
      .window(99)
      .find_device(10000, |device| device.name().contains_str("ESP32"))
      .await?;

    if let Some(device) = device {
      let mut client = BLEClient::new();
      client.on_connect(|client| {
        client.update_conn_params(120, 120, 0, 60).unwrap();
      });
      client.connect(device.addr()).await?;

      let service = client
        .get_service(uuid128!("fafafafa-fafa-fafa-fafa-fafafafafafa"))
        .await?;

      let uuid = uuid128!("d4e0e0d0-1a2b-11e9-ab14-d663bd873d93");
      let characteristic = service.get_characteristic(uuid).await?;
      let value = characteristic.read_value().await?;
      ::log::info!(
        "{} value: {}",
        characteristic,
        core::str::from_utf8(&value)?
      );

      let uuid = uuid128!("a3c87500-8ed3-4bdf-8a39-a01bebede295");
      let characteristic = service.get_characteristic(uuid).await?;

      if !characteristic.can_notify() {
        ::log::error!("characteristic can't notify: {}", characteristic);
        return anyhow::Ok(());
      }

      ::log::info!("subscribe to {}", characteristic);
      characteristic
        .on_notify(|data| {
          ::log::info!("{}", core::str::from_utf8(data).unwrap());
        })
        .subscribe_notify(false)
        .await?;

      timer.delay(timer.tick_hz() * 10).await?;

      client.disconnect()?;
    }

    return anyhow::Ok(());
  })
}
