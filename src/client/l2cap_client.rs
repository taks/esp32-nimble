use alloc::boxed::Box;
use core::borrow::BorrowMut;
use esp_idf_hal::task::block_on;

use crate::{
  ble,
  utilities::{os_mbuf_append, voidp_to_ref, L2cap},
  BLEClient, BLEError, Signal,
};

pub struct L2capClient {
  l2cap: L2cap,
  coc_chan: *mut esp_idf_sys::ble_l2cap_chan,
  signal: Signal<u32>,
}

impl L2capClient {
  pub async fn connect(ble_client: &BLEClient, psm: u16, mtu: u16) -> Result<Box<Self>, BLEError> {
    let mut ret = Box::new(Self {
      l2cap: Default::default(),
      coc_chan: core::ptr::null_mut(),
      signal: Signal::new(),
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
    let sdu_rx = self.l2cap.sdu_rx();
    os_mbuf_append(sdu_rx, data);

    ble!(unsafe { esp_idf_sys::ble_l2cap_send(self.coc_chan, sdu_rx) })
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

        // let mut chan_info = esp_idf_sys::ble_l2cap_chan_info::default();
        // let rc = unsafe { esp_idf_sys::ble_l2cap_get_chan_info(connect.chan, &mut chan_info as _) };
        // assert_eq!(rc, 0);

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
