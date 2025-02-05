use crate::{ble, utilities::OsMBuf, BLEError};
use alloc::vec::Vec;
use esp_idf_svc::sys;

#[derive(Default)]
pub struct L2cap {
  mempool: sys::os_mempool,
  mbuf_pool: sys::os_mbuf_pool,
  coc_memory: Vec<sys::os_membuf_t>,
}

impl L2cap {
  pub fn init(&mut self, mtu: u16, coc_buf_count: u16) -> Result<(), BLEError> {
    self
      .coc_memory
      .reserve_exact(os_mempool_size(coc_buf_count as _, mtu as _));

    unsafe {
      ble!(super::os_mempool_init(
        &mut self.mempool,
        coc_buf_count,
        mtu as _,
        self.coc_memory.as_mut_ptr() as _,
        c"coc_sdu_pool".as_ptr()
      ))?;

      ble!(super::os_mbuf_pool_init(
        &mut self.mbuf_pool as _,
        &mut self.mempool as _,
        mtu,
        coc_buf_count
      ))?;
    }

    Ok(())
  }

  pub fn tx(&mut self, chan: *mut sys::ble_l2cap_chan, data: &[u8]) -> Result<(), BLEError> {
    let mtu = L2cap::get_chan_info(chan).peer_l2cap_mtu as usize;
    let mut data = data;

    while !data.is_empty() {
      let mut sdu_rx = self.sdu_rx();
      let (data0, data1) = data.split_at(if data.len() < mtu { data.len() } else { mtu });

      let rc = sdu_rx.append(data0);
      assert_eq!(rc, 0);

      loop {
        let rc = unsafe { sys::ble_l2cap_send(chan, sdu_rx.0) };
        match rc as _ {
          0 | sys::BLE_HS_ESTALLED => break,
          sys::BLE_HS_EBUSY => {}
          rc => return BLEError::convert(rc),
        }
        unsafe { sys::vPortYield() };
      }

      data = data1;
    }

    Ok(())
  }

  pub fn sdu_rx(&mut self) -> OsMBuf {
    loop {
      let ret = unsafe { super::os_mbuf_get_pkthdr(&mut self.mbuf_pool, 0) };
      if !ret.is_null() {
        return OsMBuf(ret);
      }
      esp_idf_svc::hal::delay::FreeRtos::delay_ms(10);
    }
  }

  pub(crate) fn ble_l2cap_recv_ready(&mut self, chan: *mut sys::ble_l2cap_chan) -> i32 {
    let sdu_rx = self.sdu_rx();
    unsafe { sys::ble_l2cap_recv_ready(chan, sdu_rx.0) }
  }

  pub(crate) fn get_chan_info(chan: *mut sys::ble_l2cap_chan) -> sys::ble_l2cap_chan_info {
    let mut chan_info = sys::ble_l2cap_chan_info::default();
    let rc = unsafe { sys::ble_l2cap_get_chan_info(chan, &mut chan_info as _) };
    assert_eq!(rc, 0);
    chan_info
  }
}

#[inline]
const fn os_mempool_size(n: usize, blksize: usize) -> usize {
  let size = core::mem::size_of::<sys::os_membuf_t>();
  blksize.div_ceil(size) * n
}
