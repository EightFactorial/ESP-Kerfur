//! The second, "app" processor core
//!
//! Cannot access values outside of this module.

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::system::{AppCoreGuard, CpuControl, Stack};
use esp_rtos::embassy::Executor;
use static_cell::StaticCell;

mod display;

/// Create an [`Executor`] and run tasks on the application core.
#[expect(static_mut_refs, reason = "Required to access mutable statics")]
pub(super) fn spawn(mut control: CpuControl, peripherals: AppPeripherals) {
    static GUARD: StaticCell<AppCoreGuard<'static>> = StaticCell::new();
    static mut STACK: Stack<{ 1024 * 16 }> = Stack::new();
    static mut EXECUTOR: Executor = Executor::new();

    defmt::info!("Starting application core...");

    // SAFETY: We have exclusive access to the variables,
    // and consume `CpuControl` so this cannot be called twice.
    unsafe {
        GUARD.init(defmt::unwrap!(control.start_app_core(&mut STACK, || {
            EXECUTOR.run(|s: Spawner| {
                defmt::info!("Spawning application core task...");
                s.must_spawn(app(s, peripherals));
            })
        })));
    }
}

// -------------------------------------------------------------------------------------------------

/// The main task for the application core.
#[embassy_executor::task]
async fn app(s: Spawner, _p: AppPeripherals) -> ! {
    defmt::info!("Started application task!");

    // Spawn the display task
    s.must_spawn(display::task());

    loop {
        Timer::after(Duration::MAX).await;
    }
}

// -------------------------------------------------------------------------------------------------

/// The [`Peripherals`](esp_hal::Peripherals) available to the application core.
pub(crate) struct AppPeripherals {}
