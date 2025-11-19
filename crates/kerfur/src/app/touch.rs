//! TODO

use alloc::vec::Vec;

use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_time::{Duration, Timer};
use kerfur_gt911::Gt911;

use super::I2C;

/// Address of the GT911 touch sensor.
const GT911_ADDRESS: u8 = 0x5D;

/// A task that receives and handles touch input from the display.
#[embassy_executor::task]
pub(super) async fn task(i2c: &'static I2C) -> ! {
    defmt::info!("Preparing touch sensor...");

    // Create a GT911 touch sensor driver
    let mut touch = Gt911::new(I2cDevice::new(i2c), GT911_ADDRESS);

    // Initialize touch sensor
    let mut result = touch.init().await;
    while let Err(err) = result {
        defmt::error!("Failed to initialize touch sensor, {}", err);
        Timer::after(Duration::from_secs(5)).await;
        result = touch.init().await;
    }
    defmt::info!("Touch sensor initialized!");

    let mut points = Vec::with_capacity(6);

    loop {
        // Query for all touch points
        match touch.touch_list(points).await {
            Ok(updated) => {
                points = updated;
            }
            Err((failed, err)) => {
                defmt::error!("Failed to read touch points, {}", err);
                points = failed;
                points.clear();
            }
        }

        for point in &points {
            defmt::info!(
                "Touch ID: {}, X: {}, Y: {}, A: {}",
                point.point_id,
                point.x,
                point.y,
                point.area
            );
        }

        Timer::after(Duration::from_secs(5)).await;
    }
}
