use esp_idf_sys::*;

#[inline]
#[allow(unused)]
pub fn ble_npl_hw_enter_critical() {
  #[cfg(all(esp32c3, esp_idf_version_major = "4"))]
  unsafe {
    vPortEnterCritical();
  }

  #[cfg(all(not(esp32c3), esp_idf_version_major = "4"))]
  unsafe {
    xPortEnterCriticalTimeout(&mut ble_port_mutex, portMUX_NO_TIMEOUT);
  };

  #[cfg(not(esp_idf_version_major = "4"))]
  unsafe {
    npl_freertos_hw_enter_critical();
  }
}

#[inline]
#[allow(unused)]
pub fn ble_npl_hw_exit_critical() {
  #[cfg(all(esp32c3, esp_idf_version_major = "4"))]
  unsafe {
    vPortExitCritical();
  }

  #[cfg(all(not(esp32c3), esp_idf_version_major = "4"))]
  unsafe {
    vPortExitCritical(&mut ble_port_mutex);
  };

  #[cfg(not(esp_idf_version_major = "4"))]
  unsafe {
    npl_freertos_hw_exit_critical(0);
  }
}
