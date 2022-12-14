#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct BLEReturnCode(pub u32);

impl BLEReturnCode {
  pub fn fail() -> Result<(), Self> {
    Self::convert(0xFFFF)
  }

  pub const fn from(error: u32) -> Option<Self> {
    if error == 0 {
      None
    } else {
      Some(Self(error))
    }
  }

  pub fn check_and_return<T>(error: u32, value: T) -> Result<T, Self> {
    match error {
      0 | esp_idf_sys::BLE_HS_EALREADY | esp_idf_sys::BLE_HS_EDONE => Ok(value),
      error => Err(Self(error)),
    }
  }
  pub fn convert(error: u32) -> Result<(), Self> {
    Self::check_and_return(error, ())
  }
}

impl core::fmt::Debug for BLEReturnCode {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match return_code_to_string(self.0) {
      Some(text) => write!(f, "{}", text)?,
      None => write!(f, "{}", self.0)?,
    };

    Ok(())
  }
}

pub fn return_code_to_string(rc: u32) -> Option<&'static str> {
  if rc < esp_idf_sys::BLE_HS_ERR_ATT_BASE {
    match rc {
      esp_idf_sys::BLE_HS_EINVAL => Some("One or more arguments are invalid."),
      esp_idf_sys::BLE_HS_EMSGSIZE => Some("The provided buffer is too small."),
      esp_idf_sys::BLE_HS_ENOTCONN => Some("No open connection with the specified handle."),
      esp_idf_sys::BLE_HS_ETIMEOUT => Some("Operation timed out."),
      esp_idf_sys::BLE_HS_EDONE => Some("Operation completed successfully."),
      esp_idf_sys::BLE_HS_ENOADDR => {
        Some("Operation requires an identity address but none configured.")
      }
      _ => None,
    }
  } else if rc < esp_idf_sys::BLE_HS_ERR_HCI_BASE {
    let rc_ = rc - esp_idf_sys::BLE_HS_ERR_ATT_BASE;
    match rc_ {
      esp_idf_sys::BLE_ATT_ERR_INSUFFICIENT_AUTHEN => {
        Some("The attribute requires authentication before it can be read or written.")
      }
      esp_idf_sys::BLE_ATT_ERR_ATTR_NOT_LONG => {
        Some("The attribute cannot be read or written using the Read Blob Request")
      }
      esp_idf_sys::BLE_ATT_ERR_UNLIKELY => {
        Some("The attribute request has encountered an error that was unlikely, could not be completed as requested.")
      }
      esp_idf_sys::BLE_ATT_ERR_INSUFFICIENT_ENC =>
      {
        Some("The attribute requires encryption before it can be read or written.")
      }
      _ => None,
    }
  } else {
    let rc_ = rc - esp_idf_sys::BLE_HS_ERR_HCI_BASE;
    match rc_ {
      esp_idf_sys::ble_error_codes_BLE_ERR_CONN_TERM_LOCAL => {
        Some("Connection Terminated By Local Host")
      }
      esp_idf_sys::ble_error_codes_BLE_ERR_CONN_ESTABLISHMENT => {
        Some("Connection Failed to be Established.")
      }
      _ => None,
    }
  }
}

macro_rules! ble {
  ($err:expr) => {{
    $crate::BLEReturnCode::convert($err as _)
  }};
}
pub(crate) use ble;
