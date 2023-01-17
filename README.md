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

## Features
- [x] GATT server
  - [x] Advertisement
  - [x] Services
  - [x] Characteristics
    - [x] Read
    - [x] Write
    - [x] Notify
    - [x] Indicate
  - [x] Descriptors
    - [x] Read
    - [x] Write
  - [x] Encryption
- [x] GATT client
  - [x] Scan
  - [x] Services
  - [x] Characteristics
    - [x] Read
    - [x] Write
    - [x] Notify
    - [x] Indicate
  - [x] Descriptors
    - [x] Read
    - [x] Write
  - [x] Encryption
