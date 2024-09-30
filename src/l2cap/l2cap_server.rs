use super::{L2cap, ReceivedData};
use crate::{
  ble,
  utilities::{extend_lifetime_mut, mutex::Mutex, voidp_to_ref},
  BLEError, Channel,
};
use esp_idf_svc::sys;

const N: usize = sys::CONFIG_BT_NIMBLE_L2CAP_COC_MAX_NUM as usize;
static SERVER_LIST: Mutex<heapless::Vec<L2capServer, N>> = Mutex::new(heapless::Vec::new());

#[allow(clippy::type_complexity)]
pub struct L2capServer {
  l2cap: L2cap,
  coc_chan: *mut sys::ble_l2cap_chan,
  peer_sdu_size: u16,
  channel: Channel<ReceivedData, 1>,
}

impl L2capServer {
  pub fn create(psm: u16, mtu: u16) -> Result<&'static mut L2capServer, BLEError> {
    let mut list = SERVER_LIST.lock();
    list
      .push(L2capServer {
        l2cap: Default::default(),
        coc_chan: core::ptr::null_mut(),
        peer_sdu_size: 0,
        channel: Channel::new(),
      })
      .map_err(|_| BLEError::convert(sys::BLE_HS_ENOMEM as _).unwrap_err())?;

    let server = list.last_mut().unwrap();
    server.l2cap.init(mtu, 20)?;

    unsafe {
      ble!(sys::ble_l2cap_create_server(
        psm,
        mtu,
        Some(Self::handle_l2cap_event),
        server as *mut Self as _,
      ))?;
    }
    Ok(unsafe { extend_lifetime_mut(server) })
  }

  pub fn tx(&mut self, data: &[u8]) -> Result<(), BLEError> {
    self.l2cap.tx(self.coc_chan, data)
  }

  pub async fn rx(&mut self) -> ReceivedData {
    self.channel.receive().await
  }

  pub(crate) extern "C" fn handle_l2cap_event(
    _event: *mut sys::ble_l2cap_event,
    arg: *mut core::ffi::c_void,
  ) -> i32 {
    let event = unsafe { &*_event };
    let server = unsafe { voidp_to_ref::<Self>(arg) };

    match event.type_ as _ {
      sys::BLE_L2CAP_EVENT_COC_CONNECTED => {
        let connect = unsafe { event.__bindgen_anon_1.connect };
        if connect.status > 0 {
          ::log::warn!("LE COC error: {}", connect.status);
          return 0;
        }

        server.coc_chan = connect.chan;
        0
      }
      sys::BLE_L2CAP_EVENT_COC_DISCONNECTED => {
        let disconnect = unsafe { event.__bindgen_anon_1.disconnect };
        ::log::debug!("LE CoC disconnected: {:?}", disconnect.chan);
        server.coc_chan = core::ptr::null_mut();
        0
      }
      sys::BLE_L2CAP_EVENT_COC_ACCEPT => {
        let accept = unsafe { event.__bindgen_anon_1.accept };
        server.peer_sdu_size = accept.peer_sdu_size;
        server.l2cap.ble_l2cap_recv_ready(accept.chan);
        0
      }
      sys::BLE_L2CAP_EVENT_COC_DATA_RECEIVED => {
        let receive = unsafe { event.__bindgen_anon_1.receive };
        if !receive.sdu_rx.is_null() {
          let _ = server.channel.try_send(ReceivedData::from_raw(receive));
        }
        server.l2cap.ble_l2cap_recv_ready(receive.chan);
        0
      }

      _ => 0,
    }
  }
}

unsafe impl Send for L2capServer {}
