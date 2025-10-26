//! TODO
#![expect(clippy::used_underscore_binding, reason = "Macros use underscore bindings internally")]
#![no_main]
#![no_std]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

mod initialize;

#[esp_rtos::main]
async fn main(_s: Spawner) -> ! {
    let _p = initialize::init();

    loop {
        Timer::after(Duration::MAX).await;
    }
}
