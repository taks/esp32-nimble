use crate::enums;

pub struct BLESecurity {
  passkey: u32,
}

impl BLESecurity {
  pub(crate) fn new() -> Self {
    Self { passkey: 0 }
  }

  /// Set the authorization mode for this device.
  pub fn set_auth(&mut self, auth_req: enums::AuthReq) -> &mut Self {
    unsafe {
      esp_idf_sys::ble_hs_cfg.set_sm_bonding(auth_req.contains(enums::AuthReq::Bond) as _);
      esp_idf_sys::ble_hs_cfg.set_sm_mitm(auth_req.contains(enums::AuthReq::Mitm) as _);
      esp_idf_sys::ble_hs_cfg.set_sm_sc(auth_req.contains(enums::AuthReq::Sc) as _);
    }

    self
  }

  /// Get the current passkey used for pairing.
  pub fn get_passkey(&self) -> u32 {
    self.passkey
  }

  /// Set the passkey the server will ask for when pairing.
  /// * The passkey will always be exactly 6 digits. Setting the passkey to 1234
  /// will require the user to provide '001234'
  /// * a dynamic passkey can also be set by [`crate::BLEServer::on_passkey_request`]
  pub fn set_passkey(&mut self, passkey: u32) -> &mut Self {
    debug_assert!(
      passkey <= 999999,
      "passkey must be between 000000..=999999 inclusive"
    );
    self.passkey = passkey;
    self
  }

  /// Set the Input/Output capabilities of this device.
  pub fn set_io_cap(&mut self, iocap: enums::SecurityIOCap) -> &mut Self {
    unsafe { esp_idf_sys::ble_hs_cfg.sm_io_cap = iocap as _ };
    self
  }

  /// If we are the initiator of the security procedure this sets the keys we will distribute.
  pub fn set_security_init_key(&mut self, init_key: enums::PairKeyDist) -> &mut Self {
    unsafe { esp_idf_sys::ble_hs_cfg.sm_our_key_dist = init_key.bits() };
    self
  }

  /// Set the keys we are willing to accept during pairing.
  pub fn set_security_resp_key(&mut self, resp_key: enums::PairKeyDist) -> &mut Self {
    unsafe { esp_idf_sys::ble_hs_cfg.sm_their_key_dist = resp_key.bits() };
    self
  }

  /// Set up for pairing in RPA(Resolvable Private Address).
  ///
  /// ( see: https://github.com/taks/esp32-nimble/issues/24 )
  pub fn resolve_rpa(&mut self) -> &mut Self {
    self
      .set_security_init_key(enums::PairKeyDist::ENC | enums::PairKeyDist::ID)
      .set_security_resp_key(enums::PairKeyDist::ENC | enums::PairKeyDist::ID)
  }
}
