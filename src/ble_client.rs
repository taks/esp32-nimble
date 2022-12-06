use crate::{
  ble, ble_device::OWN_ADDR_TYPE, utilities::BleUuid, BLEAddress, BLERemoteService, BLEReturnCode,
  Signal,
};
use alloc::vec::Vec;
use esp_idf_sys::{c_types::c_void, *};

pub struct BLEClient {
  address: Option<BLEAddress>,
  conn_id: u16,
  services: Option<Vec<BLERemoteService>>,
  signal: Signal<u32>,
  connect_timeout_ms: u32,
  ble_gap_conn_params: ble_gap_conn_params,
}

impl BLEClient {
  pub fn new() -> Self {
    Self {
      address: None,
      conn_id: esp_idf_sys::BLE_HS_CONN_HANDLE_NONE as _,
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
    }
  }

  pub async fn connect(&mut self, addr: &BLEAddress) -> Result<(), BLEReturnCode> {
    unsafe {
      if esp_idf_sys::ble_gap_conn_find_by_addr(addr, core::ptr::null_mut()) == 0 {
        ::log::warn!("A connection to {:X?} already exists", addr.val);
        return BLEReturnCode::fail();
      }

      let rc = esp_idf_sys::ble_gap_connect(
        OWN_ADDR_TYPE,
        addr,
        self.connect_timeout_ms as _,
        &self.ble_gap_conn_params,
        Some(Self::handle_gap_event),
        self as *mut Self as _,
      ) as _;

      if rc != 0 {
        return BLEReturnCode::convert(rc);
      }
    }

    ble!(self.signal.wait().await)?;
    self.address = Some(*addr);

    Ok(())
  }

  pub fn disconnect(&self) -> Result<(), BLEReturnCode> {
    if !self.connected() {
      return Ok(());
    }

    unsafe {
      let rc = ble_gap_terminate(
        self.conn_id,
        esp_idf_sys::ble_error_codes_BLE_ERR_REM_USER_CONN_TERM as _,
      );

      BLEReturnCode::convert(rc as _)
    }
  }

  pub fn connected(&self) -> bool {
    self.conn_id != (BLE_HS_CONN_HANDLE_NONE as _)
  }

  pub async fn get_services(
    &mut self,
  ) -> Result<core::slice::IterMut<'_, BLERemoteService>, BLEReturnCode> {
    if self.services.is_none() {
      self.services = Some(Vec::new());
      unsafe {
        ble_gattc_disc_all_svcs(
          self.conn_id,
          Some(Self::service_discovered_cb),
          self as *mut Self as _,
        );
      }
      ble!(self.signal.wait().await)?;
    }

    Ok(self.services.as_mut().unwrap().iter_mut())
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
    let mut client = unsafe { &mut *(arg as *mut Self) };

    ::log::info!("handle_gap_event {}", event.type_);

    match event.type_ as _ {
      BLE_GAP_EVENT_CONNECT => {
        let connect = unsafe { &event.__bindgen_anon_1.connect };

        if connect.status == 0 {
          client.conn_id = connect.conn_handle;
          client.signal.signal(0);
        } else {
          ::log::info!("connect_status {}", connect.status);
          client.conn_id = BLE_HS_CONN_HANDLE_NONE as _;
          client.signal.signal(connect.status as _);
        }
      }
      BLE_GAP_EVENT_DISCONNECT => {
        let disconnect = unsafe { &event.__bindgen_anon_1.disconnect };
        if client.conn_id != disconnect.conn.conn_handle {
          return 0;
        }
        client.conn_id = BLE_HS_CONN_HANDLE_NONE as _;

        ::log::info!(
          "Disconnected: {:?}",
          BLEReturnCode::from(disconnect.reason as _)
        );
      }
      _ => {}
    }
    0
  }

  extern "C" fn service_discovered_cb(
    conn_handle: u16,
    error: *const ble_gatt_error,
    service: *const ble_gatt_svc,
    arg: *mut c_void,
  ) -> i32 {
    let client = unsafe { &mut *(arg as *mut Self) };
    if client.conn_id != conn_handle {
      return 0;
    }

    let error = unsafe { &*error };
    let service = unsafe { &*service };

    if error.status == 0 {
      // Found a service - add it to the vector
      let service = BLERemoteService::new(client.conn_id, service);
      client.services.as_mut().unwrap().push(service);
      return 0;
    }

    let ret = if error.status == (BLE_HS_EDONE as _) {
      0
    } else {
      error.status as _
    };

    client.signal.signal(ret);
    ret as _
  }
}
