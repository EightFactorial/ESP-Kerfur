//! The second, "app" processor core
//!
//! Cannot access values outside of this module.

use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::{Delay, Duration, Timer};
use embedded_hal::spi::MODE_0;
use esp_hal::{
    Async,
    gpio::AnyPin,
    i2c::master::{AnyI2c, Config as I2cConfig, I2c},
    system::{AppCoreGuard, CpuControl, Stack},
};
use esp_rtos::embassy::Executor;
use kerfur_bitbang::BitBangSpi;
use kerfur_tca9554::{
    Tca9554,
    pin::{TCA_P2, TCA_P3},
};
use static_cell::StaticCell;

mod display;
mod display_touch;

/// Create an [`Executor`] and run tasks on the application core.
#[expect(static_mut_refs, reason = "Required to access mutable statics")]
pub(super) fn spawn(mut control: CpuControl, peripherals: AppPeripherals<'static>) {
    static GUARD: StaticCell<AppCoreGuard<'static>> = StaticCell::new();
    static mut STACK: Stack<{ 1024 * 16 }> = Stack::new();
    static mut EXECUTOR: Executor = Executor::new();

    defmt::info!("Starting application core...");

    // SAFETY: We have exclusive access to the variables,
    // and consume `CpuControl` so this cannot be called twice.
    unsafe {
        GUARD.init(defmt::unwrap!(control.start_app_core(&mut STACK, || {
            EXECUTOR.run(|spawner: Spawner| {
                defmt::info!("Spawning application core task...");
                spawner.must_spawn(app(spawner, peripherals));
            })
        })));
    }
}

// -------------------------------------------------------------------------------------------------

type AsyncI2C<'a> = I2c<'a, Async>;
type NMutex<T> = Mutex<NoopRawMutex, T>;

type I2C = NMutex<AsyncI2C<'static>>;
type TCA = Tca9554<'static, NoopRawMutex, AsyncI2C<'static>>;
type SPI = NMutex<
    BitBangSpi<
        TCA_P2<'static, NoopRawMutex, AsyncI2C<'static>>,
        TCA_P3<'static, NoopRawMutex, AsyncI2C<'static>>,
        Delay,
    >,
>;

/// The main task for the application core.
#[embassy_executor::task]
async fn app(s: Spawner, p: AppPeripherals<'static>) -> ! {
    static I2C: StaticCell<I2C> = StaticCell::new();
    static TCA: StaticCell<TCA> = StaticCell::new();
    static SPI: StaticCell<SPI> = StaticCell::new();

    defmt::info!("Started application task!");

    // Initialize I2C
    defmt::info!("Initializing I2C...");
    let i2c = defmt::unwrap!(I2c::new(p.i2c, I2cConfig::default()));
    let i2c = i2c.with_sda(p.i2c_sda).with_scl(p.i2c_scl).into_async();
    let i2c = I2C.init(Mutex::new(i2c));

    // Initialize TCA9554
    defmt::info!("Initializing TCA9554...");
    let tca = TCA.init(Tca9554::new(i2c, 0x0));

    // Initialize SPI using TCA9554 pins
    defmt::info!("Initializing SPI...");
    let spi = BitBangSpi::new(tca.p2.reborrow(), tca.p3.reborrow(), Delay, 5, MODE_0);
    let spi = SPI.init(Mutex::new(spi));

    // Spawn the display task
    s.must_spawn(display::task(
        spi,
        display::DisplayPeripherals {
            chip_select: tca.p1.reborrow(),
            display_enable: p.display_enable,
            display_clock: p.display_clock,
            display_vsync: p.display_vsync,
            display_hsync: p.display_hsync,
            display_data: p.display_data,
        },
    ));

    // Spawn the display touch task
    s.must_spawn(display_touch::task(i2c));

    loop {
        Timer::after(Duration::MAX).await;
    }
}

// -------------------------------------------------------------------------------------------------

/// The [`Peripherals`](esp_hal::Peripherals) available to the application core.
pub(crate) struct AppPeripherals<'a> {
    // I2C, SDA, and SCL
    pub(crate) i2c: AnyI2c<'a>,
    pub(crate) i2c_sda: AnyPin<'a>,
    pub(crate) i2c_scl: AnyPin<'a>,

    // SPI Display
    pub(crate) display_enable: AnyPin<'a>,
    pub(crate) display_clock: AnyPin<'a>,
    pub(crate) display_vsync: AnyPin<'a>,
    pub(crate) display_hsync: AnyPin<'a>,
    pub(crate) display_data: [AnyPin<'a>; 16],
}
