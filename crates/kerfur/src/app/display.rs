//! TODO
#![expect(dead_code, unused_imports, reason = "Drivers have not been written yet")]

use display_interface_spi::SPIInterface;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_time::{Duration, Timer};
use esp_hal::gpio::AnyPin;
use kerfur_display::{KerfurDisplay, KerfurEmote};

use crate::{
    app::SPI,
    signal::{DISPLAY_CMD, DisplayCommand},
};

const TICKRATE: f32 = 5.0;

/// A task that handles drawing to the display.
#[embassy_executor::task]
pub(super) async fn task(spi: &'static SPI, p: DisplayPeripherals<'static>) -> ! {
    defmt::info!("Preparing display...");

    // Create a ST7701S display driver
    let _device = SPIInterface::new(SpiDevice::new(spi, p.display_cs), p.display_enable);
    // let display = ST7701S::new(
    //     device,
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
        // Wait for a display command
        let (_emote, delay) = match DISPLAY_CMD.receive().await {
            DisplayCommand::Push(emote) => (emote, Duration::MIN),
            DisplayCommand::PushHold(emote, delay) => (emote, delay),
        };

        // // Draw and animate the emote
        // kerfur.set_expression(emote);
        // while kerfur.is_animating() {
        //     // Draw to the display
        //     if let Err(err) = kerfur.draw(TICKRATE) {
        //         defmt::error!("Failed to draw to display, {}", err);
        //         Timer::after_secs(5).await;
        //         defmt::warn!("Resuming display task...");
        //     }
        // }

        Timer::after(delay).await;
    }
}

// -------------------------------------------------------------------------------------------------

pub(super) struct DisplayPeripherals<'a> {
    pub(crate) display_cs: AnyPin<'a>,
    pub(crate) display_enable: AnyPin<'a>,
    pub(crate) display_backlight: AnyPin<'a>,
    pub(crate) display_clock: AnyPin<'a>,
    pub(crate) display_vsync: AnyPin<'a>,
    pub(crate) display_hsync: AnyPin<'a>,
    pub(crate) display_data: [AnyPin<'a>; 16],
}
