use esp_idf_svc::sys;

pub struct L2capChannel(pub(crate) *mut sys::ble_l2cap_chan);
