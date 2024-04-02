use alloc::vec::Vec;

use crate::{ble, utilities::os_mbuf_get_pkthdr, BLEError};

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

    #[cfg(not(esp_idf_soc_esp_nimble_controller))]
    unsafe {
      ble!(esp_idf_sys::os_mempool_init(
        &mut self.mempool,
        coc_buf_count,
        mtu as _,
        self.coc_memory.as_mut_ptr() as _,
        c"coc_sdu_pool".as_ptr()
      ))?;

      ble!(esp_idf_sys::os_mbuf_pool_init(
        &mut self.mbuf_pool as _,
        &mut self.mempool as _,
        mtu,
        coc_buf_count
      ))?;
    }

    #[cfg(esp_idf_soc_esp_nimble_controller)]
    unsafe {
      ble!(esp_idf_sys::r_os_mempool_init(
        &mut self.mempool,
        buf_blocks,
        L2CAP_BUF_BLOCK_SIZE as _,
        self.coc_memory.as_mut_ptr() as _,
        c"coc_sdu_pool".as_ptr()
      ))?;

      ble!(esp_idf_sys::r_os_mbuf_pool_init(
        &mut self.mbuf_pool as _,
        &mut self.mempool as _,
        L2CAP_BUF_BLOCK_SIZE as _,
        buf_blocks as _
      ))?;
    }

    Ok(())
  }

  pub fn sdu_rx(&mut self) -> *mut esp_idf_sys::os_mbuf {
    loop {
      let ret = os_mbuf_get_pkthdr(&mut self.mbuf_pool, 0);
      if !ret.is_null() {
        return ret;
      }
      esp_idf_hal::delay::FreeRtos::delay_ms(10);
    }
  }
}

#[inline]
const fn os_mempool_size(n: usize, blksize: usize) -> usize {
  let size = core::mem::size_of::<esp_idf_sys::os_membuf_t>();
  blksize.div_ceil(size) * n
}
