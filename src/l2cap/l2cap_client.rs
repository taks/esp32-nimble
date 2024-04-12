use alloc::boxed::Box;
use core::borrow::BorrowMut;
use esp_idf_hal::task::block_on;

#[cfg(not(esp_idf_soc_esp_nimble_controller))]
use esp_idf_sys::os_mbuf_free;
#[cfg(esp_idf_soc_esp_nimble_controller)]
use esp_idf_sys::r_os_mbuf_free as os_mbuf_free;

use super::{L2cap, OnDataReceived};
use crate::{
  ble,
  utilities::{os_mbuf_append, voidp_to_ref},
  BLEClient, BLEError, Signal,
};

#[allow(clippy::type_complexity)]
pub struct L2capClient {
  l2cap: L2cap,
  coc_chan: *mut esp_idf_sys::ble_l2cap_chan,
  signal: Signal<u32>,
  on_data_received: Option<Box<dyn FnMut(OnDataReceived) + Send + Sync>>,
}

impl L2capClient {
  pub async fn connect(ble_client: &BLEClient, psm: u16, mtu: u16) -> Result<Box<Self>, BLEError> {
    let mut ret = Box::new(Self {
      l2cap: Default::default(),
      coc_chan: core::ptr::null_mut(),
      signal: Signal::new(),
      on_data_received: None,
    });

    ret.l2cap.init(mtu, 3)?;

    unsafe {
      ble!(esp_idf_sys::ble_l2cap_connect(
        ble_client.conn_handle(),
        psm,
        mtu,
        ret.l2cap.sdu_rx(),
        Some(Self::blecent_l2cap_coc_event_cb),
        ret.borrow_mut() as *mut Self as _,
      ))?;
    }

    ble!(ret.signal.wait().await)?;

    Ok(ret)
  }

  pub async fn disconnect(&mut self) -> Result<(), BLEError> {
    if self.coc_chan.is_null() {
      return Ok(());
    }

    ble!(unsafe { esp_idf_sys::ble_l2cap_disconnect(self.coc_chan) })?;
    let _ = self.signal.wait().await;

    Ok(())
  }

  pub fn send(&mut self, data: &[u8]) -> Result<(), BLEError> {
    let mtu = L2cap::get_chan_info(self.coc_chan).peer_l2cap_mtu as usize;
    let mut data = data;

    while !data.is_empty() {
      let sdu_rx = self.l2cap.sdu_rx();
      let (data0, data1) = data.split_at(if data.len() < mtu { data.len() } else { mtu });

      let rc = os_mbuf_append(sdu_rx, data0);
      assert_eq!(rc, 0);

      loop {
        let rc = unsafe { esp_idf_sys::ble_l2cap_send(self.coc_chan, sdu_rx) };
        match rc as _ {
          0 | esp_idf_sys::BLE_HS_ESTALLED => break,
          esp_idf_sys::BLE_HS_EBUSY => {}
          rc => return BLEError::convert(rc),
        }
        unsafe { esp_idf_sys::vPortYield() };
      }

      data = data1;
    }

    Ok(())
  }

  pub fn on_data_received(
    &mut self,
    callback: impl FnMut(OnDataReceived) + Send + Sync + 'static,
  ) -> &mut Self {
    self.on_data_received = Some(Box::new(callback));
    self
  }

  pub(crate) extern "C" fn blecent_l2cap_coc_event_cb(
    _event: *mut esp_idf_sys::ble_l2cap_event,
    arg: *mut core::ffi::c_void,
  ) -> i32 {
    let event = unsafe { &*_event };
    let client = unsafe { voidp_to_ref::<Self>(arg) };

    match event.type_ as _ {
      esp_idf_sys::BLE_L2CAP_EVENT_COC_CONNECTED => {
        let connect = unsafe { event.__bindgen_anon_1.connect };
        if connect.status > 0 {
          ::log::warn!("LE COC error: {}", connect.status);
          client.signal.signal(connect.status as _);

          return 0;
        }

        client.coc_chan = connect.chan;
        client.signal.signal(0);
        0
      }
      esp_idf_sys::BLE_L2CAP_EVENT_COC_DISCONNECTED => {
        let disconnect = unsafe { event.__bindgen_anon_1.disconnect };
        ::log::debug!("LE CoC disconnected: {:?}", disconnect.chan);
        client.coc_chan = core::ptr::null_mut();
        client.signal.signal(0);
        0
      }
      esp_idf_sys::BLE_L2CAP_EVENT_COC_DATA_RECEIVED => {
        let receive = unsafe { event.__bindgen_anon_1.receive };
        if !receive.sdu_rx.is_null() {
          if let Some(callback) = &mut client.on_data_received {
            callback(OnDataReceived::from_raw(receive));
          } else {
            unsafe { os_mbuf_free(receive.sdu_rx) };
          }
        }
        client.l2cap.ble_l2cap_recv_ready(receive.chan);
        0
      }
      _ => 0,
    }
  }
}

impl Drop for L2capClient {
  fn drop(&mut self) {
    block_on(async {
      let _ = self.disconnect().await;
    });
  }
}
