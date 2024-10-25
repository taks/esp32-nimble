## [0.8.2] - 2024-10-25
- Fix ESP-IDF v5.2.2 build ([#148](https://github.com/taks/esp32-nimble/pull/148))

## [0.8.1] - 2024-09-28
- Fixed BLEClient dropping multiple times (Fix #143) ([#144](https://github.com/taks/esp32-nimble/pull/144))
- Support ESP-IDF v5.2.3

## [0.8.0] - 2024-09-18
- Added `BLEClient::desc`
- Added `BLEDevice::set_preferred_mtu`, `BLEDevice::get_preferred_mtu`
- Fixed corruption of read data
- Fix Guru Meditation Error ([#133](https://github.com/taks/esp32-nimble/pull/133))
- BleAddress LE and BE functions (breaking change) ([#137](https://github.com/taks/esp32-nimble/pull/137))
- Added extended advertising scan support ([#141](https://github.com/taks/esp32-nimble/pull/141))
- Changed scan API (breaking change) ([#142](https://github.com/taks/esp32-nimble/pull/142))
- Add member docs to NimbleProperties ([#103](https://github.com/taks/esp32-nimble/pull/103))

## [0.7.0] - 2024-07-11
- Upgraded to `esp-idf-svc` 0.49
- Fix: Update RSSI Field in BLEAdvertisedDevice Structure During Discovery ([#127](https://github.com/taks/esp32-nimble/pull/127))
- Reflect new build args propagation in build.rs ([#129](https://github.com/taks/esp32-nimble/pull/129))
- Update README.md to include tips on increasing esp-ble task stack size ([#131](https://github.com/taks/esp32-nimble/pull/131))

## [0.6.1] - 2024-05-21
- Added BLECharacteristic.cpfd ([#114](https://github.com/taks/esp32-nimble/pull/114))
- Added Accessor Functions ([#118](https://github.com/taks/esp32-nimble/pull/118))
- Add resolve_rpa to keyboard example ([#120](https://github.com/taks/esp32-nimble/pull/120))
- Fix memory leak ([#121](https://github.com/taks/esp32-nimble/pull/121))
- Impl std::error::Error for BLEError ([#124](https://github.com/taks/esp32-nimble/pull/124))

## [0.6.0] - 2024-03-07
- Implement Display and Debug traits for BLERemoteCharacteristic & BLERemoteService ([#66](https://github.com/taks/esp32-nimble/pull/66))
- Added `BLEAdvertising::on_complete`
- Added `OnWriteArgs::notify::notify` ([#75](https://github.com/taks/esp32-nimble/pull/75))
- Added `BLEServer::on_authentication_complete`
- Added `OnWriteArgs::current_data` ([#81](https://github.com/taks/esp32-nimble/pull/81))
- Changed the return type of `get_advertising` to `Mutex<BLEAdvertising>` ([#84](https://github.com/taks/esp32-nimble/pull/84))
- Added self argument to `BLERemoteCharacteristic::on_subscribe` callback
- Fixed advertising length calculation ([#87](https://github.com/taks/esp32-nimble/pull/87))
- Avoid int underflow in `BLEWriter::write_value()` ([#91](https://github.com/taks/esp32-nimble/pull/91))
- Fixes Add missing return codes for security manager ([#95](https://github.com/taks/esp32-nimble/pull/95))
- Added `disconnect` and `disconnect_with_reason` ([#96](https://github.com/taks/esp32-nimble/pull/96))
- Added `BLEDevice::deinit_full` ([#100](https://github.com/taks/esp32-nimble/pull/100))
- Implement PartialEq and Eq for BLEAddress and BLEAddressType ([#92](https://github.com/taks/esp32-nimble/pull/92))
- Added `BLEAdvertisementData` ([#101](https://github.com/taks/esp32-nimble/pull/101))
- Added `BLEAddress::val`, `BLEAddress::addr_type`
- Changed `BLEReturnCode(pub u32)` to `BLEError(NonZeroI32)` ([#105](https://github.com/taks/esp32-nimble/pull/105))
- Fixed `BLERemoteCharacteristic::get_descriptors` ([#106](https://github.com/taks/esp32-nimble/pull/106), [#108](https://github.com/taks/esp32-nimble/pull/108))

## [0.5.1] - 2024-02-01
- Fixed a bug when changing advertising name. ([#85](https://github.com/taks/esp32-nimble/pull/85))
- Fixed `BLEAdvertising::start_with_duration`
  (`ble_gap_adv_set_fields`, `ble_gap_adv_rsp_set_fields` were called every time.)

## [0.5.0] - 2024-01-10
- Added `BLEScan::find_device` ([#55](https://github.com/taks/esp32-nimble/pull/55))
- Added `BLEAdvertisedDevice::adv_type`, `BLEAdvertisedDevice::adv_flags`
- Added `BLEAddress::from_str`
- Added whitelist API.
- Added `BLEClient::get_rssi` ([#58](https://github.com/taks/esp32-nimble/pull/58))
- Added `BLEConnDesc`
- Fixed no_std build.
- Added `BLEServer::ble_gatts_show_local`

## [0.4.0] - 2023-12-01
- Added `BLEAdvertising::min_interval`, `BLEAdvertising::max_interval` ([#51](https://github.com/taks/esp32-nimble/pull/51))
- Added `can_nofity`, `can_indicate`, `can_read`, `can_write`, `can_write_no_response` and `can_broadcast` functions to `BLERemoteCharacteristic` ([#53](https://github.com/taks/esp32-nimble/pull/53))
- Add additional checks to prevent OOB panics in BLE advertisement parser ([#54](https://github.com/taks/esp32-nimble/pull/54))
- Changed type of `BLEAdvertisedDevice::name()` to `&bstr::BStr` ([#54](https://github.com/taks/esp32-nimble/pull/54))

## [0.3.2] - 2023-11-18
- Fixed unresolved import error for the std environment. ([#48](https://github.com/taks/esp32-nimble/pull/48))
- Added `BLEAdvertising::disc_mode`, `BLEAdvertising::high_duty_cycle` ([#49](https://github.com/taks/esp32-nimble/pull/49))

## [0.3.1] - 2023-11-16
- Fix link error for esp32c3. ([#44](https://github.com/taks/esp32-nimble/pull/44))
- Changed to accept invalid utf8 advertising name. ([#45](https://github.com/taks/esp32-nimble/pull/45))

## [0.3.0] - 2023-11-15
- Upgraded to `esp-idf-hal` 0.42 and `esp-idf-svc` 0.47
- Added Self parameter to BLEScan::on_result.
- Add feature for building with std support ([#36](https://github.com/taks/esp32-nimble/pull/36))
- Change the argument of set_auth function to bitflag.
- Added `BLESecurity::set_security_init_key`, `BLESecurity::set_security_resp_key`, `BLESecurity::resolve_rpa` ([#39](https://github.com/taks/esp32-nimble/pull/39))
- Changed `BLEAdvertising::start_with_duration` to pub.
- Added `BLEDevice::set_own_addr_type`, `BLEDevice::set_rnd_addr` ([#40](https://github.com/taks/esp32-nimble/pull/40))
- Added `BLEAdvertising::advertisement_type` ([#41](https://github.com/taks/esp32-nimble/pull/41))
- Fix compile error for esp32c6 ([#33](https://github.com/taks/esp32-nimble/pull/33))

## [0.2.2] - 2023-10-14
- Fix advertising regression in v0.2.1 ([#31](https://github.com/taks/esp32-nimble/pull/31))
- Added disconnect reason parameter to BLEServer::on_disconnect.

## [0.2.1] - 2023-10-10
- Added methods to set custom adv_data and scan_reponse. ([#25](https://github.com/taks/esp32-nimble/pull/25))
- Added deinit function and support reinitialize. ([#26](https://github.com/taks/esp32-nimble/pull/26))
- Added bond management functions.
- Changed BLEService.start to pub(crate). ([#27](https://github.com/taks/esp32-nimble/pull/27))
- Added Extended advertisement support.
