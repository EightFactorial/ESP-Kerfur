//! TODO
#![expect(dead_code, unused_imports, reason = "Drivers have not been written yet")]

use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;

// use es7210_driver::ES7210;
// use es8311_driver::ES8311;
use super::I2C;

/// Address of the ES8311 audio codec
const ES8311_ADDRESS: u8 = 0x5D;
/// Address of the ES7210 microphone
const ES7210_ADDRESS: u8 = 0x48;

/// A task that configures the ES8311 and ES7210.
#[embassy_executor::task]
pub(super) async fn task(_i2c: &'static I2C) {
    defmt::info!("Preparing audio peripherals...");

    defmt::info!("Configuring ES8311 audio codec...");
    // let _es8311 = Es8311::new(I2cDevice::new(i2c), ES8311_ADDRESS);

    defmt::info!("Configuring ES7210 microphone...");
    // let _es7210 = Es7210::new(I2cDevice::new(i2c), ES7210_ADDRESS);

    // Send a signal that the peripherals were configured
    defmt::info!("Audio peripherals configured!");
    crate::signal::AUDIO_ENABLE.signal(());
}
