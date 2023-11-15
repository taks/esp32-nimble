## [0.3.0]
- Upgraded to `esp-idf-hal` 0.42 and `esp-idf-svc` 0.47
- Added Self parameter to BLEScan::on_result.
- Add feature for building with std support ([#36](https://github.com/taks/esp32-nimble/pull/36))
- Change the argument of set_auth function to bitflag.
- Added `BLESecurity::set_security_init_key`, `BLESecurity::set_security_resp_key`, `BLESecurity::resolve_rpa` ([#39](https://github.com/taks/esp32-nimble/pull/39))
- Changed `BLEAdvertising::start_with_duration` to pub.
- Added `BLEDevice::set_own_addr_type`, `BLEDevice::set_rnd_addr` ([#40](https://github.com/taks/esp32-nimble/pull/40))
- Added `BLEAdvertising::advertisement_type` ([#41](https://github.com/taks/esp32-nimble/pull/41))
- Fix compile error for esp32c6 ([#33](https://github.com/taks/esp32-nimble/pull/33))

## [0.2.2]
- Fix advertising regression in v0.2.1 ([#31](https://github.com/taks/esp32-nimble/pull/31))
- Added disconnect reason parameter to BLEServer::on_disconnect.

## [0.2.1]
- Added methods to set custom adv_data and scan_reponse. ([#25](https://github.com/taks/esp32-nimble/pull/25))
- Added deinit function and support reinitialize. ([#26](https://github.com/taks/esp32-nimble/pull/26))
- Added bond management functions.
- Changed BLEService.start to pub(crate). ([#27](https://github.com/taks/esp32-nimble/pull/27))
- Added Extended advertisement support.
