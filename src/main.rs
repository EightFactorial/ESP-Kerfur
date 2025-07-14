//! TODO

#![feature(never_type, type_alias_impl_trait)]
#![no_std]
#![no_main]

use embassy_time::Timer;
use esp_hal::{rng::Rng, timer::timg::TimerGroup};

mod display;
mod handler;

#[esp_hal_embassy::main]
async fn main(spawner: embassy_executor::Spawner) -> ! {
    // Initialize the microcontroller.
    let per = handler::init();
    // Initialize the embassy runtime.
    esp_hal_embassy::init(TimerGroup::new(per.TIMG0).timer0);

    // Initialize the RNG driver.
    let rng = Rng::new(per.RNG);

    // Spawn the display task.
    display::spawn(spawner, rng, per.I2C0, per.GPIO3, per.GPIO4);

    // Main loop, just run async tasks.
    loop {
        Timer::after_secs(10).await;
    }
}
