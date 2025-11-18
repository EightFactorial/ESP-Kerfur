use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_time::{Duration, Timer};
use kerfur_gt911::Gt911;

use super::I2C;

/// Address of the GT911 touch sensor.
const I2C_ADDRESS: u8 = 0x5D;

/// A task that receives and handles touch input from the display.
#[embassy_executor::task]
pub(super) async fn task(i2c: &'static I2C) -> ! {
    defmt::info!("Preparing touch sensor...");

    // Create a GT911 touch sensor driver
    let mut touch = Gt911::new(I2cDevice::new(i2c), I2C_ADDRESS);

    // Initialize touch sensor
    let mut result = touch.init().await;
    while let Err(err) = result {
        defmt::error!("Failed to initialize touch sensor, {}", err);
        Timer::after(Duration::from_secs(5)).await;
        result = touch.init().await;
    }
    defmt::info!("Touch sensor initialized!");

    loop {
        // Query the number of touch points
        let points = match touch.touch_count().await {
            Ok(points) => points,
            Err(err) => {
                defmt::error!("Failed to query touch point count, {}", err);
                continue;
            }
        };

        // For each point, query its status
        for point in 0..points {
            Timer::after(Duration::from_millis(10)).await;
            match touch.touch_query(point).await {
                Ok(status) => {
                    defmt::info!(
                        "Touch point `{}`: x={}, y={}, area={}",
                        point,
                        status.x,
                        status.y,
                        status.area
                    );
                }
                Err(err) => {
                    defmt::error!("Failed to query touch point `{}`, {}", point, err);
                }
            }
        }

        Timer::after(Duration::from_secs(5)).await;
    }
}
