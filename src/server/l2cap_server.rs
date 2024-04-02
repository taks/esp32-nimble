use alloc::boxed::Box;
use core::cell::RefCell;

#[cfg(not(esp_idf_soc_esp_nimble_controller))]
use esp_idf_sys::os_mbuf_free;
#[cfg(esp_idf_soc_esp_nimble_controller)]
use esp_idf_sys::r_os_mbuf_free as os_mbuf_free;

use crate::{
  ble,
  utilities::{os_mbuf_into_slice, voidp_to_ref, L2cap},
  BLEError,
};

const N: usize = esp_idf_sys::CONFIG_BT_NIMBLE_L2CAP_COC_MAX_NUM as usize;
struct L2capServerList {
  list: RefCell<heapless::Vec<L2capServer, N>>,
}
unsafe impl Sync for L2capServerList {}

static SERVER_LIST: L2capServerList = L2capServerList {
  list: RefCell::new(heapless::Vec::new()),
};

#[allow(clippy::type_complexity)]
pub struct L2capServer {
  l2cap: L2cap,
  peer_sdu_size: u16,
  on_data_received: Box<dyn FnMut(&[u8]) + Send + Sync>,
}

impl L2capServer {
  pub fn create(
    psm: u16,
    mtu: u16,
    on_data_received: impl FnMut(&[u8]) + Send + Sync + 'static,
  ) -> Result<(), BLEError> {
    let mut list = SERVER_LIST.list.borrow_mut();
    list
      .push(L2capServer {
        l2cap: Default::default(),
        peer_sdu_size: 0,
        on_data_received: Box::new(on_data_received),
      })
      .map_err(|_| BLEError::convert(esp_idf_sys::BLE_HS_ENOMEM as _).unwrap_err())?;

    let server = list.last_mut().unwrap();
    server.l2cap.init(mtu, 20)?;

    unsafe {
      ble!(esp_idf_sys::ble_l2cap_create_server(
        psm,
        mtu,
        Some(Self::handle_l2cap_event),
        server as *mut Self as _,
      ))?;
    }
    Ok(())
  }

  pub(crate) extern "C" fn handle_l2cap_event(
    _event: *mut esp_idf_sys::ble_l2cap_event,
    arg: *mut core::ffi::c_void,
  ) -> i32 {
    let event = unsafe { &*_event };
    let server = unsafe { voidp_to_ref::<Self>(arg) };

    match event.type_ as _ {
      esp_idf_sys::BLE_L2CAP_EVENT_COC_CONNECTED => {
        let connect = unsafe { event.__bindgen_anon_1.connect };
        if connect.status > 0 {
          ::log::warn!("LE COC error: {}", connect.status);
          return 0;
        }

        0
      }
      esp_idf_sys::BLE_L2CAP_EVENT_COC_ACCEPT => {
        let accept = unsafe { event.__bindgen_anon_1.accept };
        server.peer_sdu_size = accept.peer_sdu_size;
        server.ble_l2cap_recv_ready(accept.chan);
        0
      }
      esp_idf_sys::BLE_L2CAP_EVENT_COC_DATA_RECEIVED => {
        let receive = unsafe { event.__bindgen_anon_1.receive };
        if !receive.sdu_rx.is_null() {
          (server.on_data_received)(os_mbuf_into_slice(receive.sdu_rx));
          unsafe { os_mbuf_free(receive.sdu_rx) };
        }
        server.ble_l2cap_recv_ready(receive.chan);
        0
      }

      _ => 0,
    }
  }

  fn ble_l2cap_recv_ready(&mut self, chan: *mut esp_idf_sys::ble_l2cap_chan) -> i32 {
    let sdu_rx = self.l2cap.sdu_rx();
    unsafe { esp_idf_sys::ble_l2cap_recv_ready(chan, sdu_rx) }
  }
}
