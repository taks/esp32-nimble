#![no_std]
#![allow(clippy::new_without_default)]
#![allow(clippy::single_match)]

extern crate alloc;

pub type BLEAddress = esp_idf_sys::ble_addr_t;
pub type Signal<T> = embassy_sync::signal::Signal<esp_idf_hal::task::embassy_sync::EspRawMutex, T>;
pub type Channel<T, const N: usize> =
  embassy_sync::channel::Channel<esp_idf_hal::task::embassy_sync::EspRawMutex, T, N>;

mod ble_advertised_device;
pub use self::ble_advertised_device::BLEAdvertisedDevice;

mod ble_client;
pub use self::ble_client::BLEClient;

mod ble_device;
pub use self::ble_device::BLEDevice;

mod ble_remote_service;
pub use self::ble_remote_service::BLERemoteService;

mod ble_scan;
pub use self::ble_scan::BLEScan;

mod ble_return_code;
pub use self::ble_return_code::BLEReturnCode;

pub mod utilities;
