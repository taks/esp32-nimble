use esp_idf_sys::*;

#[inline]
#[allow(unused)]
pub(crate) fn ble_npl_hw_enter_critical() {
  #[cfg(esp32c3)]
  unsafe {
    vPortEnterCritical();
  }

  #[cfg(not(esp32c3))]
  unsafe {
    xPortEnterCriticalTimeout(&mut ble_port_mutex, portMUX_NO_TIMEOUT)
  };
}

#[inline]
#[allow(unused)]
pub(crate) fn ble_npl_hw_exit_critical() {
  #[cfg(esp32c3)]
  unsafe {
    vPortExitCritical();
  }

  #[cfg(not(esp32c3))]
  unsafe {
    vPortExitCritical(&mut ble_port_mutex)
  };
}
