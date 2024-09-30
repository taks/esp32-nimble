use alloc::boxed::Box;
use core::borrow::BorrowMut;
use esp_idf_svc::hal::task::block_on;
use esp_idf_svc::sys;

use super::{L2cap, ReceivedData};
use crate::{ble, utilities::voidp_to_ref, BLEClient, BLEError, Channel, Signal};

#[allow(clippy::type_complexity)]
pub struct L2capClient {
  l2cap: L2cap,
  coc_chan: *mut sys::ble_l2cap_chan,
  signal: Signal<u32>,
  channel: Channel<ReceivedData, 1>,
}

impl L2capClient {
  pub async fn connect(ble_client: &BLEClient, psm: u16, mtu: u16) -> Result<Box<Self>, BLEError> {
    let mut ret = Box::new(Self {
      l2cap: Default::default(),
      coc_chan: core::ptr::null_mut(),
      signal: Signal::new(),
      channel: Channel::new(),
    });

    ret.l2cap.init(mtu, 3)?;

    unsafe {
      ble!(sys::ble_l2cap_connect(
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

    ble!(unsafe { sys::ble_l2cap_disconnect(self.coc_chan) })?;
    let _ = self.signal.wait().await;

    Ok(())
  }

  pub fn tx(&mut self, data: &[u8]) -> Result<(), BLEError> {
    self.l2cap.tx(self.coc_chan, data)
  }

  pub async fn rx(&mut self) -> ReceivedData {
    self.channel.receive().await
  }

  pub(crate) extern "C" fn blecent_l2cap_coc_event_cb(
    _event: *mut sys::ble_l2cap_event,
    arg: *mut core::ffi::c_void,
  ) -> i32 {
    let event = unsafe { &*_event };
    let client = unsafe { voidp_to_ref::<Self>(arg) };

    match event.type_ as _ {
      sys::BLE_L2CAP_EVENT_COC_CONNECTED => {
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
      sys::BLE_L2CAP_EVENT_COC_DISCONNECTED => {
        let disconnect = unsafe { event.__bindgen_anon_1.disconnect };
        ::log::debug!("LE CoC disconnected: {:?}", disconnect.chan);
        client.coc_chan = core::ptr::null_mut();
        client.signal.signal(0);
        0
      }
      sys::BLE_L2CAP_EVENT_COC_DATA_RECEIVED => {
        let receive = unsafe { event.__bindgen_anon_1.receive };
        if !receive.sdu_rx.is_null() {
          let _ = client.channel.try_send(ReceivedData::from_raw(receive));
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
