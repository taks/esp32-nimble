## [0.4.1]
- Added `BLEScan::find_device` ([#55](https::/github.com/taks/esp32-nimble/pull/55))
- Added `BLEAdvertisedDevice::adv_type`, `BLEAdvertisedDevice::adv_flags`
- Added `BLEAddress::from_str`
- Added whitelist API.

## [0.4.0] - 2023-12-01
- Added `BLEAdvertising::min_interval`, `BLEAdvertising::max_interval` ([#51](https::/github.com/taks/esp32-nimble/pull/51))
- Added `can_nofity`, `can_indicate`, `can_read`, `can_write`, `can_write_no_response` and `can_broadcast` functions to `BLERemoteCharacteristic` ([#53](https::/github.com/taks/esp32-nimble/pull/53))
- Add additional checks to prevent OOB panics in BLE advertisement parser ([#54](https::/github.com/taks/esp32-nimble/pull/54))
- Changed type of `BLEAdvertisedDevice::name()` to `&bstr::BStr` ([#54](https::/github.com/taks/esp32-nimble/pull/54))

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
