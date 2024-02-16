use esp32_nimble::{
  enums::*, utilities::BleUuid, BLEAdvertisementData, BLEDevice, BLEReturnCode, NimbleProperties,
};
use esp_idf_sys as _;

fn main() {
  esp_idf_sys::link_patches();
  esp_idf_svc::log::EspLogger::initialize_default();

  let device = BLEDevice::take();
  let ble_advertising = device.get_advertising();

  device
    .security()
    .set_auth(AuthReq::all())
    .set_passkey(123456)
    .set_io_cap(SecurityIOCap::DisplayOnly)
    .resolve_rpa();

  let server = device.get_server();
  server.on_connect(|server, desc| {
    ::log::info!("Client connected: {:?}", desc);

    if server.connected_count() < (esp_idf_sys::CONFIG_BT_NIMBLE_MAX_CONNECTIONS as _) {
      ::log::info!("Multi-connect support: start advertising");
      ble_advertising.lock().start().unwrap();
    }
  });
  server.on_disconnect(|_desc, reason| {
    ::log::info!("Client disconnected ({:?})", BLEReturnCode(reason as _));
  });
  server.on_authentication_complete(|desc, status| {
    ::log::info!("AuthenticationComplete({}): {:?}", status, desc);
  });

  let service = server.create_service(BleUuid::Uuid16(0xABCD));

  let non_secure_characteristic = service
    .lock()
    .create_characteristic(BleUuid::Uuid16(0x1234), NimbleProperties::READ);
  non_secure_characteristic
    .lock()
    .set_value("non_secure_characteristic".as_bytes());

  let secure_characteristic = service.lock().create_characteristic(
    BleUuid::Uuid16(0x1235),
    NimbleProperties::READ | NimbleProperties::READ_ENC | NimbleProperties::READ_AUTHEN,
  );
  secure_characteristic
    .lock()
    .set_value("secure_characteristic".as_bytes());

  // With esp32-c3, advertising stops when a device is bonded.
  // (https://github.com/taks/esp32-nimble/issues/70)
  #[cfg(esp32c3)]
  ble_advertising.lock().on_complete(|_| {
    ble_advertising.lock().start().unwrap();
  });
  ble_advertising
    .lock()
    .set_data(
      BLEAdvertisementData::new()
        .name("ESP32-GATT-Server")
        .add_service_uuid(BleUuid::Uuid16(0xABCD)),
    )
    .unwrap();
  ble_advertising.lock().start().unwrap();

  ::log::info!("bonded_addresses: {:?}", device.bonded_addresses().unwrap());

  loop {
    esp_idf_hal::delay::FreeRtos::delay_ms(1000);
  }
}
