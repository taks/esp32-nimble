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
  pub fn from_le_bytes(val: [u8; 6], addr_type: BLEAddressType) -> Self {
    Self {
      value: ble_addr_t {
        val,
        type_: addr_type as _,
      },
    }
  }

  pub fn from_be_bytes(mut val: [u8; 6], addr_type: BLEAddressType) -> Self {
    val.reverse();
    Self::from_le_bytes(val, addr_type)
  }

  pub fn from_str(input: &str, addr_type: BLEAddressType) -> Option<Self> {
    let mut val = [0u8; 6];

    let mut nth = 0;
    for byte in input.split([':', '-']) {
      if nth == 6 {
        return None;
      }

      val[nth] = u8::from_str_radix(byte, 16).ok()?;

      nth += 1;
    }

    if nth != 6 {
      return None;
    }

    Some(Self::from_be_bytes(val, addr_type))
  }

  /// Get the native representation of the address.
  pub fn as_le_bytes(&self) -> [u8; 6] {
    self.value.val
  }

  pub fn as_be_bytes(&self) -> [u8; 6] {
    let mut bytes = self.value.val;
    bytes.reverse();
    bytes
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

impl From<BLEAddress> for ble_addr_t {
  fn from(value: BLEAddress) -> Self {
    value.value
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
