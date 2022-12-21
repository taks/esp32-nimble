use crate::{
  ble,
  ble_device::OWN_ADDR_TYPE,
  ble_return_code::return_code_to_string,
  utilities::{ArcUnsafeCell, BleUuid},
  BLEAddress, BLEDevice, BLERemoteService, BLEReturnCode, Signal,
};
use alloc::{boxed::Box, string::ToString, vec::Vec};
use core::ffi::c_void;
use esp_idf_sys::*;

pub(crate) struct BLEClientState {
  address: Option<BLEAddress>,
  conn_handle: u16,
  services: Option<Vec<BLERemoteService>>,
  signal: Signal<u32>,
  connect_timeout_ms: u32,
  ble_gap_conn_params: ble_gap_conn_params,
  on_passkey_request: Option<Box<dyn Fn() -> u32 + Send + Sync>>,
  on_confirm_pin: Option<Box<dyn Fn(u32) -> bool + Send + Sync>>,
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
          itvl_min: BLE_GAP_INITIAL_CONN_ITVL_MIN as _,
          itvl_max: BLE_GAP_INITIAL_CONN_ITVL_MAX as _,
          latency: BLE_GAP_INITIAL_CONN_LATENCY as _,
          supervision_timeout: BLE_GAP_INITIAL_SUPERVISION_TIMEOUT as _,
          min_ce_len: BLE_GAP_INITIAL_CONN_MIN_CE_LEN as _,
          max_ce_len: BLE_GAP_INITIAL_CONN_MAX_CE_LEN as _,
        },
        signal: Signal::new(),
        on_passkey_request: None,
        on_confirm_pin: None,
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

  pub async fn connect(&mut self, addr: &BLEAddress) -> Result<(), BLEReturnCode> {
    unsafe {
      if esp_idf_sys::ble_gap_conn_find_by_addr(addr, core::ptr::null_mut()) == 0 {
        ::log::warn!("A connection to {:X?} already exists", addr.val);
        return BLEReturnCode::fail();
      }

      ble!(esp_idf_sys::ble_gap_connect(
        OWN_ADDR_TYPE,
        addr,
        self.state.connect_timeout_ms as _,
        &self.state.ble_gap_conn_params,
        Some(Self::handle_gap_event),
        self as *mut Self as _,
      ))?;
    }

    ble!(self.state.signal.wait().await)?;
    self.state.address = Some(*addr);

    Ok(())
  }

  pub async fn secure_connection(&mut self) -> Result<(), BLEReturnCode> {
    unsafe {
      ble!(esp_idf_sys::ble_gap_security_initiate(
        self.state.conn_handle
      ))?;
    }
    ble!(self.state.signal.wait().await)?;

    ::log::info!("secure_connection: success");

    Ok(())
  }

  pub fn disconnect(&self) -> Result<(), BLEReturnCode> {
    if !self.connected() {
      return Ok(());
    }

    unsafe {
      let rc = esp_idf_sys::ble_gap_terminate(
        self.state.conn_handle,
        esp_idf_sys::ble_error_codes_BLE_ERR_REM_USER_CONN_TERM as _,
      );

      BLEReturnCode::convert(rc as _)
    }
  }

  pub fn connected(&self) -> bool {
    self.state.conn_handle != (esp_idf_sys::BLE_HS_CONN_HANDLE_NONE as _)
  }

  pub async fn get_services(
    &mut self,
  ) -> Result<core::slice::IterMut<'_, BLERemoteService>, BLEReturnCode> {
    if self.state.services.is_none() {
      self.state.services = Some(Vec::new());
      unsafe {
        esp_idf_sys::ble_gattc_disc_all_svcs(
          self.state.conn_handle,
          Some(Self::service_discovered_cb),
          self as *mut Self as _,
        );
      }
      ble!(self.state.signal.wait().await)?;
    }

    Ok(self.state.services.as_mut().unwrap().iter_mut())
  }

  pub async fn get_service(
    &mut self,
    uuid: BleUuid,
  ) -> Result<&mut BLERemoteService, BLEReturnCode> {
    let mut iter = self.get_services().await?;
    iter
      .find(|x| x.uuid() == uuid)
      .ok_or_else(|| BLEReturnCode::fail().unwrap_err())
  }

  extern "C" fn handle_gap_event(event: *mut esp_idf_sys::ble_gap_event, arg: *mut c_void) -> i32 {
    let event = unsafe { &*event };
    let client = unsafe { &mut *(arg as *mut Self) };

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
          unsafe { esp_idf_sys::ble_store_util_delete_peer(&desc.peer_id_addr) };
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
    let client = unsafe { &mut *(arg as *mut Self) };
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
