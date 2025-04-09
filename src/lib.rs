#![no_std]
#![allow(clippy::new_without_default)]
#![allow(clippy::single_match)]
#![allow(static_mut_refs)]
#![feature(decl_macro)]
#![feature(get_mut_unchecked)]
#![doc = include_str!("../README.md")]

#[cfg(feature = "std")]
#[allow(unused_imports)]
#[macro_use]
extern crate std;

extern crate alloc;

#[doc(hidden)]
pub use uuid::uuid as uuid_macro;

mod ble_address;
pub use self::ble_address::*;

pub(crate) type Signal<T> =
  embassy_sync::signal::Signal<esp_idf_svc::hal::task::embassy_sync::EspRawMutex, T>;
#[allow(dead_code)]
pub(crate) type Channel<T, const N: usize> =
  embassy_sync::channel::Channel<esp_idf_svc::hal::task::embassy_sync::EspRawMutex, T, N>;

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

pub mod l2cap;

pub mod utilities;

#[allow(unused)]
macro_rules! dbg {
  ($val:expr) => {
    match $val {
      tmp => {
        ::log::info!("{} = {:#?}", stringify!($val), &tmp);
        tmp
      }
    }
  };
}

#[allow(unused)]
pub(crate) use dbg;
