//! The device configuration module.
//!
//! Imports required components, calls setup macros, and initializes the device.

use esp_alloc::heap_allocator;
use esp_hal::{Config, peripherals::Peripherals};
use esp_println::logger::init_logger_from_env;
use log::info;

// Use `embassy_time` for `defmt` timestamps.
#[rustfmt::skip]
use embassy_time as _;
// Use `esp_alloc` as the global allocator.
#[rustfmt::skip]
use esp_alloc as _;
// Use `esp_backtrace` as the panic handler.
#[rustfmt::skip]
use esp_backtrace as _;
// Use `esp_println` as the global logger.
#[rustfmt::skip]
use esp_println as _;

esp_bootloader_esp_idf::esp_app_desc!();

// -------------------------------------------------------------------------------------------------

/// Initialize the device.
pub(super) fn init() -> Peripherals {
    // Initialize the logger
    init_logger_from_env();
    info!("Initialized the logger");

    // Initialize the heap allocator
    heap_allocator!(#[unsafe(link_section = ".dram2_uninit")] size: 64 * 1000);
    info!("Initialized the heap");

    // Initialize the microcontroller
    esp_hal::init(Config::default())
}
