mod att_value;
pub use self::att_value::AttValue;

mod ble_2904;
pub use self::ble_2904::*;

mod ble_advertisement_data;
pub use self::ble_advertisement_data::BLEAdvertisementData;

mod ble_advertising;
pub use self::ble_advertising::BLEAdvertising;

mod ble_characteristic;
pub use self::ble_characteristic::*;

mod ble_conn_desc;
pub use self::ble_conn_desc::*;

mod ble_descriptor;
pub use self::ble_descriptor::BLEDescriptor;
pub use self::ble_descriptor::DescriptorProperties;

#[cfg(esp_idf_bt_nimble_ext_adv)]
#[cfg_attr(doc, doc(cfg(esp_idf_bt_nimble_ext_adv)))]
mod ble_ext_advertising;
#[cfg(esp_idf_bt_nimble_ext_adv)]
// #[cfg_attr(doc_cfg, doc(cfg(esp_idf_bt_nimble_ext_adv)))]
pub use self::ble_ext_advertising::*;

mod ble_hid_device;
pub use self::ble_hid_device::BLEHIDDevice;

mod ble_server;
pub use self::ble_server::BLEServer;

mod ble_service;
pub use self::ble_service::BLEService;

#[cfg(all(
  esp_idf_version_major = "5",
  esp_idf_version_minor = "2",
  not(esp_idf_version_patch = "0")
))]
pub mod cpfd;

pub mod hid;

mod on_write_args;
pub use self::on_write_args::OnWriteArgs;
pub use self::on_write_args::OnWriteDescriptorArgs;
