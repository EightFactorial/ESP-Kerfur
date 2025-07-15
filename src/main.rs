//! TODO

#![feature(never_type, type_alias_impl_trait)]
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;
use esp_hal::{rng::Rng, timer::timg::TimerGroup};
use time::UtcOffset;

mod clock;
use clock::Clock;

mod display;
mod handler;

/// The current timezone offset.
const TIMEZONE_OFFSET: UtcOffset = UtcOffset::UTC;
/// The NTP server to use for time synchronization.
const NTP_SERVER: &str = "pool.ntp.org";

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) -> ! {
    // Initialize the microcontroller.
    let per = handler::init();

    // Initialize the embassy runtime.
    esp_hal_embassy::init(TimerGroup::new(per.TIMG0).timer0);

    // Initialize the RNG driver.
    let rng = Rng::new(per.RNG);

    // Spawn the display task.
    let clock =
        Clock::new(spawner, TimerGroup::new(per.TIMG1).timer0, per.WIFI, per.RADIO_CLK, rng).await;
    display::spawn(spawner, per.GPIO5, per.I2C0, per.GPIO3, per.GPIO4, clock, rng);

    // Main loop, just run async tasks.
    loop {
        Timer::after_secs(30).await;
    }
}
