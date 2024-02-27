use crate::{
  ble,
  utilities::{ble_gap_conn_find, extend_lifetime_mut, mutex::Mutex, BleUuid},
  BLECharacteristic, BLEConnDesc, BLEDevice, BLEReturnCode, BLEService, NimbleProperties, NotifyTx,
};
use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::{
  cell::UnsafeCell,
  ffi::{c_int, c_void},
};

const BLE_HS_CONN_HANDLE_NONE: u16 = esp_idf_sys::BLE_HS_CONN_HANDLE_NONE as _;

#[allow(clippy::type_complexity)]
pub struct BLEServer {
  pub(crate) started: bool,
  advertise_on_disconnect: bool,
  services: Vec<Arc<Mutex<BLEService>>>,
  notify_characteristic: Vec<&'static mut BLECharacteristic>,
  connections: Vec<u16>,
  indicate_wait: [u16; esp_idf_sys::CONFIG_BT_NIMBLE_MAX_CONNECTIONS as _],

  on_connect: Option<Box<dyn FnMut(&mut Self, &BLEConnDesc) + Send + Sync>>,
  on_disconnect: Option<Box<dyn FnMut(&BLEConnDesc, c_int) + Send + Sync>>,
  on_passkey_request: Option<Box<dyn Fn() -> u32 + Send + Sync>>,
  on_confirm_pin: Option<Box<dyn Fn(u32) -> bool + Send + Sync>>,
  on_authentication_complete: Option<Box<dyn Fn(&BLEConnDesc, c_int) + Send + Sync>>,
}

impl BLEServer {
  pub(crate) fn new() -> Self {
    Self {
      started: false,
      advertise_on_disconnect: true,
      services: Vec::new(),
      notify_characteristic: Vec::new(),
      connections: Vec::new(),
      indicate_wait: [BLE_HS_CONN_HANDLE_NONE; esp_idf_sys::CONFIG_BT_NIMBLE_MAX_CONNECTIONS as _],
      on_connect: None,
      on_disconnect: None,
      on_passkey_request: None,
      on_confirm_pin: None,
      on_authentication_complete: None,
    }
  }

  pub fn on_connect(
    &mut self,
    callback: impl FnMut(&mut Self, &BLEConnDesc) + Send + Sync + 'static,
  ) -> &mut Self {
    self.on_connect = Some(Box::new(callback));
    self
  }

  /// Handle a client disconnection.
  /// * callback first parameter: A reference to a `esp_idf_sys::ble_gap_conn_desc` instance with information about the peer connection parameters.
  /// * callback second parameter: The reason code for the disconnection.
  pub fn on_disconnect(
    &mut self,
    callback: impl FnMut(&BLEConnDesc, c_int) + Send + Sync + 'static,
  ) -> &mut Self {
    self.on_disconnect = Some(Box::new(callback));
    self
  }

  /// Set a callback fn for generating a passkey if required by the connection
  /// * The passkey will always be exactly 6 digits. Setting the passkey to 1234
  /// will require the user to provide '001234'
  /// * a static passkey can also be set by [`crate::BLESecurity::set_passkey`]
  pub fn on_passkey_request(
    &mut self,
    callback: impl Fn() -> u32 + Send + Sync + 'static,
  ) -> &mut Self {
    if cfg!(debug_assertions) {
      self.on_passkey_request = Some(Box::new(move || {
        let passkey = callback();
        debug_assert!(
          passkey <= 999999,
          "passkey must be between 000000..=999999 inclusive"
        );
        passkey
      }));
    } else {
      self.on_passkey_request = Some(Box::new(callback));
    }

    self
  }

  pub fn on_confirm_pin(
    &mut self,
    callback: impl Fn(u32) -> bool + Send + Sync + 'static,
  ) -> &mut Self {
    self.on_confirm_pin = Some(Box::new(callback));
    self
  }

  /// The callback function is called when the pairing procedure is complete.
  /// * callback first parameter: A reference to a `BLEConnDesc` instance.
  /// * callback second parameter: Indicates the result of the encryption state change attempt;
  /// o 0: the encrypted state was successfully updated;
  /// o BLE host error code: the encryption state change attempt failed for the specified reason.
  pub fn on_authentication_complete(
    &mut self,
    callback: impl Fn(&BLEConnDesc, c_int) + Send + Sync + 'static,
  ) -> &mut Self {
    self.on_authentication_complete = Some(Box::new(callback));
    self
  }

  pub fn start(&mut self) -> Result<(), BLEReturnCode> {
    if self.started {
      return Ok(());
    }

    unsafe {
      esp_idf_sys::ble_gatts_reset();
      esp_idf_sys::ble_svc_gap_init();
      esp_idf_sys::ble_svc_gatt_init();
    }

    for svc in &mut self.services {
      svc.lock().start()?;
    }

    unsafe {
      ble!(esp_idf_sys::ble_gatts_start())?;

      for svc in &self.services {
        let mut svc = svc.lock();
        ble!(esp_idf_sys::ble_gatts_find_svc(
          &svc.uuid.u,
          &mut svc.handle
        ))?;

        for chr in &svc.characteristics {
          let mut chr = chr.lock();
          if chr
            .properties
            .intersects(NimbleProperties::INDICATE | NimbleProperties::NOTIFY)
          {
            let chr = &mut *chr;
            self.notify_characteristic.push(extend_lifetime_mut(chr));
          }
        }
      }
    }

    self.started = true;

    Ok(())
  }

  /// Disconnect the specified client.
  pub fn disconnect(&mut self, conn_id: u16) -> Result<(), BLEReturnCode> {
    self.disconnect_with_reason(
      conn_id,
      esp_idf_sys::ble_error_codes_BLE_ERR_REM_USER_CONN_TERM as _,
    )
  }

  /// Disconnect the specified client with optional reason.
  pub fn disconnect_with_reason(&mut self, conn_id: u16, reason: u8) -> Result<(), BLEReturnCode> {
    unsafe { ble!(esp_idf_sys::ble_gap_terminate(conn_id, reason)) }
  }

  /// Prints dump of local GATT database.
  /// This is useful to log local state of database in human readable form.
  pub fn ble_gatts_show_local(&self) {
    unsafe {
      esp_idf_sys::ble_gatts_show_local();
    }
  }

  pub fn connected_count(&self) -> usize {
    self.connections.len()
  }

  pub fn connections(&self) -> impl Iterator<Item = BLEConnDesc> + '_ {
    self
      .connections
      .iter()
      .filter_map(|x| ble_gap_conn_find(*x).ok())
  }

  pub fn create_service(&mut self, uuid: BleUuid) -> Arc<Mutex<BLEService>> {
    let service = Arc::new(Mutex::new(BLEService::new(uuid)));
    self.services.push(service.clone());
    service
  }

  /// Set the server to automatically start advertising when a client disconnects.
  pub fn advertise_on_disconnect(&mut self, value: bool) -> &mut Self {
    self.advertise_on_disconnect = value;
    self
  }

  /// Request an Update the connection parameters:
  /// Can only be used after a connection has been established.
  ///
  /// * `conn_handle`: The connection handle of the peer to send the request to.
  /// * `min_interval`: The minimum connection interval in 1.25ms units.
  /// * `max_interval`: The maximum connection interval in 1.25ms units.
  /// * `latency`: The number of packets allowed to skip (extends max interval).
  /// * `timeout`: The timeout time in 10ms units before disconnecting.
  pub fn update_conn_params(
    &mut self,
    conn_handle: u16,
    min_interval: u16,
    max_interval: u16,
    latency: u16,
    timeout: u16,
  ) -> Result<(), BLEReturnCode> {
    let params = esp_idf_sys::ble_gap_upd_params {
      itvl_min: min_interval,
      itvl_max: max_interval,
      latency,
      supervision_timeout: timeout,
      min_ce_len: esp_idf_sys::BLE_GAP_INITIAL_CONN_MIN_CE_LEN as _,
      max_ce_len: esp_idf_sys::BLE_GAP_INITIAL_CONN_MAX_CE_LEN as _,
    };

    unsafe { ble!(esp_idf_sys::ble_gap_update_params(conn_handle, &params)) }
  }

  pub(crate) fn reset(&mut self) {
    self.advertise_on_disconnect = true;
    self.services.clear();
    self.notify_characteristic.clear();
    self.connections.clear();
    self.on_connect = None;
    self.on_disconnect = None;
    self.on_passkey_request = None;
    self.on_confirm_pin = None;
    self.on_authentication_complete = None;
  }

  pub(crate) extern "C" fn handle_gap_event(
    _event: *mut esp_idf_sys::ble_gap_event,
    _arg: *mut c_void,
  ) -> i32 {
    let event = unsafe { &*_event };
    let server = BLEDevice::take().get_server();

    match event.type_ as _ {
      esp_idf_sys::BLE_GAP_EVENT_CONNECT => {
        let connect = unsafe { &event.__bindgen_anon_1.connect };
        if connect.status == 0 {
          server.connections.push(connect.conn_handle);

          if let Ok(desc) = ble_gap_conn_find(connect.conn_handle) {
            let server = UnsafeCell::new(server);
            unsafe {
              if let Some(callback) = (*server.get()).on_connect.as_mut() {
                callback(*server.get(), &desc);
              }
            }
          }
        }
      }
      esp_idf_sys::BLE_GAP_EVENT_DISCONNECT => {
        let disconnect = unsafe { &event.__bindgen_anon_1.disconnect };
        if let Some(idx) = server
          .connections
          .iter()
          .position(|x| *x == disconnect.conn.conn_handle)
        {
          server.connections.swap_remove(idx);
        }

        if let Some(callback) = server.on_disconnect.as_mut() {
          callback(&BLEConnDesc(disconnect.conn), disconnect.reason);
        }

        #[cfg(not(esp_idf_bt_nimble_ext_adv))]
        if server.advertise_on_disconnect {
          if let Err(err) = BLEDevice::take().get_advertising().lock().start() {
            ::log::warn!("can't start advertising: {:?}", err);
          }
        }
      }
      esp_idf_sys::BLE_GAP_EVENT_SUBSCRIBE => {
        let subscribe = unsafe { &event.__bindgen_anon_1.subscribe };
        if let Some(chr) = server
          .notify_characteristic
          .iter_mut()
          .find(|x| x.handle == subscribe.attr_handle)
        {
          if chr.properties.intersects(
            NimbleProperties::READ_AUTHEN
              | NimbleProperties::READ_AUTHOR
              | NimbleProperties::READ_ENC,
          ) {
            if let Ok(desc) = ble_gap_conn_find(subscribe.conn_handle) {
              if !desc.encrypted() {
                let rc = unsafe { esp_idf_sys::ble_gap_security_initiate(subscribe.conn_handle) };
                if rc != 0 {
                  ::log::error!("ble_gap_security_initiate: rc={}", rc);
                }
              }
            }
          }

          chr.subscribe(subscribe);
        }
      }
      esp_idf_sys::BLE_GAP_EVENT_MTU => {
        let mtu = unsafe { &event.__bindgen_anon_1.mtu };
        ::log::info!(
          "mtu update event; conn_handle={} mtu={}",
          mtu.conn_handle,
          mtu.value
        );
      }
      esp_idf_sys::BLE_GAP_EVENT_NOTIFY_TX => {
        let notify_tx = unsafe { &event.__bindgen_anon_1.notify_tx };
        if let Some(chr) = server
          .notify_characteristic
          .iter_mut()
          .find(|x| x.handle == notify_tx.attr_handle)
        {
          if notify_tx.indication() > 0 {
            if notify_tx.status == 0 {
              return 0;
            }

            BLEDevice::take()
              .get_server()
              .clear_indicate_wait(notify_tx.conn_handle);
          }

          if let Some(callback) = &mut chr.on_notify_tx {
            callback(NotifyTx { notify_tx });
          }
        }
      }
      #[cfg(not(esp_idf_bt_nimble_ext_adv))]
      esp_idf_sys::BLE_GAP_EVENT_ADV_COMPLETE => {
        return crate::BLEAdvertising::handle_gap_event(_event, _arg);
      }
      #[cfg(esp_idf_bt_nimble_ext_adv)]
      esp_idf_sys::BLE_GAP_EVENT_ADV_COMPLETE | esp_idf_sys::BLE_GAP_EVENT_SCAN_REQ_RCVD => {
        return crate::BLEExtAdvertising::handle_gap_event(_event, _arg);
      }
      esp_idf_sys::BLE_GAP_EVENT_CONN_UPDATE => {
        ::log::debug!("Connection parameters updated.");
      }
      esp_idf_sys::BLE_GAP_EVENT_CONN_UPDATE_REQ => {}
      esp_idf_sys::BLE_GAP_EVENT_REPEAT_PAIRING => {
        let repeat_pairing = unsafe { &event.__bindgen_anon_1.repeat_pairing };

        // Delete the old bond
        let Ok(desc) = crate::utilities::ble_gap_conn_find(repeat_pairing.conn_handle) else {
          return esp_idf_sys::BLE_GAP_REPEAT_PAIRING_IGNORE as _;
        };
        unsafe {
          esp_idf_sys::ble_store_util_delete_peer(&desc.0.peer_id_addr);
        }

        // Return BLE_GAP_REPEAT_PAIRING_RETRY to indicate that the host should
        // continue with the pairing operation.
        return esp_idf_sys::BLE_GAP_REPEAT_PAIRING_RETRY as _;
      }
      esp_idf_sys::BLE_GAP_EVENT_ENC_CHANGE => {
        let enc_change = unsafe { &event.__bindgen_anon_1.enc_change };
        let Ok(desk) = ble_gap_conn_find(enc_change.conn_handle) else {
          return esp_idf_sys::BLE_ATT_ERR_INVALID_HANDLE as _;
        };
        if let Some(callback) = &server.on_authentication_complete {
          callback(&desk, enc_change.status);
        }
      }
      esp_idf_sys::BLE_GAP_EVENT_PASSKEY_ACTION => {
        let passkey = unsafe { &event.__bindgen_anon_1.passkey };
        let mut pkey = esp_idf_sys::ble_sm_io {
          action: passkey.params.action,
          ..Default::default()
        };
        match passkey.params.action as _ {
          esp_idf_sys::BLE_SM_IOACT_DISP => {
            pkey.__bindgen_anon_1.passkey = if let Some(callback) = &server.on_passkey_request {
              callback()
            } else {
              BLEDevice::take().security().get_passkey()
            };

            let rc = unsafe { esp_idf_sys::ble_sm_inject_io(passkey.conn_handle, &mut pkey) };
            ::log::debug!("BLE_SM_IOACT_DISP; ble_sm_inject_io result: {}", rc);
          }
          esp_idf_sys::BLE_SM_IOACT_NUMCMP => {
            if let Some(callback) = &server.on_confirm_pin {
              pkey.__bindgen_anon_1.numcmp_accept = callback(passkey.params.numcmp) as _;
            } else {
              ::log::warn!("on_passkey_request is not setted");
            }
            let rc = unsafe { esp_idf_sys::ble_sm_inject_io(passkey.conn_handle, &mut pkey) };
            ::log::debug!("BLE_SM_IOACT_NUMCMP; ble_sm_inject_io result: {}", rc);
          }
          esp_idf_sys::BLE_SM_IOACT_INPUT => {
            if let Some(callback) = &server.on_passkey_request {
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
      esp_idf_sys::BLE_GAP_EVENT_IDENTITY_RESOLVED
      | esp_idf_sys::BLE_GAP_EVENT_PHY_UPDATE_COMPLETE => {}
      _ => {
        ::log::warn!("unhandled event: {}", event.type_);
      }
    }

    0
  }

  pub(super) fn set_indicate_wait(&self, conn_handle: u16) -> bool {
    !self.indicate_wait.contains(&conn_handle)
  }

  pub(super) fn clear_indicate_wait(&mut self, conn_handle: u16) {
    if let Some(it) = self.indicate_wait.iter_mut().find(|x| **x == conn_handle) {
      *it = BLE_HS_CONN_HANDLE_NONE;
    }
  }
}
