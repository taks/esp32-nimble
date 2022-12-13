use crate::{
  ble,
  utilities::{mutex::Mutex, BleUuid},
  BLEDevice, BLEReturnCode, BLEService,
};
use alloc::{sync::Arc, vec::Vec};
use core::ffi::c_void;

const BLE_HS_CONN_HANDLE_NONE: u16 = esp_idf_sys::BLE_HS_CONN_HANDLE_NONE as _;

pub struct BLEServer {
  pub(crate) started: bool,
  advertise_on_disconnect: bool,
  services: Vec<Arc<Mutex<BLEService>>>,
  connections: Vec<u16>,
  indicate_wait: [u16; esp_idf_sys::CONFIG_BT_NIMBLE_MAX_CONNECTIONS as _],
}

impl BLEServer {
  pub(crate) fn new() -> Self {
    Self {
      started: false,
      advertise_on_disconnect: true,
      services: Vec::new(),
      connections: Vec::new(),
      indicate_wait: [BLE_HS_CONN_HANDLE_NONE; esp_idf_sys::CONFIG_BT_NIMBLE_MAX_CONNECTIONS as _],
    }
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

      for svc in &self.services {
        let mut svc = svc.lock();
        ble!(esp_idf_sys::ble_gatts_find_svc(
          &svc.uuid.u,
          &mut svc.handle
        ))?;
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

        if server.advertise_on_disconnect {
          if let Err(err) = BLEDevice::take().get_advertising().start(None) {
            ::log::warn!("can't start advertising: {:?}", err);
          }
        }
      }
      esp_idf_sys::BLE_GAP_EVENT_SUBSCRIBE => {
        let subscribe = unsafe { &event.__bindgen_anon_1.subscribe };
        for svc in &server.services {
          for chr in &svc.lock().characteristics {
            let mut chr = chr.lock();
            if chr.handle == subscribe.attr_handle {
              chr.subscribe(subscribe);
              return 0;
            }
          }
        }
      }
      _ => {}
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
