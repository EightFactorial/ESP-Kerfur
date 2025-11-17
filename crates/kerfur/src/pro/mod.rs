//! The first, "pro" processor core
//!
//! Cannot access values outside of this module.

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

/// Spawn the task and run it on the protocol core.
#[rustfmt::skip]
pub(super) fn spawn(spawner: Spawner, peripherals: ProPeripherals) {
    defmt::info!("Spawning protocol core task...");
    spawner.must_spawn(pro(spawner, peripherals));
}

// -------------------------------------------------------------------------------------------------

/// The main task for the protocol core.
#[embassy_executor::task]
async fn pro(_s: Spawner, _p: ProPeripherals) -> ! {
    defmt::info!("Started protocol task!");

    loop {
        Timer::after(Duration::MAX).await;
    }
}

// -------------------------------------------------------------------------------------------------

/// The [`Peripherals`](esp_hal::Peripherals) available to the protocol core.
pub(crate) struct ProPeripherals {}
