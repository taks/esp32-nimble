use crate::{
  ble,
  utilities::{ble_gap_conn_find, extend_lifetime_mut, mutex::Mutex, BleUuid},
  BLECharacteristic, BLEDevice, BLEReturnCode, BLEService, NimbleProperties, NotifyTx,
};
use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::ffi::c_void;

const BLE_HS_CONN_HANDLE_NONE: u16 = esp_idf_sys::BLE_HS_CONN_HANDLE_NONE as _;

#[allow(clippy::type_complexity)]
pub struct BLEServer {
  pub(crate) started: bool,
  advertise_on_disconnect: bool,
  services: Vec<Arc<Mutex<BLEService>>>,
  notify_characteristic: Vec<&'static mut BLECharacteristic>,
  connections: Vec<u16>,
  indicate_wait: [u16; esp_idf_sys::CONFIG_BT_NIMBLE_MAX_CONNECTIONS as _],

  on_connect: Option<Box<dyn FnMut(&esp_idf_sys::ble_gap_conn_desc) + Send + Sync>>,
  on_disconnect: Option<Box<dyn FnMut(&esp_idf_sys::ble_gap_conn_desc) + Send + Sync>>,
  on_passkey_request: Option<Box<dyn Fn() -> u32 + Send + Sync>>,
  on_confirm_pin: Option<Box<dyn Fn(u32) -> bool + Send + Sync>>,
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
    }
  }

  pub fn on_connect(
    &mut self,
    callback: impl FnMut(&esp_idf_sys::ble_gap_conn_desc) + Send + Sync + 'static,
  ) -> &mut Self {
    self.on_connect = Some(Box::new(callback));
    self
  }

  pub fn on_disconnect(
    &mut self,
    callback: impl FnMut(&esp_idf_sys::ble_gap_conn_desc) + Send + Sync + 'static,
  ) -> &mut Self {
    self.on_disconnect = Some(Box::new(callback));
    self
  }

  pub fn on_passkey_request(
    &mut self,
    callback: impl Fn() -> u32 + Send + Sync + 'static,
  ) -> &mut Self {
    self.on_passkey_request = Some(Box::new(callback));
    self
  }

  pub fn on_confirm_pin(
    &mut self,
    callback: impl Fn(u32) -> bool + Send + Sync + 'static,
  ) -> &mut Self {
    self.on_confirm_pin = Some(Box::new(callback));
    self
  }

  pub fn start(&mut self) -> Result<(), BLEReturnCode> {
    if self.started {
      return Ok(());
    }

    for svc in &mut self.services {
      svc.lock().start()?;
    }

    unsafe {
      ble!(esp_idf_sys::ble_gatts_start())?;

      #[cfg(debug_assertions)]
      esp_idf_sys::ble_gatts_show_local();

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

  pub fn connected_count(&self) -> usize {
    self.connections.len()
  }

  pub fn create_service(&mut self, uuid: BleUuid) -> Arc<Mutex<BLEService>> {
    let service = Arc::new(Mutex::new(BLEService::new(uuid)));
    self.services.push(service.clone());
    service
  }

  pub(crate) extern "C" fn handle_gap_event(
    event: *mut esp_idf_sys::ble_gap_event,
    _arg: *mut c_void,
  ) -> i32 {
    let event = unsafe { &*event };
    let server = BLEDevice::take().get_server();

    match event.type_ as _ {
      esp_idf_sys::BLE_GAP_EVENT_CONNECT => {
        let connect = unsafe { &event.__bindgen_anon_1.connect };
        if connect.status == 0 {
          server.connections.push(connect.conn_handle);

          if let Ok(desc) = ble_gap_conn_find(connect.conn_handle) {
            if let Some(callback) = server.on_connect.as_mut() {
              callback(&desc);
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
          callback(&disconnect.conn);
        }

        if server.advertise_on_disconnect {
          if let Err(err) = BLEDevice::take().get_advertising().start() {
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
              if desc.sec_state.encrypted() == 0 {
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
      esp_idf_sys::BLE_GAP_EVENT_CONN_UPDATE => {
        ::log::debug!("Connection parameters updated.");
      }
      esp_idf_sys::BLE_GAP_EVENT_CONN_UPDATE_REQ => {}
      esp_idf_sys::BLE_GAP_EVENT_ENC_CHANGE => {
        // let enc_change = unsafe { &event.__bindgen_anon_1.enc_change };
        ::log::info!("AuthenticationComplete");
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
