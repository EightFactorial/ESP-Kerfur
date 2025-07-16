//! The device configuration module.
//!
//! Imports required components, calls setup macros, and initializes the device.

use esp_alloc::heap_allocator;
use esp_hal::{Config, clock::CpuClock, peripherals::Peripherals};
use esp_println::logger::init_logger_from_env;
use log::info;

// Use `embassy_time` for `defmt` timestamps.
#[rustfmt::skip]
use embassy_time as _;
// Use `esp_backtrace` as the panic handler.
#[rustfmt::skip]
use esp_backtrace as _;
// Use `esp_alloc` as the global allocator.
#[rustfmt::skip]
use esp_alloc as _;

esp_bootloader_esp_idf::esp_app_desc!();

// Set the `defmt` panic handler.
#[defmt::panic_handler]
#[expect(clippy::panic, unused_braces)]
fn panic() -> ! { core::panic!("panic via `defmt::panic!`") }

// -------------------------------------------------------------------------------------------------

#[cfg(not(any(feature = "esp32c3", feature = "esp32c6")))]
compile_error!("Either the `esp32c3` or `esp32c6` feature must be enabled.");

#[cfg(all(feature = "esp32c3", feature = "esp32c6"))]
compile_error!("Only one of the `esp32c3` or `esp32c6` features can be enabled.");

/// Initialize the device.
pub(super) fn init() -> Peripherals {
    // Initialize the logger
    init_logger_from_env();
    info!("Initialized the logger");

    // Initialize the heap allocator
    heap_allocator!(size: 64 * 1000);
    info!("Initialized the heap");

    // Initialize the microcontroller
    esp_hal::init(Config::default().with_cpu_clock(CpuClock::_160MHz))
}
