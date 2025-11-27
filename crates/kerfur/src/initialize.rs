//! The device initialization module.
//!
//! Imports required globals, initializes the allocator, and sets up the microcontroller.
#![cfg_attr(rustfmt, rustfmt_skip)]

use esp_hal::{Config, clock::CpuClock, peripherals::Peripherals, psram::{PsramConfig, SpiRamFreq}};
use esp_alloc::psram_allocator;

// Use `embassy_time` for `defmt` timestamps.
use embassy_time as _;
// Use `esp_alloc` as the global allocator.
use esp_alloc as _;
// Use `esp_backtrace` as the panic handler.
use esp_backtrace as _;
// Use `esp_println` as the global logger.
use esp_println as _;

// Embed the application descriptor
esp_bootloader_esp_idf::esp_app_desc!();

// -------------------------------------------------------------------------------------------------

/// Initialize the device.
pub(super) fn init() -> Peripherals {
    // Initialize the microcontroller
    let config = PsramConfig { ram_frequency: SpiRamFreq::Freq80m,  ..PsramConfig::default() };
    let peripherals = esp_hal::init(Config::default().with_cpu_clock(CpuClock::_240MHz).with_psram(config));

    // Initialize the psram allocator
    psram_allocator!(peripherals.PSRAM, esp_hal::psram);

    peripherals
}
