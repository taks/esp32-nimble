#![no_std]
#![allow(clippy::new_without_default)]
#![allow(clippy::single_match)]
#![feature(decl_macro)]

extern crate alloc;

#[doc(hidden)]
pub use uuid::uuid as uuid_macro;

mod ble_address;
pub use self::ble_address::*;

pub(crate) type Signal<T> =
  embassy_sync::signal::Signal<esp_idf_hal::task::embassy_sync::EspRawMutex, T>;
#[allow(dead_code)]
pub(crate) type Channel<T, const N: usize> =
  embassy_sync::channel::Channel<esp_idf_hal::task::embassy_sync::EspRawMutex, T, N>;

mod ble_device;
pub use self::ble_device::BLEDevice;

mod ble_error;
pub(crate) use self::ble_error::ble;
pub use self::ble_error::BLEError;

mod ble_security;
pub use self::ble_security::BLESecurity;

pub mod enums;

mod client;
pub use self::client::*;

mod server;
pub use self::server::*;

pub mod utilities;
