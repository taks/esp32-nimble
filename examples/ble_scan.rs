use esp32_nimble::BLEDevice;
use esp_idf_svc::hal::task::block_on;
use log::*;

fn main() -> anyhow::Result<()> {
  esp_idf_svc::sys::link_patches();

  esp_idf_svc::log::EspLogger::initialize_default();
  log::set_max_level(log::LevelFilter::Debug);

  block_on(async {
    let ble_device = BLEDevice::take();
    let ble_scan = ble_device.get_scan();
    ble_scan
      .active_scan(true)
      .interval(100)
      .window(99)
      .on_result(|_scan, param| {
        info!("Advertised Device: {:?}", param);
      });
    ble_scan.start(5000).await?;
    info!("Scan end");

    Ok(())
  })
}
