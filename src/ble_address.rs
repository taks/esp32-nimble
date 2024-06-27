use esp_idf_svc::sys::*;
use num_enum::TryFromPrimitive;

/// Bluetooth Device address type
#[derive(PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub enum BLEAddressType {
  Public = BLE_ADDR_PUBLIC as _,
  Random = BLE_ADDR_RANDOM as _,
  PublicID = BLE_ADDR_PUBLIC_ID as _,
  RandomID = BLE_ADDR_RANDOM_ID as _,
}

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct BLEAddress {
  pub(crate) value: ble_addr_t,
}

impl BLEAddress {
  pub fn new(val: [u8; 6], addr_type: BLEAddressType) -> Self {
    let mut ret = Self {
      value: ble_addr_t {
        val,
        type_: addr_type as _,
      },
    };
    ret.value.val.reverse();
    ret
  }

  pub fn from_str(input: &str, addr_type: BLEAddressType) -> Option<Self> {
    let mut val = [0u8; 6];

    let mut nth = 0;
    for byte in input.split(|c| c == ':' || c == '-') {
      if nth == 6 {
        return None;
      }

      val[nth] = u8::from_str_radix(byte, 16).ok()?;

      nth += 1;
    }

    if nth != 6 {
      return None;
    }

    Some(Self::new(val, addr_type))
  }

  /// Get the native representation of the address.
  pub fn val(&self) -> [u8; 6] {
    self.value.val
  }

  /// Get the address type.
  pub fn addr_type(&self) -> BLEAddressType {
    BLEAddressType::try_from(self.value.type_).unwrap()
  }
}

impl From<ble_addr_t> for BLEAddress {
  fn from(value: ble_addr_t) -> Self {
    Self { value }
  }
}

impl core::fmt::Display for BLEAddress {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(
      f,
      "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
      self.value.val[5],
      self.value.val[4],
      self.value.val[3],
      self.value.val[2],
      self.value.val[1],
      self.value.val[0]
    )
  }
}

impl core::fmt::Debug for BLEAddress {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    let type_str = match self.value.type_ as _ {
      BLE_ADDR_RANDOM => "(random)",
      BLE_ADDR_PUBLIC_ID => "(publicID)",
      BLE_ADDR_RANDOM_ID => "(randomID)",
      _ => "",
    };
    write!(f, "{self}{type_str}")
  }
}

impl PartialEq for BLEAddress {
  fn eq(&self, other: &Self) -> bool {
    self.value.val == other.value.val
  }
}

impl Eq for BLEAddress {}
