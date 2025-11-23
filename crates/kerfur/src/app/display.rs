//! TODO
#![expect(dead_code, unused_imports, reason = "Drivers have not been written yet")]

use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::{Duration, Timer};
use esp_hal::{
    dma_buffers,
    gpio::{AnyPin, Level, Output, OutputConfig},
    peripherals::DMA_CH0,
};
use kerfur_display::{KerfurDisplay, KerfurEmote};

use super::SPI;

/// A task that handles drawing to the display.
#[embassy_executor::task]
pub(super) async fn task(spi: &'static SPI, p: DisplayPeripherals<'static>) -> ! {
    defmt::info!("Preparing display...");

    let (_, _, _tx_buf, _tx_desc) = dma_buffers!(0, 8 * 3 * 480 * 480);

    // Create a ST7701S display driver
    let _device = SpiDevice::new(spi, p.display_cs);
    // let display = ST7701S::new(
    //     device,
    //     tx_buf,
    //     tx_desc,
    //     p.display_enable,
    //     p.display_backlight,
    //     p.display_clock,
    //     p.display_vsync,
    //     p.display_hsync,
    //     p.display_data,
    // )
    // .await
    // .unwrap();

    // // Wrap the display in a KerfurDisplay
    // let mut kerfur: KerfurDisplay<'static, _> = KerfurDisplay::blue(display,
    // KerfurEmote::Neutral);

    loop {
        // // Draw to the display
        // if let Err(err) = kerfur.draw(5.0) {
        //     defmt::error!("Failed to draw to display, {}", err);
        //     Timer::after_secs(5).await;
        //     defmt::warn!("Resuming display task...");
        //     continue;
        // }

        // // If the display isn't animating, wait for a new expression
        // if !kerfur.is_animating() {
        //     todo!()
        // }

        Timer::after_secs(5).await;
    }
}

// -------------------------------------------------------------------------------------------------

pub(crate) enum DisplayCommand {}

pub(super) struct DisplayPeripherals<'a> {
    pub(crate) display_dma: DMA_CH0<'a>,
    pub(crate) display_cs: AnyPin<'a>,
    pub(crate) display_enable: AnyPin<'a>,
    pub(crate) display_backlight: AnyPin<'a>,
    pub(crate) display_clock: AnyPin<'a>,
    pub(crate) display_vsync: AnyPin<'a>,
    pub(crate) display_hsync: AnyPin<'a>,
    pub(crate) display_data: [AnyPin<'a>; 16],
}
