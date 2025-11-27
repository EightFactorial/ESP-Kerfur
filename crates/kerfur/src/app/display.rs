//! TODO
#![expect(dead_code, reason = "Drivers have not been written yet")]

use ef_st7701s::{
    St7701s,
    color::Rgb565,
    interface::{Generic16BitBus, ParallelInterface},
};
use embassy_time::{Delay, Duration, Timer};
use esp_hal::{
    gpio::{AnyPin, Level, NoPin, Output, OutputConfig},
    mcpwm::{McPwm, PeripheralClockConfig, operator::PwmPinConfig, timer::PwmWorkingMode},
    peripherals::MCPWM0,
    time::Rate,
};
use kerfur_display::{KerfurDisplay, KerfurEmote};

use crate::{
    app::SPI,
    signal::{DISPLAY_CMD, DisplayCommand},
};

const TICKRATE: f32 = 5.0;

/// A task that handles drawing to the display.
#[embassy_executor::task]
pub(super) async fn task(spi: &'static SPI, mut p: DisplayPeripherals<'static>) -> ! {
    defmt::info!("Preparing display...");

    // Create a ST7701S display driver
    let [p0, p1, p2, p3, p4, p5, p6, p7, p8, p9, p10, p11, p12, p13, p14, p15] = p.display_data;
    let display = St7701s::<Rgb565, _, _>::new(
        ParallelInterface::new(
            Generic16BitBus::new((
                Output::new(p0, Level::Low, OutputConfig::default()),
                Output::new(p1, Level::Low, OutputConfig::default()),
                Output::new(p2, Level::Low, OutputConfig::default()),
                Output::new(p3, Level::Low, OutputConfig::default()),
                Output::new(p4, Level::Low, OutputConfig::default()),
                Output::new(p5, Level::Low, OutputConfig::default()),
                Output::new(p6, Level::Low, OutputConfig::default()),
                Output::new(p7, Level::Low, OutputConfig::default()),
                Output::new(p8, Level::Low, OutputConfig::default()),
                Output::new(p9, Level::Low, OutputConfig::default()),
                Output::new(p10, Level::Low, OutputConfig::default()),
                Output::new(p11, Level::Low, OutputConfig::default()),
                Output::new(p12, Level::Low, OutputConfig::default()),
                Output::new(p13, Level::Low, OutputConfig::default()),
                Output::new(p14, Level::Low, OutputConfig::default()),
                Output::new(p15, Level::Low, OutputConfig::default()),
            )),
            Output::new(p.display_de, Level::High, OutputConfig::default()),
            NoPin,
        ),
        NoPin,
        &mut Delay,
    );

    // Wrap the display in a KerfurDisplay
    let mut kerfur: KerfurDisplay<'static, _> =
        KerfurDisplay::blue_565(display.release(), KerfurEmote::Neutral);
    if kerfur.draw(TICKRATE).is_err() {
        defmt::warn!("Failed to draw initial frame, ignoring...");
    }

    // Enable the backlight
    let _backlight = Output::new(p.display_backlight, Level::High, OutputConfig::default());

    defmt::info!("Display initialized!");
    loop {
        // Wait for a display command
        let (emote, delay) = match DISPLAY_CMD.receive().await {
            DisplayCommand::Push(emote) => (emote, Duration::MIN),
            DisplayCommand::PushHold(emote, delay) => (emote, delay),
        };

        // Draw and animate the emote
        kerfur.set_expression(emote);
        while kerfur.is_animating() {
            // Draw to the display
            if kerfur.draw(TICKRATE).is_err() {
                defmt::error!("Failed to draw to display, waiting before retrying...");
                Timer::after_secs(5).await;
                defmt::warn!("Resuming display task...");
            }
        }

        Timer::after(delay).await;
    }
}

// -------------------------------------------------------------------------------------------------

pub(super) struct DisplayPeripherals<'a> {
    pub(crate) display_cs: AnyPin<'a>,
    pub(crate) display_de: AnyPin<'a>,
    pub(crate) display_pwm: McPwm<'a, MCPWM0<'a>>,
    pub(crate) display_backlight: AnyPin<'a>,
    pub(crate) display_clock: AnyPin<'a>,
    pub(crate) display_vsync: AnyPin<'a>,
    pub(crate) display_hsync: AnyPin<'a>,
    pub(crate) display_data: [AnyPin<'a>; 16],
}
