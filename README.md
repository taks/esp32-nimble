# NimBLE Rust wrapper for ESP32
[![crates.io](https://img.shields.io/crates/v/esp32-nimble)](https://crates.io/crates/esp32-nimble)
[![build](https://github.com/taks/esp32-nimble/actions/workflows/ci.yml/badge.svg)](https://github.com/taks/esp32-nimble/actions/workflows/ci.yml)
![crates.io](https://img.shields.io/crates/l/esp32-nimble)

## Features

- [x] GATT server
  - [x] Advertisement
    - [x] Custom name
    - [ ] Custom appearance
  - [x] Services
    - [ ] Declaration
    - [ ] Advertisement
  - [x] Characteristics
    - [ ] Declaration
    - [ ] Broadcast
    - [x] Read
      - [x] Static (by stack)
      - [x] Dynamic (by application, with callback)
      - [ ] Long
    - [x] Write
      - [x] With response
      - [x] Without response
      - [ ] Long
    - [x] Notify
    - [x] Indicate
  - [x] Descriptors
    - [ ] Declaration
    - [x] Read
    - [x] Write
  - [ ] Encryption
- [x] GATT client
  - [x] Scan
  - [x] Services
  - [x] Characteristics
    - [ ] Broadcast
    - [x] Read
    - [x] Write
    - [x] Notify
    - [x] Indicate
  - [x] Descriptors
    - [x] Read
    - [x] Write
  - [ ] Encryption
