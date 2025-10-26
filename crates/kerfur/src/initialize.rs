//! The device initialization module.
//!
//! Imports required globals, initializes the allocator, and sets up the microcontroller.
#![cfg_attr(rustfmt, rustfmt_skip)]

use esp_hal::{Config, clock::CpuClock, peripherals::Peripherals};
use esp_alloc::heap_allocator;

// Use `embassy_time` for `defmt` timestamps.
use embassy_time as _;
// Use `esp_alloc` as the global allocator.
use esp_alloc as _;
// Use `esp_backtrace` as the panic handler.
use esp_backtrace as _;
// Use `esp_println` as the global logger.
use esp_println as _;

// -------------------------------------------------------------------------------------------------

/// Initialize the device.
pub(super) fn init() -> Peripherals {
    // Initialize the heap allocator
    heap_allocator!(size: 64 * 1000);

    // Initialize the microcontroller
    esp_hal::init(Config::default().with_cpu_clock(CpuClock::_240MHz))
}
