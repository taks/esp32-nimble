use crate::{utilities::BleUuid, BLEAddress, BLEDevice, BLERemoteService, Signal};
use alloc::vec::Vec;
use esp_idf_sys::*;

pub struct BLEClient {
  address: BLEAddress,
  connected: bool,
  gattc_if: esp_gatt_if_t,
  conn_id: u16,
  services: Option<Vec<BLERemoteService>>,
  app_id: u16,
  signal: Signal<u32>,
}

impl BLEClient {
  pub fn new() -> Self {
    Self {
      address: [0; 6],
      connected: false,
      gattc_if: esp_idf_sys::ESP_GATT_IF_NONE as _,
      conn_id: esp_idf_sys::ESP_GATT_IF_NONE as _,
      services: None,
      app_id: 0,
      signal: Signal::new(),
    }
  }

  pub async fn connect(
    &mut self,
    address: BLEAddress,
    addr_type: esp_ble_addr_type_t,
  ) -> Result<(), EspError> {
    self.app_id = BLEDevice::add_device(self);
    unsafe {
      esp!(esp_idf_sys::esp_ble_gattc_app_register(0))?;

      esp!(self.signal.wait().await)?;

      self.address = address;

      esp!(esp_idf_sys::esp_ble_gattc_open(
        self.gattc_if,
        self.address.as_mut_ptr(),
        addr_type,
        true
      ))?;

      esp!(self.signal.wait().await)?;
      self.connected = true;
    }

    Ok(())
  }

  pub fn disconnect(&self) -> Result<(), EspError> {
    unsafe {
      esp!(esp_idf_sys::esp_ble_gattc_close(
        self.gattc_if,
        self.conn_id
      ))
    }
  }

  pub fn connected(&self) -> bool {
    self.connected
  }

  pub async fn get_services(
    &mut self,
  ) -> Result<core::slice::Iter<'_, BLERemoteService>, EspError> {
    if self.services.is_none() {
      self.services = Some(Vec::new());
      unsafe {
        esp!(esp_idf_sys::esp_ble_gattc_search_service(
          self.gattc_if,
          self.conn_id,
          core::ptr::null_mut()
        ))?;
      }
      esp!(self.signal.wait().await)?;
    }

    Ok(self.services.as_ref().unwrap().iter())
  }

  pub async fn get_service(&mut self, uuid: BleUuid) -> Result<BLERemoteService, EspError> {
    let mut iter = self.get_services().await?;
    iter
      .find(|x| x.uuid == uuid)
      .copied()
      .ok_or(EspError::from(ESP_FAIL).unwrap())
  }

  pub(crate) fn handle_gattc_event(
    &mut self,
    event: esp_gattc_cb_event_t,
    gattc_if: esp_gatt_if_t,
    param: *mut esp_ble_gattc_cb_param_t,
  ) {
    if self.gattc_if == (esp_idf_sys::ESP_GATT_IF_NONE as _)
      && event != esp_idf_sys::esp_gattc_cb_event_t_ESP_GATTC_REG_EVT
    {
      return;
    }

    match event {
      esp_idf_sys::esp_gattc_cb_event_t_ESP_GATTC_DISCONNECT_EVT => {
        let disconnect = unsafe { (*param).disconnect };
        if disconnect.conn_id != self.conn_id {
          return;
        }
        self.connected = false;
        unsafe {
          esp_idf_sys::esp_ble_gattc_app_unregister(self.gattc_if);
        }
        BLEDevice::remove_device(self);
      }
      esp_idf_sys::esp_gattc_cb_event_t_ESP_GATTC_OPEN_EVT => {
        self.conn_id = unsafe { (*param).open.conn_id };
        self.signal.signal(unsafe { (*param).open.status });
      }
      esp_idf_sys::esp_gattc_cb_event_t_ESP_GATTC_REG_EVT => {
        self.gattc_if = gattc_if;
        self.signal.signal(unsafe { (*param).reg.status });
      }
      esp_idf_sys::esp_gattc_cb_event_t_ESP_GATTC_SEARCH_CMPL_EVT => {
        let search_cmpl = unsafe { (*param).search_cmpl };
        if search_cmpl.conn_id != self.conn_id {
          return;
        }
        if search_cmpl.status != esp_idf_sys::esp_gatt_status_t_ESP_GATT_OK {
          log::error!(
            "search service failed, error status = {:X}",
            search_cmpl.status
          );
          return;
        }
        self.signal.signal(0);
      }
      esp_idf_sys::esp_gattc_cb_event_t_ESP_GATTC_SEARCH_RES_EVT => {
        let search_res = unsafe { (*param).search_res };
        if search_res.conn_id != self.conn_id {
          return;
        }
        let uuid = BleUuid::from(search_res.srvc_id);
        let remote_service =
          BLERemoteService::new(uuid, search_res.start_handle, search_res.end_handle);
        self.services.as_mut().unwrap().push(remote_service);
      }
      _ => {}
    }
  }
}
