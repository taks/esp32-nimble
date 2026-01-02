use esp_idf_svc::hal::task::block_on;
use esp32_nimble::{BLEDevice, BLEScan};
use log::*;

fn main() -> anyhow::Result<()> {
  esp_idf_svc::sys::link_patches();

  esp_idf_svc::log::EspLogger::initialize_default();
  log::set_max_level(log::LevelFilter::Debug);

  block_on(async {
    let ble_device = BLEDevice::take();
    let mut ble_scan = BLEScan::new();
    ble_scan.active_scan(true).interval(100).window(99);

    ble_scan
      .start(ble_device, 5000, |device, data| {
        info!("Advertised Device: ({:?}, {:?})", device, data);
        None::<()>
      })
      .await?;

    info!("Scan end");

    Ok(())
  })
}
