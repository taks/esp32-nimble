use esp32_nimble::{enums::*, utilities::BleUuid, BLEClient, BLEDevice};
use esp_idf_hal::task::block_on;
use esp_idf_sys as _;
use log::*;

const SERVICE_UUID: BleUuid = BleUuid::Uuid16(0xABCD);

fn main() {
  esp_idf_sys::link_patches();
  esp_idf_svc::log::EspLogger::initialize_default();

  block_on(async {
    let device = BLEDevice::take();
    device
      .set_power(PowerType::Default, PowerLevel::P9)
      .unwrap();
    device
      .security()
      .set_auth(AuthReq::all())
      .set_io_cap(SecurityIOCap::KeyboardOnly);

    let ble_scan = device.get_scan();

    let device = ble_scan
      .active_scan(true)
      .interval(100)
      .window(99)
      .find_device(10000, move |device| {
        device.is_advertising_service(&SERVICE_UUID)
      })
      .await
      .unwrap();

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
  });
}
