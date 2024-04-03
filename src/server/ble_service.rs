use alloc::{sync::Arc, vec::Vec};
use esp_idf_sys::ble_uuid_any_t;

use crate::{
  ble,
  utilities::{as_mut_ptr, mutex::Mutex, BleUuid},
  BLECharacteristic, BLEError,
};

use super::ble_characteristic::NimbleProperties;

const NULL_HANDLE: u16 = 0xFFFF;

pub struct BLEService {
  pub(crate) uuid: ble_uuid_any_t,
  pub(crate) handle: u16,
  pub(crate) characteristics: Vec<Arc<Mutex<BLECharacteristic>>>,
  svc_def: Option<[esp_idf_sys::ble_gatt_svc_def; 2]>,
  svc_def_characteristics: Vec<esp_idf_sys::ble_gatt_chr_def>,
}

impl BLEService {
  pub(crate) fn new(uuid: BleUuid) -> Self {
    Self {
      uuid: ble_uuid_any_t::from(uuid),
      handle: NULL_HANDLE,
      characteristics: Vec::new(),
      svc_def: None,
      svc_def_characteristics: Vec::new(),
    }
  }

  pub fn uuid(&self) -> BleUuid {
    BleUuid::from(self.uuid)
  }

  pub(crate) fn start(&mut self) -> Result<(), BLEError> {
    let svc_def = self.svc_def.get_or_insert_with(|| {
      let mut svc = [esp_idf_sys::ble_gatt_svc_def::default(); 2];
      svc[0].type_ = esp_idf_sys::BLE_GATT_SVC_TYPE_PRIMARY as _;
      svc[0].uuid = unsafe { &self.uuid.u };
      svc[0].includes = core::ptr::null_mut();

      if self.characteristics.is_empty() {
      } else {
        for chr in &self.characteristics {
          let arg = unsafe { as_mut_ptr(Arc::into_raw(chr.clone())) };
          let mut chr = chr.lock();

          self
            .svc_def_characteristics
            .push(esp_idf_sys::ble_gatt_chr_def {
              uuid: unsafe { &chr.uuid.u },
              access_cb: Some(BLECharacteristic::handle_gap_event),
              arg: arg as _,
              descriptors: chr.construct_svc_def_descriptors(),
              flags: chr.properties.bits(),
              min_key_size: 0,
              val_handle: &mut chr.handle,
              #[cfg(all(
                esp_idf_version_major = "5",
                esp_idf_version_minor = "2",
                not(esp_idf_version_patch = "0")
              ))]
              cpfd: chr.cpfd.as_mut_ptr(),
            });
        }
        self
          .svc_def_characteristics
          .push(esp_idf_sys::ble_gatt_chr_def::default());
        svc[0].characteristics = self.svc_def_characteristics.as_ptr();
      }

      svc[1].type_ = 0;
      svc
    });

    unsafe {
      ble!(esp_idf_sys::ble_gatts_count_cfg(svc_def.as_ptr()))?;
      ble!(esp_idf_sys::ble_gatts_add_svcs(svc_def.as_ptr()))?;
    }
    Ok(())
  }

  pub fn create_characteristic(
    &mut self,
    uuid: BleUuid,
    properties: NimbleProperties,
  ) -> Arc<Mutex<BLECharacteristic>> {
    let characteristic = Arc::new(Mutex::new(BLECharacteristic::new(uuid, properties)));
    self.characteristics.push(characteristic.clone());
    characteristic
  }

  /// Get the characteristic object for the UUID.
  pub async fn get_characteristic(&self, uuid: BleUuid) -> Option<&Arc<Mutex<BLECharacteristic>>> {
    self
      .characteristics
      .iter()
      .find(|x| unsafe { x.raw() }.uuid() == uuid)
  }
}

unsafe impl Send for BLEService {}
