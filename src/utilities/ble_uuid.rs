// Originally: https://github.com/pulse-loop/bluedroid/blob/develop/src/utilities/ble_uuid.rs

use alloc::string::String;

/// A Bluetooth UUID.
#[derive(Copy, Clone)]
pub enum BleUuid {
  /// A 16-bit UUID.
  Uuid16(u16),
  /// A 32-bit UUID.
  Uuid32(u32),
  /// A 128-bit UUID.
  Uuid128([u8; 16]),
}

impl BleUuid {
  /// Creates a new [`BleUuid`] from a 16-bit integer.
  #[must_use]
  pub const fn from_uuid16(uuid: u16) -> Self {
    Self::Uuid16(uuid)
  }

  /// Creates a new [`BleUuid`] from a 32-bit integer.
  #[must_use]
  pub const fn from_uuid32(uuid: u32) -> Self {
    Self::Uuid32(uuid)
  }

  /// Creates a new [`BleUuid`] from a 16 byte array.
  #[must_use]
  pub const fn from_uuid128(uuid: [u8; 16]) -> Self {
    Self::Uuid128(uuid)
  }

  /// Creates a new [`BleUuid`] from a formatted string.
  ///
  /// # Panics
  ///
  /// Panics if the string contains invalid characters.
  pub const fn from_uuid128_string(uuid: &str) -> Result<Self, uuid::Error> {
    // Accepts the following formats:
    // "00000000-0000-0000-0000-000000000000"
    // "00000000000000000000000000000000"

    match uuid::Uuid::try_parse(uuid) {
      Ok(uuid) => Ok(Self::Uuid128(uuid.as_u128().to_le_bytes())),
      Err(err) => Err(err),
    }
  }

  #[must_use]
  pub(crate) fn as_uuid128_array(&self) -> [u8; 16] {
    let base_ble_uuid = [
      0xfb, 0x34, 0x9b, 0x5f, 0x80, 0x00, 0x00, 0x80, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00,
      0x00,
    ];

    match self {
      Self::Uuid16(uuid) => {
        let mut uuid128 = base_ble_uuid;

        let mut uuid_as_bytes: [u8; 2] = uuid.to_be_bytes();
        uuid_as_bytes.reverse();

        uuid128[12..=13].copy_from_slice(&uuid_as_bytes[..]);
        uuid128
      }
      Self::Uuid32(uuid) => {
        let mut uuid128 = base_ble_uuid;

        let mut uuid_as_bytes: [u8; 4] = uuid.to_be_bytes();
        uuid_as_bytes.reverse();

        uuid128[12..=15].copy_from_slice(&uuid_as_bytes[..]);
        uuid128
      }
      Self::Uuid128(uuid) => *uuid,
    }
  }
}

impl PartialEq for BleUuid {
  fn eq(&self, other: &Self) -> bool {
    self.as_uuid128_array() == other.as_uuid128_array()
  }
}

impl From<BleUuid> for esp_idf_sys::ble_uuid_any_t {
  #[allow(clippy::cast_possible_truncation)]
  fn from(val: BleUuid) -> Self {
    let mut result: Self = Self::default();

    match val {
      BleUuid::Uuid16(uuid) => {
        result.u.type_ = esp_idf_sys::BLE_UUID_TYPE_16 as _;
        result.u16_.value = uuid;
      }
      BleUuid::Uuid32(uuid) => {
        result.u.type_ = esp_idf_sys::BLE_UUID_TYPE_32 as _;
        result.u32_.value = uuid;
      }
      BleUuid::Uuid128(uuid) => {
        result.u.type_ = esp_idf_sys::BLE_UUID_TYPE_128 as _;
        result.u128_.value = uuid;
      }
    }

    result
  }
}

impl From<esp_idf_sys::ble_uuid_any_t> for BleUuid {
  fn from(uuid: esp_idf_sys::ble_uuid_any_t) -> Self {
    unsafe {
      match uuid.u.type_ as _ {
        esp_idf_sys::BLE_UUID_TYPE_16 => Self::Uuid16(uuid.u16_.value),
        esp_idf_sys::BLE_UUID_TYPE_32 => Self::Uuid32(uuid.u32_.value),
        esp_idf_sys::BLE_UUID_TYPE_128 => Self::Uuid128(uuid.u128_.value),
        // Never happens
        _ => unreachable!("Invalid UUID length"),
      }
    }
  }
}

impl core::fmt::Display for BleUuid {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::Uuid16(uuid) => write!(f, "0x{uuid:04x}"),
      Self::Uuid32(uuid) => write!(f, "0x{uuid:08x}"),
      Self::Uuid128(uuid) => {
        let mut uuid = *uuid;
        uuid.reverse();

        let mut uuid_str = String::new();

        for byte in &uuid {
          uuid_str.push_str(&alloc::format!("{byte:02x}"));
        }
        uuid_str.insert(8, '-');
        uuid_str.insert(13, '-');
        uuid_str.insert(18, '-');
        uuid_str.insert(23, '-');

        write!(f, "{uuid_str}")
      }
    }
  }
}

impl core::fmt::Debug for BleUuid {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "{self}")
  }
}

#[macro_export]
/// Parse Uuid128 from string literals at compile time.
macro_rules! uuid128 {
  ($uuid:expr) => {{
    $crate::utilities::BleUuid::Uuid128($crate::uuid_macro!($uuid).as_u128().to_le_bytes())
  }};
}
