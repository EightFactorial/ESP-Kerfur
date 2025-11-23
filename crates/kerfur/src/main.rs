//! TODO
#![expect(clippy::used_underscore_binding, reason = "Macros use underscore bindings internally")]
#![no_main]
#![no_std]

use embassy_executor::Spawner;
use esp_hal::{interrupt::software::SoftwareInterruptControl, timer::timg::TimerGroup};

use crate::{app::AppPeripherals, pro::ProPeripherals};

mod app;
mod initialize;
mod pro;
mod signal;
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

    // Create peripheral structs
    let (app, pro) = (
        AppPeripherals {
            i2c: peripherals.I2C0.into(),
            i2c_sda: peripherals.GPIO19.into(),
            i2c_scl: peripherals.GPIO45.into(),

            spi: peripherals.SPI2.into(),
            spi_sclk: peripherals.GPIO48.into(),
            spi_mosi: peripherals.GPIO47.into(),
            spi_miso: peripherals.GPIO41.into(),
            display_cs: peripherals.GPIO39.into(),
            _sdcard_cs: peripherals.GPIO42.into(),

            display_dma: peripherals.DMA_CH0,
            display_enable: peripherals.GPIO18.into(),
            display_backlight: peripherals.GPIO38.into(),
            display_clock: peripherals.GPIO21.into(),
            display_vsync: peripherals.GPIO17.into(),
            display_hsync: peripherals.GPIO16.into(),
            display_data: [
                peripherals.GPIO11.into(),
                peripherals.GPIO12.into(),
                peripherals.GPIO13.into(),
                peripherals.GPIO14.into(),
                peripherals.GPIO0.into(),
                peripherals.GPIO8.into(),
                peripherals.GPIO20.into(),
                peripherals.GPIO3.into(),
                peripherals.GPIO46.into(),
                peripherals.GPIO9.into(),
                peripherals.GPIO10.into(),
                peripherals.GPIO4.into(),
                peripherals.GPIO5.into(),
                peripherals.GPIO6.into(),
                peripherals.GPIO7.into(),
                peripherals.GPIO15.into(),
            ],
        },
        ProPeripherals {
            i2s: peripherals.I2S0.into(),
            i2s_dma: peripherals.DMA_CH1,
            // i2s_sclock: peripherals.GPIO16.into(),
            // i2s_mclock: peripherals.GPIO5.into(),
            // i2s_lclock: peripherals.GPIO7.into(),
            // i2s_dataout: peripherals.GPIO6.into(),
            // i2s_soundin: peripherals.GPIO15.into(),
        },
    );

    // Start the app task
    let int = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    app::spawn(peripherals.CPU_CTRL, int.software_interrupt0, int.software_interrupt1, app);

    // Start the pro task
    pro::spawn(spawner, pro);
}
