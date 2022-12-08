use alloc::vec::Vec;

use crate::{
  ble_remote_characteristic::BLERemoteCharacteristicState,
  utilities::{BLEReader, BLEWriter, BleUuid, WeakUnsafeCell},
  BLEReturnCode,
};

pub struct BLERemoteDescriptor {
  characteristic: WeakUnsafeCell<BLERemoteCharacteristicState>,
  uuid: BleUuid,
  handle: u16,
}

impl BLERemoteDescriptor {
  pub(crate) fn new(
    characteristic: WeakUnsafeCell<BLERemoteCharacteristicState>,
    dsc: &esp_idf_sys::ble_gatt_dsc,
  ) -> Self {
    Self {
      characteristic,
      uuid: BleUuid::from(dsc.uuid),
      handle: dsc.handle,
    }
  }

  pub fn uuid(&self) -> BleUuid {
    self.uuid
  }

  fn conn_handle(&self) -> u16 {
    match self.characteristic.upgrade() {
      Some(x) => x.conn_handle(),
      None => esp_idf_sys::BLE_HS_CONN_HANDLE_NONE as _,
    }
  }

  pub async fn read_value(&mut self) -> Result<Vec<u8>, BLEReturnCode> {
    let mut reader = BLEReader::new(self.conn_handle(), self.handle);
    reader.read_value().await
  }

  pub async fn write_value(&mut self, data: &[u8], response: bool) -> Result<(), BLEReturnCode> {
    let mut writer = BLEWriter::new(self.conn_handle(), self.handle);
    writer.write_value(data, response).await
  }
}
