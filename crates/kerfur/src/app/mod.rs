//! The second, "app" processor core
//!
//! Cannot access values outside of this module.

use core::cell::RefCell;

use embassy_executor::Spawner;
use embassy_sync::{
    blocking_mutex::{Mutex as BlockingMutex, raw::NoopRawMutex},
    mutex::Mutex as AsyncMutex,
};
use embassy_time::Timer;
use esp_hal::{
    Async, Blocking,
    gpio::AnyPin,
    i2c::master::{AnyI2c, Config as I2cConfig, I2c},
    interrupt::software::SoftwareInterrupt,
    mcpwm::McPwm,
    peripherals::{CPU_CTRL, MCPWM0},
    spi::{
        Mode,
        master::{AnySpi, Config as SpiConfig, Spi},
    },
    system::Stack,
    time::Rate,
};
use esp_rtos::embassy::Executor;
use static_cell::StaticCell;

mod audio;
mod display;
mod touch;

/// Create an [`Executor`] and run tasks on the application core.
#[expect(static_mut_refs, reason = "Required to access mutable statics")]
pub(super) fn spawn(
    control: CPU_CTRL<'static>,
    int0: SoftwareInterrupt<'static, 0>,
    int1: SoftwareInterrupt<'static, 1>,
    peripherals: AppPeripherals<'static>,
) {
    static mut STACK: Stack<{ 16 * 1024 }> = Stack::new();
    static mut EXECUTOR: Executor = Executor::new();

    defmt::info!("Starting application core...");

    // SAFETY: We have exclusive access to the variables,
    // and consume peripherals so this cannot be called twice.
    unsafe {
        esp_rtos::start_second_core(control, int0, int1, &mut STACK, || {
            EXECUTOR.run(|spawner: Spawner| {
                defmt::info!("Spawning application core task...");
                spawner.must_spawn(app(spawner, peripherals));
            })
        });
    }
}

// -------------------------------------------------------------------------------------------------

type AMutex<T> = AsyncMutex<NoopRawMutex, T>;
type BMutex<T> = BlockingMutex<NoopRawMutex, T>;

type I2C = AMutex<I2c<'static, Async>>;
type SPI = BMutex<RefCell<Spi<'static, Blocking>>>;

/// The main task for the application core.
#[embassy_executor::task]
async fn app(s: Spawner, p: AppPeripherals<'static>) -> ! {
    static I2C: StaticCell<I2C> = StaticCell::new();
    static SPI: StaticCell<SPI> = StaticCell::new();

    defmt::info!("Started application task!");

    // Initialize I2C
    defmt::info!("Initializing I2C...");
    let config = I2cConfig::default().with_frequency(Rate::from_khz(100));
    let i2c = defmt::unwrap!(I2c::new(p.i2c, config));
    let i2c = i2c.with_sda(p.i2c_sda).with_scl(p.i2c_scl).into_async();
    let i2c = I2C.init(AsyncMutex::new(i2c));

    // Initialize SPI
    defmt::info!("Initializing SPI...");
    let config = SpiConfig::default().with_frequency(Rate::from_mhz(2)).with_mode(Mode::_3);
    let spi = defmt::unwrap!(Spi::new(p.spi, config));
    let spi = spi.with_sck(p.spi_sclk).with_mosi(p.spi_mosi).with_miso(p.spi_miso);
    let spi = SPI.init(BlockingMutex::new(RefCell::new(spi)));

    // Spawn the audio config task
    s.must_spawn(audio::task(i2c));

    // Spawn the display manager task
    s.must_spawn(display::task(
        spi,
        display::DisplayPeripherals {
            display_cs: p.display_cs,
            display_de: p.display_de,
            display_pwm: p.display_pwm,
            display_backlight: p.display_backlight,
            display_clock: p.display_clock,
            display_vsync: p.display_vsync,
            display_hsync: p.display_hsync,
            display_data: p.display_data,
        },
    ));

    // Spawn the touch sensor task
    s.must_spawn(touch::task(i2c));

    loop {
        Timer::after_secs(30).await;
    }
}

// -------------------------------------------------------------------------------------------------

/// The [`Peripherals`](esp_hal::Peripherals) available to the application core.
pub(crate) struct AppPeripherals<'a> {
    // I2C for ADC, DAC, and Touch Sensor
    pub(crate) i2c: AnyI2c<'a>,
    pub(crate) i2c_sda: AnyPin<'a>,
    pub(crate) i2c_scl: AnyPin<'a>,

    // SPI for SD Card and Display
    pub(crate) spi: AnySpi<'a>,
    pub(crate) spi_sclk: AnyPin<'a>,
    pub(crate) spi_mosi: AnyPin<'a>,
    pub(crate) spi_miso: AnyPin<'a>,
    pub(crate) display_cs: AnyPin<'a>,
    pub(crate) _sdcard_cs: AnyPin<'a>,

    // ST7701S Display
    pub(crate) display_de: AnyPin<'a>,
    pub(crate) display_pwm: McPwm<'a, MCPWM0<'a>>,
    pub(crate) display_backlight: AnyPin<'a>,
    pub(crate) display_clock: AnyPin<'a>,
    pub(crate) display_vsync: AnyPin<'a>,
    pub(crate) display_hsync: AnyPin<'a>,
    pub(crate) display_data: [AnyPin<'a>; 16],
}
