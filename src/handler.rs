//! TODO

use esp_alloc::heap_allocator;
use esp_hal::{Config, clock::CpuClock, peripherals::Peripherals};
use esp_println::logger::init_logger_from_env;
use log::info;

// Use `esp_backtrace` as the panic handler.
#[rustfmt::skip]
use esp_backtrace as _;
// Use `esp_alloc` as the global allocator.
#[rustfmt::skip]
use esp_alloc as _;

// Set the `defmt` panic handler.
#[defmt::panic_handler]
#[expect(clippy::panic, unused_braces)]
fn panic() -> ! { core::panic!("panic via `defmt::panic!`") }

// -------------------------------------------------------------------------------------------------

/// Initialize the device.
pub(super) fn init() -> Peripherals {
    // Initialize the logger
    init_logger_from_env();
    info!("Initialized the logger");

    // Initialize the heap allocator
    heap_allocator!(size: 64 * 1000);
    info!("Initialized the heap");

    // Initialize the microcontroller
    esp_hal::init(Config::default().with_cpu_clock(CpuClock::_80MHz))
}
