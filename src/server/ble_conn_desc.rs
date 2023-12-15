use crate::{ble, BLEAddress, BLEReturnCode};
use esp_idf_sys::ble_gap_conn_desc;

#[repr(transparent)]
pub struct BLEConnDesc(pub(crate) ble_gap_conn_desc);

impl BLEConnDesc {
  /// Gets the over-the-air address of the connected peer
  #[inline]
  pub fn address(&self) -> BLEAddress {
    BLEAddress::from(self.0.peer_ota_addr)
  }

  /// Gets the ID address of the connected peer
  #[inline]
  pub fn id_address(&self) -> BLEAddress {
    BLEAddress::from(self.0.peer_id_addr)
  }

  /// Gets the connection handle of the connected peer
  #[inline]
  pub fn conn_handle(&self) -> u16 {
    self.0.conn_handle
  }

  /// Gets the connection interval for this connection (in 1.25ms units)
  #[inline]
  pub fn interval(&self) -> u16 {
    self.0.conn_itvl
  }

  /// Gets the supervision timeout for this connection (in 10ms units)
  #[inline]
  pub fn timeout(&self) -> u16 {
    self.0.supervision_timeout
  }

  /// Gets the allowable latency for this connection (unit = number of intervals)
  #[inline]
  pub fn latency(&self) -> u16 {
    self.0.conn_latency
  }

  /// Gets the maximum transmission unit size for this connection (in bytes)
  #[inline]
  pub fn mtu(&self) -> u16 {
    unsafe { esp_idf_sys::ble_att_mtu(self.0.conn_handle) }
  }

  /// Check if we are connected to a bonded peer
  #[inline]
  pub fn bonded(&self) -> bool {
    self.0.sec_state.bonded() != 0
  }

  /// Check if the connection in encrypted
  #[inline]
  pub fn encrypted(&self) -> bool {
    self.0.sec_state.encrypted() != 0
  }

  /// Check if the the connection has been authenticated
  #[inline]
  pub fn authenticated(&self) -> bool {
    self.0.sec_state.authenticated() != 0
  }

  /// Gets the key size used to encrypt the connection
  #[inline]
  pub fn sec_key_size(&self) -> u32 {
    self.0.sec_state.key_size()
  }

  /// Retrieves the most-recently measured RSSI.
  /// A connection’s RSSI is updated whenever a data channel PDU is received.
  pub fn get_rssi(&self) -> Result<i8, BLEReturnCode> {
    let mut rssi: i8 = 0;
    unsafe {
      ble!(esp_idf_sys::ble_gap_conn_rssi(
        self.0.conn_handle,
        &mut rssi
      ))?;
    }
    Ok(rssi)
  }
}

impl core::fmt::Debug for BLEConnDesc {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("BLEConnDesc")
      .field("address", &self.address())
      .field("bonded", &self.bonded())
      .field("encrypted", &self.encrypted())
      .field("authenticated", &self.authenticated())
      .finish()
  }
}
