use crate::{
  ble,
  ble_device::OWN_ADDR_TYPE,
  ble_error::return_code_to_string,
  utilities::{as_void_ptr, voidp_to_ref, ArcUnsafeCell, BleUuid},
  BLEAddress, BLEConnDesc, BLEDevice, BLEError, BLERemoteService, Signal,
};
use alloc::{boxed::Box, string::ToString, vec::Vec};
use core::{cell::UnsafeCell, ffi::c_void};
use esp_idf_svc::sys as esp_idf_sys;
use esp_idf_sys::*;

#[allow(clippy::type_complexity)]
pub(crate) struct BLEClientState {
  address: Option<BLEAddress>,
  conn_handle: u16,
  services: Option<Vec<BLERemoteService>>,
  signal: Signal<u32>,
  connect_timeout_ms: u32,
  ble_gap_conn_params: ble_gap_conn_params,
  on_passkey_request: Option<Box<dyn Fn() -> u32 + Send + Sync>>,
  on_confirm_pin: Option<Box<dyn Fn(u32) -> bool + Send + Sync>>,
  on_connect: Option<Box<dyn Fn(&mut BLEClient) + Send + Sync>>,
  on_disconnect: Option<Box<dyn Fn(i32) + Send + Sync>>,
}

pub struct BLEClient {
  state: ArcUnsafeCell<BLEClientState>,
}

impl BLEClient {
  pub fn new() -> Self {
    Self {
      state: ArcUnsafeCell::new(BLEClientState {
        address: None,
        conn_handle: esp_idf_sys::BLE_HS_CONN_HANDLE_NONE as _,
        services: None,
        connect_timeout_ms: 30000,
        ble_gap_conn_params: ble_gap_conn_params {
          scan_itvl: 16,
          scan_window: 16,
          itvl_min: (30 * 1000 / BLE_HCI_CONN_ITVL) as _,
          itvl_max: (50 * 1000 / BLE_HCI_CONN_ITVL) as _,
          latency: BLE_GAP_INITIAL_CONN_LATENCY as _,
          supervision_timeout: BLE_GAP_INITIAL_SUPERVISION_TIMEOUT as _,
          min_ce_len: BLE_GAP_INITIAL_CONN_MIN_CE_LEN as _,
          max_ce_len: BLE_GAP_INITIAL_CONN_MAX_CE_LEN as _,
        },
        signal: Signal::new(),
        on_passkey_request: None,
        on_confirm_pin: None,
        on_disconnect: None,
        on_connect: None,
      }),
    }
  }

  pub(crate) fn from_state(state: ArcUnsafeCell<BLEClientState>) -> Self {
    Self { state }
  }

  pub(crate) fn conn_handle(&self) -> u16 {
    self.state.conn_handle
  }

  pub fn on_passkey_request(
    &mut self,
    callback: impl Fn() -> u32 + Send + Sync + 'static,
  ) -> &mut Self {
    self.state.on_passkey_request = Some(Box::new(callback));
    self
  }

  pub fn on_confirm_pin(
    &mut self,
    callback: impl Fn(u32) -> bool + Send + Sync + 'static,
  ) -> &mut Self {
    self.state.on_confirm_pin = Some(Box::new(callback));
    self
  }

  pub fn on_connect(&mut self, callback: impl Fn(&mut Self) + Send + Sync + 'static) -> &mut Self {
    self.state.on_connect = Some(Box::new(callback));
    self
  }

  pub fn on_disconnect(&mut self, callback: impl Fn(i32) + Send + Sync + 'static) -> &mut Self {
    self.state.on_disconnect = Some(Box::new(callback));
    self
  }

  pub async fn connect(&mut self, addr: &BLEAddress) -> Result<(), BLEError> {
    unsafe {
      if esp_idf_sys::ble_gap_conn_find_by_addr(&addr.value, core::ptr::null_mut()) == 0 {
        ::log::warn!("A connection to {:?} already exists", addr);
        return BLEError::fail();
      }

      ble!(esp_idf_sys::ble_gap_connect(
        OWN_ADDR_TYPE as _,
        &addr.value,
        self.state.connect_timeout_ms as _,
        &self.state.ble_gap_conn_params,
        Some(Self::handle_gap_event),
        as_void_ptr(self),
      ))?;
    }

    ble!(self.state.signal.wait().await)?;
    self.state.address = Some(*addr);

    let client = UnsafeCell::new(self);
    unsafe {
      if let Some(callback) = &(*client.get()).state.on_connect {
        callback(*client.get());
      }
    }

    Ok(())
  }

  pub async fn secure_connection(&mut self) -> Result<(), BLEError> {
    unsafe {
      ble!(esp_idf_sys::ble_gap_security_initiate(
        self.state.conn_handle
      ))?;
    }
    ble!(self.state.signal.wait().await)?;

    ::log::info!("secure_connection: success");

    Ok(())
  }

  /// Disconnect from the peer.
  pub fn disconnect(&mut self) -> Result<(), BLEError> {
    self.disconnect_with_reason(esp_idf_sys::ble_error_codes_BLE_ERR_REM_USER_CONN_TERM as _)
  }

  /// Disconnect from the peer with optional reason.
  pub fn disconnect_with_reason(&mut self, reason: u8) -> Result<(), BLEError> {
    if !self.connected() {
      return Ok(());
    }

    unsafe {
      ble!(esp_idf_sys::ble_gap_terminate(
        self.state.conn_handle,
        reason
      ))
    }
  }

  pub fn connected(&self) -> bool {
    self.state.conn_handle != (esp_idf_sys::BLE_HS_CONN_HANDLE_NONE as _)
  }

  /// Set the connection parameters to use when connecting to a server.
  ///
  /// * `min_interval`: The minimum connection interval in 1.25ms units.
  /// * `max_interval`: The maximum connection interval in 1.25ms units.
  /// * `latency`: The number of packets allowed to skip (extends max interval).
  /// * `timeout`: The timeout time in 10ms units before disconnecting.
  /// * `scan_interval`: The scan interval to use when attempting to connect in 0.625ms units.
  /// * `scan_window`: The scan window to use when attempting to connect in 0.625ms units.
  pub fn set_connection_params(
    &mut self,
    min_interval: u16,
    max_interval: u16,
    latency: u16,
    timeout: u16,
    scan_interval: u16,
    scan_window: u16,
  ) {
    self.state.ble_gap_conn_params.scan_itvl = scan_interval;
    self.state.ble_gap_conn_params.scan_window = scan_window;
    self.state.ble_gap_conn_params.itvl_min = min_interval;
    self.state.ble_gap_conn_params.itvl_max = max_interval;
    self.state.ble_gap_conn_params.latency = latency;
    self.state.ble_gap_conn_params.supervision_timeout = timeout;
  }

  /// Request an Update the connection parameters:
  /// Can only be used after a connection has been established.
  ///
  /// * `min_interval`: The minimum connection interval in 1.25ms units.
  /// * `max_interval`: The maximum connection interval in 1.25ms units.
  /// * `latency`: The number of packets allowed to skip (extends max interval).
  /// * `timeout`: The timeout time in 10ms units before disconnecting.
  pub fn update_conn_params(
    &mut self,
    min_interval: u16,
    max_interval: u16,
    latency: u16,
    timeout: u16,
  ) -> Result<(), BLEError> {
    let params = esp_idf_sys::ble_gap_upd_params {
      itvl_min: min_interval,
      itvl_max: max_interval,
      latency,
      supervision_timeout: timeout,
      min_ce_len: esp_idf_sys::BLE_GAP_INITIAL_CONN_MIN_CE_LEN as _,
      max_ce_len: esp_idf_sys::BLE_GAP_INITIAL_CONN_MAX_CE_LEN as _,
    };

    unsafe {
      ble!(esp_idf_sys::ble_gap_update_params(
        self.state.conn_handle,
        &params
      ))
    }
  }

  pub fn desc(&self) -> Result<BLEConnDesc, crate::BLEError> {
    crate::utilities::ble_gap_conn_find(self.conn_handle())
  }

  /// Retrieves the most-recently measured RSSI.
  /// A connection’s RSSI is updated whenever a data channel PDU is received.
  pub fn get_rssi(&self) -> Result<i8, BLEError> {
    let mut rssi: i8 = 0;
    unsafe {
      ble!(esp_idf_sys::ble_gap_conn_rssi(
        self.conn_handle(),
        &mut rssi
      ))?;
    }
    Ok(rssi)
  }

  pub async fn get_services(
    &mut self,
  ) -> Result<core::slice::IterMut<'_, BLERemoteService>, BLEError> {
    if self.state.services.is_none() {
      self.state.services = Some(Vec::new());
      unsafe {
        esp_idf_sys::ble_gattc_disc_all_svcs(
          self.state.conn_handle,
          Some(Self::service_discovered_cb),
          as_void_ptr(self),
        );
      }
      ble!(self.state.signal.wait().await)?;
    }

    Ok(self.state.services.as_mut().unwrap().iter_mut())
  }

  pub async fn get_service(&mut self, uuid: BleUuid) -> Result<&mut BLERemoteService, BLEError> {
    let mut iter = self.get_services().await?;
    iter
      .find(|x| x.uuid() == uuid)
      .ok_or_else(|| BLEError::fail().unwrap_err())
  }

  extern "C" fn handle_gap_event(event: *mut esp_idf_sys::ble_gap_event, arg: *mut c_void) -> i32 {
    let event = unsafe { &*event };
    let client = unsafe { voidp_to_ref::<Self>(arg) };

    match event.type_ as _ {
      BLE_GAP_EVENT_CONNECT => {
        let connect = unsafe { &event.__bindgen_anon_1.connect };

        if connect.status == 0 {
          client.state.conn_handle = connect.conn_handle;

          let rc =
            unsafe { ble_gattc_exchange_mtu(connect.conn_handle, None, core::ptr::null_mut()) };

          if rc != 0 {
            client.state.signal.signal(rc as _);
          }
        } else {
          ::log::info!("connect_status {}", connect.status);
          client.state.conn_handle = esp_idf_sys::BLE_HS_CONN_HANDLE_NONE as _;
          client.state.signal.signal(connect.status as _);
        }
      }
      BLE_GAP_EVENT_DISCONNECT => {
        let disconnect = unsafe { &event.__bindgen_anon_1.disconnect };
        if client.state.conn_handle != disconnect.conn.conn_handle {
          return 0;
        }
        client.state.conn_handle = esp_idf_sys::BLE_HS_CONN_HANDLE_NONE as _;

        ::log::info!(
          "Disconnected: {}",
          return_code_to_string(disconnect.reason as _)
            .map_or_else(|| disconnect.reason.to_string(), |x| x.to_string())
        );

        if let Some(callback) = &client.state.on_disconnect {
          callback(disconnect.reason);
        }
      }
      BLE_GAP_EVENT_ENC_CHANGE => {
        let enc_change = unsafe { &event.__bindgen_anon_1.enc_change };
        if client.state.conn_handle != enc_change.conn_handle {
          return 0;
        }

        if enc_change.status
          == ((BLE_HS_ERR_HCI_BASE + ble_error_codes_BLE_ERR_PINKEY_MISSING) as _)
        {
          let desc = crate::utilities::ble_gap_conn_find(enc_change.conn_handle).unwrap();
          unsafe { esp_idf_sys::ble_store_util_delete_peer(&desc.0.peer_id_addr) };
        }

        client.state.signal.signal(enc_change.status as _);
      }
      BLE_GAP_EVENT_MTU => {
        let mtu = unsafe { &event.__bindgen_anon_1.mtu };
        if client.state.conn_handle != mtu.conn_handle {
          return 0;
        }
        ::log::info!(
          "mtu update event; conn_handle={} mtu={}",
          mtu.conn_handle,
          mtu.value
        );
        client.state.signal.signal(0);
      }
      BLE_GAP_EVENT_NOTIFY_RX => {
        let notify_rx = unsafe { &event.__bindgen_anon_1.notify_rx };
        if client.state.conn_handle != notify_rx.conn_handle {
          return 0;
        }

        if let Some(services) = &mut client.state.services {
          for service in services {
            if service.state.end_handle < notify_rx.attr_handle {
              continue;
            }

            if let Some(characteristics) = &mut service.state.characteristics {
              for characteristic in characteristics {
                if characteristic.state().handle == notify_rx.attr_handle {
                  unsafe {
                    characteristic.notify(notify_rx.om);
                  }
                  return 0;
                }
              }
            }
          }
        }
      }
      BLE_GAP_EVENT_CONN_UPDATE | BLE_GAP_EVENT_L2CAP_UPDATE_REQ => {
        let conn_update_req = unsafe { &event.__bindgen_anon_1.conn_update_req };
        if client.state.conn_handle != conn_update_req.conn_handle {
          return 0;
        }
        unsafe {
          ::log::debug!("Peer requesting to update connection parameters");
          ::log::debug!(
            "MinInterval: {}, MaxInterval: {}, Latency: {}, Timeout: {}",
            (*conn_update_req.peer_params).itvl_min,
            (*conn_update_req.peer_params).itvl_max,
            (*conn_update_req.peer_params).latency,
            (*conn_update_req.peer_params).supervision_timeout
          );
        }
      }
      BLE_GAP_EVENT_PASSKEY_ACTION => {
        let passkey = unsafe { &event.__bindgen_anon_1.passkey };
        if client.state.conn_handle != passkey.conn_handle {
          return 0;
        }
        let mut pkey = esp_idf_sys::ble_sm_io {
          action: passkey.params.action,
          ..Default::default()
        };
        match passkey.params.action as _ {
          esp_idf_sys::BLE_SM_IOACT_DISP => {
            pkey.__bindgen_anon_1.passkey = BLEDevice::take().security().get_passkey();
            let rc = unsafe { esp_idf_sys::ble_sm_inject_io(passkey.conn_handle, &mut pkey) };
            ::log::debug!("BLE_SM_IOACT_DISP; ble_sm_inject_io result: {}", rc);
          }
          esp_idf_sys::BLE_SM_IOACT_NUMCMP => {
            if let Some(callback) = &client.state.on_confirm_pin {
              pkey.__bindgen_anon_1.numcmp_accept = callback(passkey.params.numcmp) as _;
            } else {
              ::log::warn!("on_passkey_request is not setted");
            }
            let rc = unsafe { esp_idf_sys::ble_sm_inject_io(passkey.conn_handle, &mut pkey) };
            ::log::debug!("BLE_SM_IOACT_NUMCMP; ble_sm_inject_io result: {}", rc);
          }
          esp_idf_sys::BLE_SM_IOACT_INPUT => {
            if let Some(callback) = &client.state.on_passkey_request {
              pkey.__bindgen_anon_1.passkey = callback();
            } else {
              ::log::warn!("on_passkey_request is not setted");
            }
            let rc = unsafe { esp_idf_sys::ble_sm_inject_io(passkey.conn_handle, &mut pkey) };
            ::log::debug!("BLE_SM_IOACT_INPUT; ble_sm_inject_io result: {}", rc);
          }
          esp_idf_sys::BLE_SM_IOACT_NONE => {
            ::log::debug!("BLE_SM_IOACT_NONE; No passkey action required");
          }
          action => {
            todo!("implementation required: {}", action);
          }
        }
      }
      _ => {
        ::log::warn!("unhandled event: {}", event.type_);
      }
    }
    0
  }

  extern "C" fn service_discovered_cb(
    conn_handle: u16,
    error: *const esp_idf_sys::ble_gatt_error,
    service: *const esp_idf_sys::ble_gatt_svc,
    arg: *mut c_void,
  ) -> i32 {
    let client = unsafe { voidp_to_ref::<Self>(arg) };
    if client.state.conn_handle != conn_handle {
      return 0;
    }

    let error = unsafe { &*error };
    let service = unsafe { &*service };

    if error.status == 0 {
      // Found a service - add it to the vector
      let service = BLERemoteService::new(ArcUnsafeCell::downgrade(&client.state), service);
      client.state.services.as_mut().unwrap().push(service);
      return 0;
    }

    let ret = if error.status == (esp_idf_sys::BLE_HS_EDONE as _) {
      0
    } else {
      error.status as _
    };

    client.state.signal.signal(ret);
    ret as _
  }
}
