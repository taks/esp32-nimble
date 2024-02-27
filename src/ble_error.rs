use core::num::NonZeroI32;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct BLEError(NonZeroI32);

impl BLEError {
  pub fn fail() -> Result<(), Self> {
    Self::convert(0xFFFF)
  }

  pub const fn from_non_zero(error: NonZeroI32) -> Self {
    Self(error)
  }

  pub fn check_and_return<T>(error: u32, value: T) -> Result<T, Self> {
    match error {
      0 | esp_idf_sys::BLE_HS_EALREADY | esp_idf_sys::BLE_HS_EDONE => Ok(value),
      error => Err(Self(unsafe { NonZeroI32::new_unchecked(error as _) })),
    }
  }

  pub const fn convert(error: u32) -> Result<(), Self> {
    match error {
      0 | esp_idf_sys::BLE_HS_EALREADY | esp_idf_sys::BLE_HS_EDONE => Ok(()),
      error => Err(Self(unsafe { NonZeroI32::new_unchecked(error as _) })),
    }
  }

  pub fn code(&self) -> u32 {
    self.0.get() as _
  }
}

impl core::fmt::Debug for BLEError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match return_code_to_string(self.0.get()) {
      Some(text) => write!(f, "{text}")?,
      None => write!(f, "0x{:X}", self.0)?,
    };

    Ok(())
  }
}

pub fn return_code_to_string(rc: i32) -> Option<&'static str> {
  let rc = rc as u32;
  if rc < esp_idf_sys::BLE_HS_ERR_ATT_BASE {
    match rc {
      esp_idf_sys::BLE_HS_EALREADY => Some("Operation already in progress or completed."),
      esp_idf_sys::BLE_HS_EINVAL => Some("One or more arguments are invalid."),
      esp_idf_sys::BLE_HS_EMSGSIZE => Some("The provided buffer is too small."),
      esp_idf_sys::BLE_HS_ENOENT => Some("No entry matching the specified criteria."),
      esp_idf_sys::BLE_HS_ENOMEM => Some("Operation failed due to resource exhaustion."),
      esp_idf_sys::BLE_HS_ENOTCONN => Some("No open connection with the specified handle."),
      esp_idf_sys::BLE_HS_ENOTSUP => Some("Operation disabled at compile time."),
      esp_idf_sys::BLE_HS_EAPP => Some("Application callback behaved unexpectedly."),
      esp_idf_sys::BLE_HS_EBADDATA => Some("Command from peer is invalid."),
      esp_idf_sys::BLE_HS_EOS => Some("Mynewt OS error."),
      esp_idf_sys::BLE_HS_ECONTROLLER => Some("Event from controller is invalid."),
      esp_idf_sys::BLE_HS_ETIMEOUT => Some("Operation timed out."),
      esp_idf_sys::BLE_HS_EDONE => Some("Operation completed successfully."),
      esp_idf_sys::BLE_HS_EBUSY => Some("Operation cannot be performed until procedure completes."),
      esp_idf_sys::BLE_HS_EREJECT => Some("Peer rejected a connection parameter update request."),
      esp_idf_sys::BLE_HS_EUNKNOWN => Some("Unexpected failure; catch all."),
      esp_idf_sys::BLE_HS_EROLE => {
        Some("Operation requires different role (e.g., central vs. peripheral).")
      }
      esp_idf_sys::BLE_HS_ETIMEOUT_HCI => Some("HCI request timed out; controller unresponsive."),
      esp_idf_sys::BLE_HS_ENOMEM_EVT => Some(
        "Controller failed to send event due to memory exhaustion (combined host-controller only).",
      ),
      esp_idf_sys::BLE_HS_ENOADDR => {
        Some("Operation requires an identity address but none configured.")
      }
      esp_idf_sys::BLE_HS_ENOTSYNCED => {
        Some("Attempt to use the host before it is synced with controller.")
      }
      esp_idf_sys::BLE_HS_EAUTHEN => Some("Insufficient authentication."),
      esp_idf_sys::BLE_HS_EAUTHOR => Some("Insufficient authorization."),
      esp_idf_sys::BLE_HS_EENCRYPT => Some("Insufficient encryption level."),
      esp_idf_sys::BLE_HS_EENCRYPT_KEY_SZ => Some("Insufficient key size"),
      esp_idf_sys::BLE_HS_ESTORE_CAP => Some("Storage at capacity."),
      esp_idf_sys::BLE_HS_ESTORE_FAIL => Some("Storage IO error."),
      _ => None,
    }
  } else if rc < esp_idf_sys::BLE_HS_ERR_HCI_BASE {
    let rc_ = rc - esp_idf_sys::BLE_HS_ERR_ATT_BASE;
    match rc_ {
      esp_idf_sys::BLE_ATT_ERR_INVALID_HANDLE => Some("The attribute handle given was not valid on this server."),
      esp_idf_sys::BLE_ATT_ERR_READ_NOT_PERMITTED  => Some("The attribute cannot be read."),
      esp_idf_sys::BLE_ATT_ERR_WRITE_NOT_PERMITTED => Some("The attribute cannot be written."),
      esp_idf_sys::BLE_ATT_ERR_INVALID_PDU => Some("The attribute PDU was invalid."),
      esp_idf_sys::BLE_ATT_ERR_INSUFFICIENT_AUTHEN => Some("The attribute requires authentication before it can be read or written."),
      esp_idf_sys::BLE_ATT_ERR_REQ_NOT_SUPPORTED => Some("Attribute server does not support the request received from the client."),
      esp_idf_sys::BLE_ATT_ERR_INVALID_OFFSET => Some("Offset specified was past the end of the attribute."),
      esp_idf_sys::BLE_ATT_ERR_INSUFFICIENT_AUTHOR => Some("The attribute requires authorization before it can be read or written."),
      esp_idf_sys::BLE_ATT_ERR_PREPARE_QUEUE_FULL => Some("Too many prepare writes have been queued."),
      esp_idf_sys::BLE_ATT_ERR_ATTR_NOT_FOUND => Some("No attribute found within the given attribute handle range."),
      esp_idf_sys::BLE_ATT_ERR_ATTR_NOT_LONG => Some("The attribute cannot be read or written using the Read Blob Request."),
      esp_idf_sys::BLE_ATT_ERR_INSUFFICIENT_KEY_SZ => Some("The Encryption Key Size used for encrypting this link is insufficient."),
      esp_idf_sys::BLE_ATT_ERR_INVALID_ATTR_VALUE_LEN => Some("The attribute value length is invalid for the operation."),
      esp_idf_sys::BLE_ATT_ERR_UNLIKELY => Some("The attribute request has encountered an error that was unlikely, could not be completed as requested."),
      esp_idf_sys::BLE_ATT_ERR_INSUFFICIENT_ENC => Some("The attribute requires encryption before it can be read or written."),
      esp_idf_sys::BLE_ATT_ERR_UNSUPPORTED_GROUP => Some("The attribute type is not a supported grouping attribute as defined by a higher layer specification."),
      esp_idf_sys::BLE_ATT_ERR_INSUFFICIENT_RES => Some("Insufficient Resources to complete the request."),
      _ => None,
    }
  } else if rc < esp_idf_sys::BLE_HS_ERR_L2C_BASE {
    let rc_ = rc - esp_idf_sys::BLE_HS_ERR_HCI_BASE;
    match rc_ {
      esp_idf_sys::BLE_L2CAP_SIG_ERR_CMD_NOT_UNDERSTOOD => {
        Some("Invalid or unsupported incoming L2CAP sig command.")
      }
      esp_idf_sys::BLE_L2CAP_SIG_ERR_MTU_EXCEEDED => Some("Incoming packet too large."),
      esp_idf_sys::BLE_L2CAP_SIG_ERR_INVALID_CID => Some("No channel with specified ID."),
      _ => None,
    }
  } else if rc < esp_idf_sys::BLE_HS_ERR_SM_US_BASE {
    let rc_ = rc - esp_idf_sys::BLE_HS_ERR_L2C_BASE;
    match rc_ {
      esp_idf_sys::BLE_SM_ERR_PASSKEY => {
        Some("The user input of passkey failed, for example, the user cancelled the operation.")
      }
      esp_idf_sys::BLE_SM_ERR_OOB => Some("The OOB data is not available."),
      esp_idf_sys::BLE_SM_ERR_AUTHREQ => Some("The pairing procedure cannot be performed as authentication requirements cannot be met due to IO capabilities of one or both devices."),
      esp_idf_sys::BLE_SM_ERR_CONFIRM_MISMATCH => Some("The confirm value does not match the calculated compare value."),
      esp_idf_sys::BLE_SM_ERR_PAIR_NOT_SUPP => Some("Pairing is not supported by the device."),
      esp_idf_sys::BLE_SM_ERR_ENC_KEY_SZ => Some("The resultant encryption key size is insufficient for the security requirements of this device."),
      esp_idf_sys::BLE_SM_ERR_CMD_NOT_SUPP => Some("The SMP command received is not supported on this device."),
      esp_idf_sys::BLE_SM_ERR_UNSPECIFIED => Some("Pairing failed due to an unspecified reason."),
      esp_idf_sys::BLE_SM_ERR_REPEATED => Some("Pairing or authentication procedure is disallowed because too little time has elapsed since last pairing request or security request."),
      esp_idf_sys::BLE_SM_ERR_INVAL => Some("The Invalid Parameters error code indicates that the command length is invalid or that a parameter is outside of the specified range."),
      esp_idf_sys::BLE_SM_ERR_DHKEY => Some("Indicates to the remote device that the DHKey Check value received doesn’t match the one calculated by the local device."),
      esp_idf_sys::BLE_SM_ERR_NUMCMP => Some("Indicates that the confirm values in the numeric comparison protocol do not match."),
      esp_idf_sys::BLE_SM_ERR_ALREADY => Some("Indicates that the pairing over the LE transport failed due to a Pairing Request sent over the BR/EDR transport in process."),
      esp_idf_sys::BLE_SM_ERR_CROSS_TRANS => Some("Indicates that the BR/EDR Link Key generated on the BR/EDR transport cannot be used to derive and distribute keys for the LE transport."),
      _ => None,
    }
  } else if rc < esp_idf_sys::BLE_HS_ERR_SM_PEER_BASE {
    let rc_ = rc - esp_idf_sys::BLE_HS_ERR_SM_US_BASE;
    match rc_ {
      esp_idf_sys::BLE_SM_ERR_PASSKEY => Some("The user input of passkey failed, for example, the user cancelled the operation."),
      esp_idf_sys::BLE_SM_ERR_OOB => Some("The OOB data is not available."),
      esp_idf_sys::BLE_SM_ERR_AUTHREQ => Some("The pairing procedure cannot be performed as authentication requirements cannot be met due to IO capabilities of one or both devices."),
      esp_idf_sys::BLE_SM_ERR_CONFIRM_MISMATCH => Some("The confirm value does not match the calculated compare value."),
      esp_idf_sys::BLE_SM_ERR_PAIR_NOT_SUPP => Some("Pairing is not supported by the device."),
      esp_idf_sys::BLE_SM_ERR_ENC_KEY_SZ => Some("The resultant encryption key size is insufficient for the security requirements of this device."),
      esp_idf_sys::BLE_SM_ERR_CMD_NOT_SUPP => Some("The SMP command received is not supported on this device."),
      esp_idf_sys::BLE_SM_ERR_UNSPECIFIED => Some("Pairing failed due to an unspecified reason."),
      esp_idf_sys::BLE_SM_ERR_REPEATED => Some("Pairing or authentication procedure is disallowed because too little time has elapsed since last pairing request or security request."),
      esp_idf_sys::BLE_SM_ERR_INVAL => Some("The Invalid Parameters error code indicates that the command length is invalid or that a parameter is outside of the specified range."),
      esp_idf_sys::BLE_SM_ERR_DHKEY => Some("Indicates to the remote device that the DHKey Check value received doesn’t match the one calculated by the local device."),
      esp_idf_sys::BLE_SM_ERR_NUMCMP => Some("Indicates that the confirm values in the numeric comparison protocol do not match."),
      esp_idf_sys::BLE_SM_ERR_ALREADY => Some("Indicates that the pairing over the LE transport failed due to a Pairing Request sent over the BR/EDR transport in process."),
      esp_idf_sys::BLE_SM_ERR_CROSS_TRANS => Some("Indicates that the BR/EDR Link Key generated on the BR/EDR transport cannot be used to derive and distribute keys for the LE transport."),
      _ => None,
    }
  } else {
    //TODO: esp_idf_sys::BLE_HS_ERR_HW_BASE
    let rc_ = rc - esp_idf_sys::BLE_HS_ERR_SM_PEER_BASE;
    match rc_ {
      esp_idf_sys::ble_error_codes_BLE_ERR_UNKNOWN_HCI_CMD => Some("Unknown HCI Command"),
      esp_idf_sys::ble_error_codes_BLE_ERR_UNK_CONN_ID => Some("Unknown Connection Identifier"),
      esp_idf_sys::ble_error_codes_BLE_ERR_AUTH_FAIL => Some("Authentication Failure"),
      esp_idf_sys::ble_error_codes_BLE_ERR_INV_HCI_CMD_PARMS => {
        Some("Invalid HCI Command Parameters")
      }
      esp_idf_sys::ble_error_codes_BLE_ERR_REM_USER_CONN_TERM => {
        Some("Remote User Terminated Connection")
      }
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
    $crate::BLEError::convert($err as _)
  }};
}

pub(crate) use ble;
