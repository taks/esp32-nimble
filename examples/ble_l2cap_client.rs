use bstr::ByteSlice;
use esp32_nimble::{l2cap::L2capClient, BLEClient, BLEDevice};
use esp_idf_hal::task::block_on;
use esp_idf_sys as _;

fn main() {
  esp_idf_sys::link_patches();
  esp_idf_svc::log::EspLogger::initialize_default();

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
      client.connect(device.addr()).await.unwrap();

      let mut l2cap = L2capClient::connect(&client, 0x1002, 512).await.unwrap();
      for i in 0..4 {
        l2cap.send(format!("test{}", i).as_bytes()).unwrap();
        esp_idf_hal::delay::FreeRtos::delay_ms(1000);
      }
      l2cap.disconnect().await.unwrap();

      client.disconnect().unwrap();
    }
  });
}
