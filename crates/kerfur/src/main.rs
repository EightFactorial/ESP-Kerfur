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
mod utility;

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
    let (app, pro) = (
        AppPeripherals {
            i2c: peripherals.I2C0.into(),
            i2c_sda: peripherals.GPIO8.into(),
            i2c_scl: peripherals.GPIO18.into(),

            display_enable: peripherals.GPIO17.into(),
            display_clock: peripherals.GPIO9.into(),
            display_vsync: peripherals.GPIO3.into(),
            display_hsync: peripherals.GPIO46.into(),
            display_data: [
                peripherals.GPIO10.into(),
                peripherals.GPIO11.into(),
                peripherals.GPIO12.into(),
                peripherals.GPIO13.into(),
                peripherals.GPIO14.into(),
                peripherals.GPIO21.into(),
                peripherals.GPIO47.into(),
                peripherals.GPIO48.into(),
                peripherals.GPIO45.into(),
                peripherals.GPIO38.into(),
                peripherals.GPIO39.into(),
                peripherals.GPIO40.into(),
                peripherals.GPIO41.into(),
                peripherals.GPIO42.into(),
                peripherals.GPIO1.into(),
                peripherals.GPIO2.into(),
            ],
        },
        ProPeripherals {},
    );

    // Start the app task
    app::spawn(control, app);
    // Start the pro task
    pro::spawn(spawner, pro);
}
