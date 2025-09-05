//! TODO
#![feature(never_type, type_alias_impl_trait)]
#![no_main]
#![no_std]

extern crate alloc;

use embassy_executor::Spawner;
use embassy_time::Timer;
use esp_hal::{rng::Rng, timer::timg::TimerGroup};

mod clock;
use clock::Clock;

mod display;
mod handler;
mod wifi;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) -> ! {
    // Initialize the microcontroller.
    let per = handler::init();

    // Initialize the embassy runtime.
    esp_hal_embassy::init(TimerGroup::new(per.TIMG0).timer0);

    // Initialize the RNG driver.
    let rng = Rng::new(per.RNG);
    // Create a Clock.
    let clock = Clock::new();

    // Spawn the clock synchronization task.
    clock::spawn(spawner, TimerGroup::new(per.TIMG1).timer0, per.WIFI, clock, rng);
    // Spawn the display task.
    display::spawn(spawner, per.GPIO5, per.GPIO21, per.I2C0, per.GPIO3, per.GPIO4, clock, rng);

    // Main loop, just run async tasks.
    loop {
        Timer::after_secs(30).await;
    }
}
