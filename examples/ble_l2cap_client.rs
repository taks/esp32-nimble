use bstr::ByteSlice;
use core::str;
use esp32_nimble::{l2cap::L2capClient, BLEDevice, BLEScan};
use esp_idf_svc::hal::task::block_on;

fn main() -> anyhow::Result<()> {
  esp_idf_svc::sys::link_patches();
  esp_idf_svc::log::EspLogger::initialize_default();

  block_on(async {
    let ble_device = BLEDevice::take();
    let mut ble_scan = BLEScan::new();
    let device = ble_scan
      .active_scan(true)
      .interval(100)
      .window(99)
      .start(ble_device, 10000, |device, data| {
        if let Some(name) = data.name() {
          if name.contains_str("ESP32") {
            return Some(*device);
          }
        }
        None
      })
      .await?;

    if let Some(device) = device {
      let mut client = ble_device.new_client();
      client.connect(&device.addr()).await.unwrap();

      let mut l2cap = L2capClient::connect(&client, 0x1002, 512).await.unwrap();
      for i in 0..4 {
        l2cap.tx(format!("test{}", i).as_bytes()).unwrap();
        ::log::info!("< {:?}", str::from_utf8(l2cap.rx().await.data()));
        esp_idf_svc::hal::delay::FreeRtos::delay_ms(1000);
      }
      l2cap.disconnect().await.unwrap();

      client.disconnect().unwrap();
    }

    anyhow::Ok(())
  })
}
