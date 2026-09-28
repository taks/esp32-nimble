use esp32_nimble::{
    BLEAdvertisementData, BLEDevice, BLEServerCallbacks, NimbleProperties, enums::*,
    utilities::BleUuid,
};

struct ServerCallbacks;
impl BLEServerCallbacks for ServerCallbacks {
    fn on_connect(
        &mut self,
        server: &mut esp32_nimble::BLEServer,
        desc: &esp32_nimble::BLEConnDesc,
    ) {
        ::log::info!("Client connected: {:?}", desc);

        if server.connected_count() < (esp_idf_svc::sys::CONFIG_BT_NIMBLE_MAX_CONNECTIONS as _) {
            ::log::info!("Multi-connect support: start advertising");
            BLEDevice::take().get_advertising().lock().start().unwrap();
        }
    }

    fn on_disconnect(
        &mut self,
        _desc: &esp32_nimble::BLEConnDesc,
        reason: Result<(), esp32_nimble::BLEError>,
    ) {
        ::log::info!("Client disconnected ({:?})", reason);
    }

    fn on_authentication_complete(
        &mut self,
        _server: &mut esp32_nimble::BLEServer,
        desc: &esp32_nimble::BLEConnDesc,
        result: Result<(), esp32_nimble::BLEError>,
    ) {
        ::log::info!("AuthenticationComplete({:?}): {:?}", result, desc);
    }
}

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
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
    server.set_callbacks(ServerCallbacks);

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
    ble_advertising.lock().set_data(
        BLEAdvertisementData::new()
            .name("ESP32-GATT-Server")
            .add_service_uuid(BleUuid::Uuid16(0xABCD)),
    )?;
    ble_advertising.lock().start()?;

    ::log::info!("bonded_addresses: {:?}", device.bonded_addresses());

    loop {
        esp_idf_svc::hal::delay::FreeRtos::delay_ms(1000);
    }
}
