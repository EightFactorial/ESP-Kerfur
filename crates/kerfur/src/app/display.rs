//! TODO

use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::Timer;
use esp_hal::gpio::{AnyPin, Level, Output, OutputConfig};
use kerfur_display::{KerfurDisplay, KerfurEmote};
use kerfur_gc9503::{Gc9503, Gc9503Channels, color::Rgb888};
use kerfur_tca9554::pin::TCA_P1;

use super::{AsyncI2C, SPI};

/// A task that handles drawing to the display.
#[embassy_executor::task]
pub(super) async fn task(spi: &'static SPI, p: DisplayPeripherals<'static>) -> ! {
    defmt::info!("Preparing display...");

    // Initialize GC9503 display
    let display: Gc9503<Rgb888, _, Output<'static>> = Gc9503::new(
        SpiDevice::new(spi, p.chip_select),
        Gc9503Channels {
            enable: Output::new(p.display_enable, Level::Low, OutputConfig::default()),
            p_clck: Output::new(p.display_clock, Level::Low, OutputConfig::default()),
            v_sync: Output::new(p.display_vsync, Level::Low, OutputConfig::default()),
            h_sync: Output::new(p.display_hsync, Level::Low, OutputConfig::default()),
            display: None,
            display_data: p
                .display_data
                .map(|pin| Output::new(pin, Level::Low, OutputConfig::default())),
        },
    );

    // Wrap the display in a KerfurDisplay
    let mut kerfur: KerfurDisplay<'static, _> = KerfurDisplay::blue(display, KerfurEmote::Neutral);

    loop {
        // Draw to the display
        if let Err(err) = kerfur.draw(5.0) {
            defmt::error!("Failed to draw to display, {}", err);
            Timer::after_secs(5).await;
            defmt::warn!("Resuming display task...");
            continue;
        }

        // If the display isn't animating, wait for a new expression
        if !kerfur.is_animating() {
            todo!()
        }
    }
}

// -------------------------------------------------------------------------------------------------

pub(super) struct DisplayPeripherals<'a> {
    pub(super) chip_select: TCA_P1<'a, NoopRawMutex, AsyncI2C<'a>>,
    pub(super) display_enable: AnyPin<'a>,
    pub(super) display_clock: AnyPin<'a>,
    pub(super) display_vsync: AnyPin<'a>,
    pub(super) display_hsync: AnyPin<'a>,
    pub(super) display_data: [AnyPin<'a>; 16],
}
