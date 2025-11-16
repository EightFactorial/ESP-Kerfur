//! Actions the TCA9554 can perform.
//!
//! Since these functions can be called either by the TCA9554 driver given a pin
//! type, or by the pin itself, they are written here to avoid code duplication.

use embassy_sync::{blocking_mutex::raw::RawMutex, mutex::Mutex};
use embedded_hal_async::i2c::{I2c, SevenBitAddress};

use crate::pin::TCAPin;

pub(crate) async fn placeholder<M: RawMutex, I2C: I2c, PIN: TCAPin>(
    i2c: &Mutex<M, I2C>,
    _addr: SevenBitAddress,
) {
    let _i2c = i2c.lock().await;
}
