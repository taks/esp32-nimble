use crate::BLEAddress;
use crate::enums::*;
use esp_idf_svc::sys;

#[cfg(esp_idf_bt_nimble_ext_adv)]
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct BLEAdvertisedDevice(sys::ble_gap_ext_disc_desc);

#[cfg(not(esp_idf_bt_nimble_ext_adv))]
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct BLEAdvertisedDevice(sys::ble_gap_disc_desc);

impl BLEAdvertisedDevice {
  /// Get the address of the advertising device.
  pub fn addr(&self) -> BLEAddress {
    self.0.addr.into()
  }

  /// Get the advertisement type.
  pub fn adv_type(&self) -> AdvType {
    #[cfg(esp_idf_bt_nimble_ext_adv)]
    {
      if (self.0.props & (sys::BLE_HCI_ADV_LEGACY_MASK as u8)) != 0 {
        AdvType::from_event_type(self.0.legacy_event_type)
      } else {
        AdvType::Extended(self.0.props)
      }
    }

    #[cfg(not(esp_idf_bt_nimble_ext_adv))]
    {
      AdvType::from_event_type(self.0.event_type)
    }
  }

  pub fn rssi(&self) -> i8 {
    self.0.rssi
  }

  #[cfg(esp_idf_bt_nimble_ext_adv)]
  /// Get the set ID of the extended advertisement.
  pub fn sid(&self) -> u8 {
    self.0.sid
  }

  #[cfg(esp_idf_bt_nimble_ext_adv)]
  /// Get the primary PHY used by this advertisement.
  pub fn prim_phy(&self) -> PrimPhy {
    PrimPhy::try_from(self.0.prim_phy).unwrap()
  }

  #[cfg(esp_idf_bt_nimble_ext_adv)]
  /// Get the secondary PHY used by this advertisement.
  pub fn sec_phy(&self) -> Option<SecPhy> {
    SecPhy::try_from(self.0.sec_phy).ok()
  }

  #[cfg(esp_idf_bt_nimble_ext_adv)]
  /// Get the periodic interval of the advertisement.
  pub fn periodic_itvl(&self) -> u16 {
    self.0.periodic_adv_itvl
  }
}

impl core::fmt::Debug for BLEAdvertisedDevice {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    #[cfg(esp_idf_bt_nimble_ext_adv)]
    {
      f.debug_struct("BLEAdvertisedDevice")
        .field("addr", &self.addr())
        .field("adv_type", &self.adv_type())
        .field("rssi", &self.rssi())
        .field("sid", &self.sid())
        .field("prim_phy", &self.prim_phy())
        .field("sec_phy", &self.sec_phy())
        .field("periodic_itvl", &self.periodic_itvl())
        .finish()
    }
    #[cfg(not(esp_idf_bt_nimble_ext_adv))]
    {
      f.debug_struct("BLEAdvertisedDevice")
        .field("addr", &self.addr())
        .field("adv_type", &self.adv_type())
        .field("rssi", &self.rssi())
        .finish()
    }
  }
}
