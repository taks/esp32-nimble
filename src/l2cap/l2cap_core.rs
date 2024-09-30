use crate::{ble, utilities::os_mbuf_append, BLEError};
use alloc::vec::Vec;

#[derive(Default)]
pub struct L2cap {
  mempool: esp_idf_sys::os_mempool,
  mbuf_pool: esp_idf_sys::os_mbuf_pool,
  coc_memory: Vec<esp_idf_sys::os_membuf_t>,
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

  pub fn tx(
    &mut self,
    chan: *mut esp_idf_sys::ble_l2cap_chan,
    data: &[u8],
  ) -> Result<(), BLEError> {
    let mtu = L2cap::get_chan_info(chan).peer_l2cap_mtu as usize;
    let mut data = data;

    while !data.is_empty() {
      let sdu_rx = self.sdu_rx();
      let (data0, data1) = data.split_at(if data.len() < mtu { data.len() } else { mtu });

      let rc = os_mbuf_append(sdu_rx, data0);
      assert_eq!(rc, 0);

      loop {
        let rc = unsafe { esp_idf_sys::ble_l2cap_send(chan, sdu_rx) };
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

  pub fn sdu_rx(&mut self) -> *mut esp_idf_sys::os_mbuf {
    loop {
      let ret = unsafe { super::os_mbuf_get_pkthdr(&mut self.mbuf_pool, 0) };
      if !ret.is_null() {
        return ret;
      }
      esp_idf_hal::delay::FreeRtos::delay_ms(10);
    }
  }

  pub(crate) fn ble_l2cap_recv_ready(&mut self, chan: *mut esp_idf_sys::ble_l2cap_chan) -> i32 {
    let sdu_rx = self.sdu_rx();
    unsafe { esp_idf_sys::ble_l2cap_recv_ready(chan, sdu_rx) }
  }

  pub(crate) fn get_chan_info(
    chan: *mut esp_idf_sys::ble_l2cap_chan,
  ) -> esp_idf_sys::ble_l2cap_chan_info {
    let mut chan_info = esp_idf_sys::ble_l2cap_chan_info::default();
    let rc = unsafe { esp_idf_sys::ble_l2cap_get_chan_info(chan, &mut chan_info as _) };
    assert_eq!(rc, 0);
    chan_info
  }
}

#[inline]
const fn os_mempool_size(n: usize, blksize: usize) -> usize {
  let size = core::mem::size_of::<esp_idf_sys::os_membuf_t>();
  blksize.div_ceil(size) * n
}
