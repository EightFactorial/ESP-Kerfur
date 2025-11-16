//! TODO
#![expect(clippy::used_underscore_binding, reason = "Macros use underscore bindings internally")]
#![no_main]
#![no_std]

use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::{Duration, Timer};
use esp_hal::{
    Async,
    i2c::master::{Config as I2cConfig, I2c},
    spi::master::{Config as SpiConfig, Spi},
};
use kerfur_tca9554::Tca9554;
use static_cell::StaticCell;

mod initialize;

#[esp_rtos::main]
async fn main(_s: Spawner) -> ! {
    static I2C: StaticCell<Mutex<NoopRawMutex, I2c<'static, Async>>> = StaticCell::new();
    static TCA: StaticCell<Tca9554<'static, NoopRawMutex, I2c<'static, Async>>> = StaticCell::new();
    static SPI: StaticCell<Mutex<NoopRawMutex, Spi<'static, Async>>> = StaticCell::new();

    // Initialize the device
    let p = initialize::init();

    // Initialize I2C
    let mut i2c = I2c::new(p.I2C0, I2cConfig::default()).unwrap();
    i2c = i2c.with_sda(p.GPIO8).with_scl(p.GPIO18);
    let i2c = I2C.init(Mutex::new(i2c.into_async()));

    // Initialize TCA9554 IO expander
    let tca = Tca9554::<NoopRawMutex, I2c<'static, Async>>::new(i2c, 0x0);
    let _tca = TCA.init(tca);

    // Initialize SPI using TCA9554 pins
    let spi = Spi::new(p.SPI2, SpiConfig::default()).unwrap();
    // spi = spi.with_sck(tca.p2).with_cs(tca.p1).with_mosi(tca.p3);
    let _spi = SPI.init(Mutex::new(spi.into_async()));

    loop {
        Timer::after(Duration::MAX).await;
    }
}
