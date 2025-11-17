// use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
// use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
// use embassy_time::Delay;
// use embedded_hal::spi::MODE_0;
// use esp_hal::{
//     Async,
//     gpio::{Level, Output, OutputConfig},
//     i2c::master::{Config as I2cConfig, I2c},
// };
// use kerfur_bitbang::BitBangSpi;
// use kerfur_gc9503::{Gc9503, Gc9503Channels, color::Rgb888};
// use kerfur_tca9554::{
//     Tca9554,
//     pin::{TCA_P2, TCA_P3},
// };
// use static_cell::StaticCell;

#[embassy_executor::task]
pub(super) async fn task() {
    // static I2C: StaticCell<Mutex<NoopRawMutex, I2c<'static, Async>>> = StaticCell::new();
    // static TCA: StaticCell<Tca9554<'static, NoopRawMutex, I2c<'static, Async>>> = StaticCell::new();
    // static SPI: StaticCell<
    //     Mutex<
    //         NoopRawMutex,
    //         BitBangSpi<
    //             TCA_P2<NoopRawMutex, I2c<'static, Async>>,
    //             TCA_P3<NoopRawMutex, I2c<'static, Async>>,
    //             Delay,
    //         >,
    //     >,
    // > = StaticCell::new();

    // // Initialize I2C
    // let mut i2c = I2c::new(peripherals.I2C0, I2cConfig::default()).unwrap();
    // i2c = i2c.with_sda(peripherals.GPIO8).with_scl(peripherals.GPIO18);
    // let i2c = I2C.init(Mutex::new(i2c.into_async()));

    // // Initialize TCA9554 IO expander
    // let tca = Tca9554::new(i2c, 0x0);
    // let tca = TCA.init(tca);

    // // Bitbang SPI using TCA9554 pins
    // let spi = BitBangSpi::new(tca.p2.reborrow(), tca.p3.reborrow(), Delay, 5, MODE_0);
    // let spi = SPI.init(Mutex::new(spi));

    // // Initialize GC9503 display
    // let _display = Gc9503::<Rgb888, _, _, _>::new(
    //     SpiDevice::new(spi, tca.p1.reborrow()),
    //     Output::new(peripherals.GPIO0, Level::Low, OutputConfig::default()),
    //     Gc9503Channels {
    //         enable: Output::new(peripherals.GPIO17, Level::Low, OutputConfig::default()),
    //         p_clck: Output::new(peripherals.GPIO9, Level::Low, OutputConfig::default()),
    //         v_sync: Output::new(peripherals.GPIO3, Level::Low, OutputConfig::default()),
    //         h_sync: Output::new(peripherals.GPIO46, Level::Low, OutputConfig::default()),
    //         display: None,
    //         display_data: [
    //             Output::new(peripherals.GPIO10, Level::Low, OutputConfig::default()),
    //             Output::new(peripherals.GPIO11, Level::Low, OutputConfig::default()),
    //             Output::new(peripherals.GPIO12, Level::Low, OutputConfig::default()),
    //             Output::new(peripherals.GPIO13, Level::Low, OutputConfig::default()),
    //             Output::new(peripherals.GPIO14, Level::Low, OutputConfig::default()),
    //             Output::new(peripherals.GPIO21, Level::Low, OutputConfig::default()),
    //             Output::new(peripherals.GPIO47, Level::Low, OutputConfig::default()),
    //             Output::new(peripherals.GPIO48, Level::Low, OutputConfig::default()),
    //             Output::new(peripherals.GPIO45, Level::Low, OutputConfig::default()),
    //             Output::new(peripherals.GPIO38, Level::Low, OutputConfig::default()),
    //             Output::new(peripherals.GPIO39, Level::Low, OutputConfig::default()),
    //             Output::new(peripherals.GPIO40, Level::Low, OutputConfig::default()),
    //             Output::new(peripherals.GPIO41, Level::Low, OutputConfig::default()),
    //             Output::new(peripherals.GPIO42, Level::Low, OutputConfig::default()),
    //             Output::new(peripherals.GPIO1, Level::Low, OutputConfig::default()),
    //             Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default()),
    //         ],
    //     },
    // );
}
