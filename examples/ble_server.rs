use esp32_nimble::{
    BLEAdvertisementData, BLEDevice, BLEServerCallbacks, NimbleProperties, uuid128,
};
use std::format;

struct ServerCallbacks;
impl BLEServerCallbacks for ServerCallbacks {
    fn on_connect(
        &mut self,
        server: &mut esp32_nimble::BLEServer,
        desc: &esp32_nimble::BLEConnDesc,
    ) {
        ::log::info!("Client connected: {:?}", desc);

        server
            .update_conn_params(desc.conn_handle(), 24, 48, 0, 60)
            .unwrap();

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
}

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let ble_device = BLEDevice::take();
    let ble_advertising = ble_device.get_advertising();

    let server = ble_device.get_server();
    server.set_callbacks(ServerCallbacks);

    let service = server.create_service(uuid128!("fafafafa-fafa-fafa-fafa-fafafafafafa"));

    // A static characteristic.
    let static_characteristic = service.lock().create_characteristic(
        uuid128!("d4e0e0d0-1a2b-11e9-ab14-d663bd873d93"),
        NimbleProperties::READ,
    );
    static_characteristic
        .lock()
        .set_value("Hello, world!".as_bytes());

    // A characteristic that notifies every second.
    let notifying_characteristic = service.lock().create_characteristic(
        uuid128!("a3c87500-8ed3-4bdf-8a39-a01bebede295"),
        NimbleProperties::READ | NimbleProperties::NOTIFY,
    );
    notifying_characteristic.lock().set_value(b"Initial value.");

    // A writable characteristic.
    let writable_characteristic = service.lock().create_characteristic(
        uuid128!("3c9a3f00-8ed3-4bdf-8a39-a01bebede295"),
        NimbleProperties::READ | NimbleProperties::WRITE,
    );
    writable_characteristic
        .lock()
        .on_read(move |_, _| {
            ::log::info!("Read from writable characteristic.");
        })
        .on_write(|args| {
            ::log::info!(
                "Wrote to writable characteristic: {:?} -> {:?}",
                args.current_data(),
                args.recv_data()
            );
        });

    ble_advertising.lock().set_data(
        BLEAdvertisementData::new()
            .name("ESP32-GATT-Server")
            .add_service_uuid(uuid128!("fafafafa-fafa-fafa-fafa-fafafafafafa")),
    )?;
    ble_advertising.lock().start()?;

    server.ble_gatts_show_local();

    let mut counter = 0;
    loop {
        esp_idf_svc::hal::delay::FreeRtos::delay_ms(1000);
        notifying_characteristic
            .lock()
            .set_value(format!("Counter: {counter}").as_bytes())
            .notify();

        counter += 1;
    }
}
