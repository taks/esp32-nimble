# NimBLE Rust wrapper for ESP32
[![crates.io](https://img.shields.io/crates/v/esp32-nimble)](https://crates.io/crates/esp32-nimble)
[![build](https://github.com/taks/esp32-nimble/actions/workflows/ci.yml/badge.svg)](https://github.com/taks/esp32-nimble/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/esp32-nimble)](https://github.com/taks/esp32-nimble/blob/develop/LICENSE)
[![Documentation](https://img.shields.io/badge/docs-esp32--nimble-brightgreen)](https://taks.github.io/esp32-nimble/esp32_nimble/index.html)

This is a Rust wrapper for the NimBLE Bluetooth stack for ESP32.
Inspired by [NimBLE-Arduino](https://github.com/h2zero/NimBLE-Arduino).

## Usage
Add below settings to your project's `sdkconfig.defaults`.
```
CONFIG_BT_ENABLED=y
CONFIG_BT_BLE_ENABLED=y
CONFIG_BT_BLUEDROID_ENABLED=n
CONFIG_BT_NIMBLE_ENABLED=y
```

- To enable Extended advertising, additionally append `CONFIG_BT_NIMBLE_EXT_ADV=y`.<br>
  (For use with ESP32C3, ESP32S3, ESP32H2 ONLY)

### Configuring for iOS Auto-Reconnect

For auto-reconnection of iOS devices to a BLE server, specific settings in the `sdkconfig` file and the Rust code are required.

#### Update `sdkconfig`

Add the following line to your `sdkconfig` file:

```
CONFIG_BT_NIMBLE_NVS_PERSIST=y
```

This ensures the persistence of bonding information in the device's non-volatile storage (NVS), allowing iOS devices to automatically reconnect without needing to rebond after a system reset or power cycle.

#### Set Device Security Options in Rust

In your Rust implementation, set the security options as follows:

```rust
device
  .security()
  .set_auth(AuthReq::Bond) // or .set_auth(AuthReq::Bond | AuthReq:Mitm)
  .set_io_cap(SecurityIOCap::NoInputNoOutput)
```

Here, `.set_auth(AuthReq::Bond)` is used to enable bonding, which stores security keys for future reconnections. Note that the use of `.set_auth(AuthReq::Sc)` (Secure Connections) prevents iOS devices from reconnecting.

The `.set_io_cap(SecurityIOCap::NoInputNoOutput)` setting is important for devices that lack a user interface, ensuring compatibility with a wider range of device types.