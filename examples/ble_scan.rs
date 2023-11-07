
use esp32_nimble::BLEDevice;
use esp_idf_hal::task::block_on;
use esp_idf_sys as _;
use log::*;

fn main() {
  // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
  // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
  esp_idf_sys::link_patches();

  // Bind the log crate to the ESP Logging facilities
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
    ble_scan.start(5000).await.unwrap();
    info!("Scan end");
  });
}
