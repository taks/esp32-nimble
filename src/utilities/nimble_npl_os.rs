use esp_idf_sys::{
  ble_port_mutex, portMUX_NO_TIMEOUT, vPortExitCritical, xPortEnterCriticalTimeout,
};

#[inline]
#[allow(unused)]
pub(crate) fn ble_npl_hw_enter_critical() {
  unsafe { xPortEnterCriticalTimeout(&mut ble_port_mutex, portMUX_NO_TIMEOUT) };
}

#[inline]
#[allow(unused)]
pub(crate) fn ble_npl_hw_exit_critical() {
  unsafe { vPortExitCritical(&mut ble_port_mutex) };
}
