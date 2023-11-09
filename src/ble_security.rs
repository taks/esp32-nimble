use crate::enums;
pub struct BLESecurity {
  passkey: u32,
}

impl BLESecurity {
  pub(crate) fn new() -> Self {
    Self { passkey: 0 }
  }

  pub fn set_auth(&mut self, auth_req: enums::AuthReq) -> &mut Self {
    unsafe {
      esp_idf_sys::ble_hs_cfg.set_sm_bonding(auth_req.contains(enums::AuthReq::Bond) as _);
      esp_idf_sys::ble_hs_cfg.set_sm_mitm(auth_req.contains(enums::AuthReq::Mitm) as _);
      esp_idf_sys::ble_hs_cfg.set_sm_sc(auth_req.contains(enums::AuthReq::Sc) as _);
    }

    self
  }

  pub fn get_passkey(&self) -> u32 {
    self.passkey
  }

  pub fn set_passkey(&mut self, passkey: u32) -> &mut Self {
    self.passkey = passkey;
    self
  }

  pub fn set_io_cap(&mut self, iocap: enums::SecurityIOCap) -> &mut Self {
    unsafe { esp_idf_sys::ble_hs_cfg.sm_io_cap = iocap as _ };
    self
  }
}
