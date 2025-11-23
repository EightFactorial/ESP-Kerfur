//! TODO

use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_time::Timer;
use gt911_driver::{GT911, GT911Error};

use super::I2C;

/// Address of the GT911 touch sensor.
const GT911_ADDRESS: u8 = 0x5D;

/// A task that receives and handles touch input from the display.
#[embassy_executor::task]
pub(super) async fn task(i2c: &'static I2C) -> ! {
    defmt::info!("Preparing touch sensor...");

    // Create a GT911 touch sensor driver
    let mut touch = GT911::new(I2cDevice::new(i2c), GT911_ADDRESS);

    // Initialize touch sensor
    let mut result = touch.init_async().await;
    while let Err(err) = result {
        if let GT911Error::DeviceNotReady(status) = &err {
            defmt::warn!("Touch sensor not ready, status: {:#010b}", status.bits());
        } else {
            defmt::error!("Failed to initialize touch sensor, {}", err);
        }
        Timer::after_millis(100).await;
        result = touch.init_async().await;
    }
    defmt::info!("Touch sensor configured!");

    loop {
        // Query for all touch points
        match touch.query_touch_all_async().await {
            Ok(points) => {
                for point in points {
                    if let Some(point) = point {
                        defmt::info!(
                            "Touch ID: {}, X: {}, Y: {}, A: {}",
                            point.point,
                            point.x,
                            point.y,
                            point.area
                        );
                    }
                }
            }
            Err(err) => {
                defmt::error!("Failed to read gesture, {}", err);
            }
        }

        Timer::after_millis(25).await;
    }
}
