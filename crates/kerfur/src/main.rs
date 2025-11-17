//! TODO
#![expect(clippy::used_underscore_binding, reason = "Macros use underscore bindings internally")]
#![no_main]
#![no_std]

use embassy_executor::Spawner;
use esp_hal::{system::CpuControl, timer::timg::TimerGroup};

use crate::{app::AppPeripherals, pro::ProPeripherals};

mod app;
mod initialize;
mod pro;

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    // Initialize the device
    let peripherals = initialize::init();
    defmt::info!("Initialized peripherals");

    // Start the scheduler
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);
    defmt::info!("Started scheduler");

    // Create the CpuControl and peripheral structs
    let control = CpuControl::new(peripherals.CPU_CTRL);
    let (app, pro) = (AppPeripherals {}, ProPeripherals {});

    // Start the app task
    app::spawn(control, app);
    // Start the pro task
    pro::spawn(spawner, pro);
}
