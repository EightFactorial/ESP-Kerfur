//! The display module. Contains all logic for managing the display.
//!
//! See [`KerfurDisplay::execute`] for the main display logic.

use alloc::string::ToString;

use embassy_executor::Spawner;
use embassy_time::Timer;
use embedded_graphics::{
    image::Image,
    mono_font::{MonoTextStyle, MonoTextStyleBuilder, ascii::FONT_10X20},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use esp_hal::{
    Blocking, DriverMode,
    gpio::{AnyPin, DriveMode, Input, InputConfig, Level, Output, OutputConfig, Pull},
    i2c::master::{AnyI2c, Config, Error as I2cError, I2c},
    rng::Rng,
    time::Rate,
};
use futures_lite::future;
use log::{error, info, warn};
use sh1106::{
    Error as DisplayError, mode::displaymode::DisplayModeTrait, prelude::*,
    properties::DisplayProperties,
};
use static_cell::make_static;
use tinybmp::Bmp;

use crate::clock::Clock;

/// Initialize the display and spawn the display manager task.
#[expect(clippy::too_many_arguments)]
pub(super) fn spawn(
    spawner: Spawner,
    touch: impl Into<AnyPin<'static>>,
    audio: impl Into<AnyPin<'static>>,
    i2c: impl Into<AnyI2c<'static>>,
    sda: impl Into<AnyPin<'static>>,
    scl: impl Into<AnyPin<'static>>,
    clock: Clock,
    rng: Rng,
) {
    // Create the I2C interface
    let config = Config::default().with_frequency(Rate::from_khz(400));
    let i2c = I2c::new(i2c.into(), config)
        .expect("Failed to create I2C interface, invalid configuration");
    let wrapper = I2cWrapper(i2c.with_sda(sda.into()).with_scl(scl.into()));
    let iface = I2cInterface::new(wrapper, 0x3C);

    // Create the kerfur display
    let properties =
        DisplayProperties::new(iface, DisplaySize::Display128x64, DisplayRotation::Rotate180);

    // Configure the touch input and audio output pins
    let touch = Input::new(touch.into(), InputConfig::default().with_pull(Pull::Down));
    let audio = Output::new(
        audio.into(),
        Level::High,
        OutputConfig::default().with_drive_mode(DriveMode::OpenDrain),
    );

    // Spawn the display manager task
    let kerfur = KerfurDisplay::new(GraphicsMode::new(properties), touch, audio, clock, rng);
    spawner.must_spawn(display_manager(kerfur));
}

/// An async task that manages the [`KerfurDisplay`].
#[embassy_executor::task]
async fn display_manager(mut kerfur: KerfurDisplay) -> ! {
    info!("Starting display manager");

    loop {
        // Execute the display loop, which will run until an error occurs.
        match kerfur.execute().await {
            Err(DisplayManagerError::DisplayI2c(err)) => error!("Display I2C error: {err:?}"),
        }

        // Wait 30 seconds before restarting the display manager
        Timer::after_secs(10).await;
        warn!("Attempting to restart display manager...");
    }
}

/// Errors that can occur in the display manager.
#[non_exhaustive]
enum DisplayManagerError {
    /// An error occurred while communicating with the display.
    DisplayI2c(I2cError),
}

impl From<I2cError> for DisplayManagerError {
    fn from(err: I2cError) -> Self { DisplayManagerError::DisplayI2c(err) }
}
impl From<DisplayError<I2cError, ()>> for DisplayManagerError {
    fn from(err: DisplayError<I2cError, ()>) -> Self {
        match err {
            DisplayError::Comm(err) => Self::from(err),
            DisplayError::Pin(()) => unreachable!(),
        }
    }
}

// -------------------------------------------------------------------------------------------------

#[non_exhaustive]
#[expect(clippy::missing_docs_in_private_items)]
struct KerfurDisplay {
    /// The display to draw to.
    display: DisplayInterface<'static>,
    /// The input that triggers a meow action.
    touch: Input<'static>,
    /// The output that triggers a meow sound.
    audio: Output<'static>,

    /// Images to be drawn on the display.
    neutral: Image<'static, Bmp<'static, BinaryColor>>,
    blink: Image<'static, Bmp<'static, BinaryColor>>,
    meow: Image<'static, Bmp<'static, BinaryColor>>,

    /// A clock used to track the current time.
    clock: Clock,
    /// A font used to display the time.
    font: MonoTextStyle<'static, BinaryColor>,

    /// A source of random numbers for timing.
    rng: Rng,
}

/// A face that Kerfur can display.
enum KerfurFace {
    /// The neutral face, displayed most of the time.
    Neutral,
    /// The neutral face with eyes closed, displayed when blinking.
    Blink,
    /// The meow face, displayed when the Kerfur meows.
    Meow,
    /// The clock face, displayed when showing the time.
    Clock,
}

/// An action that Kerfur can perform.
enum KerfurAction {
    /// Perform a blink.
    Blink,
    /// Perform a meow.
    Meow,
    /// Show the time.
    Time,
}

impl KerfurDisplay {
    /// Create a new [`KerfurDisplay`].
    ///
    /// # Panics
    /// Only one display can ever exist.
    /// Calling this function twice will immediately panic.
    #[must_use]
    fn new(
        display: DisplayInterface<'static>,
        touch: Input<'static>,
        audio: Output<'static>,
        clock: Clock,
        rng: Rng,
    ) -> Self {
        // Create static images for the frames
        let neutral = make_static!(
            Bmp::from_slice(include_bytes!("../assets/kerfur_neutral.bmp"))
                .expect("Bmp image failed to parse from slice")
        );
        let blink = make_static!(
            Bmp::from_slice(include_bytes!("../assets/kerfur_blink.bmp"))
                .expect("Bmp image failed to parse from slice")
        );
        let meow = make_static!(
            Bmp::from_slice(include_bytes!("../assets/kerfur_meow.bmp"))
                .expect("Bmp image failed to parse from slice")
        );

        Self {
            display,
            touch,
            audio,
            neutral: Image::new(neutral, Point::zero()),
            blink: Image::new(blink, Point::zero()),
            meow: Image::new(meow, Point::zero()),
            font: MonoTextStyleBuilder::new().font(&FONT_10X20).text_color(BinaryColor::On).build(),
            clock,
            rng,
        }
    }

    // ---------------------------------------------------------------------------------------------

    /// Run the kerfur display loop.
    ///
    /// This is the main loop that will run indefinitely.
    /// Contains all of the logic for what to show on the display.
    async fn execute(&mut self) -> Result<!, DisplayManagerError> {
        // Initialize the display
        self.display.init()?;
        self.display.clear();
        self.display.flush()?;

        loop {
            // Display the neutral frame
            self.display_face(KerfurFace::Neutral).await?;

            match future::or::<KerfurAction, _, _>(
                // Wait an amount of time before blinking
                async {
                    // Have a ~1/32 chance to quickly show the next frame
                    let delay = self.rng.random();
                    if delay.is_multiple_of(32) {
                        // Wait between 1.0 (0+1000) and 1.499 (499+1000) seconds
                        Timer::after_millis(u64::from(delay % 500 + 1000)).await;
                    } else {
                        // Wait between 3.0 (0+3000) and 6.999 (3999+3000) seconds
                        Timer::after_millis(u64::from(delay % 4000 + 3000)).await;
                    }

                    KerfurAction::Blink
                },
                // Wait for a button press to meow
                async {
                    // `Input::wait_for` is documented as "not cancellation-safe",
                    // but the description of why seems to imply that it is?
                    self.touch.wait_for_high().await;

                    future::or::<KerfurAction, _, _>(
                        // If the button is held, show the clock face
                        async {
                            Timer::after_millis(2000).await;
                            KerfurAction::Time
                        },
                        // If the button is released, meow
                        async {
                            self.touch.wait_for_low().await;
                            KerfurAction::Meow
                        },
                    )
                    .await
                },
            )
            .await
            {
                // Show the blink frame
                KerfurAction::Blink => {
                    self.display_face(KerfurFace::Blink).await?;
                    Timer::after_millis(100).await;
                }
                // Show the meow frame
                KerfurAction::Meow => {
                    // Play the meow sound
                    self.audio.set_low();
                    Timer::after_millis(50).await;
                    self.audio.set_high();

                    // Display the meow face briefly
                    Timer::after_millis(100).await;
                    self.display_face(KerfurFace::Meow).await?;
                    Timer::after_millis(650).await;
                }
                // Show the meow face, then the clock face
                KerfurAction::Time => {
                    // Play the meow sound
                    self.audio.set_low();
                    Timer::after_millis(50).await;
                    self.audio.set_high();

                    // Display the meow face briefly
                    Timer::after_millis(100).await;
                    self.display_face(KerfurFace::Meow).await?;
                    Timer::after_millis(650).await;

                    // Display the clock face
                    self.display_face(KerfurFace::Clock).await?;
                    Timer::after_millis(5000).await;
                    // If the button is still being held,
                    // wait for it to be released before continuing.
                    self.touch.wait_for_low().await;
                }
            }
        }
    }

    /// Display a [`KerfurFace`] on the display.
    async fn display_face(&mut self, face: KerfurFace) -> Result<(), DisplayManagerError> {
        match face {
            KerfurFace::Neutral => self.neutral.draw(&mut self.display).unwrap(),
            KerfurFace::Blink => self.blink.draw(&mut self.display).unwrap(),
            KerfurFace::Meow => self.meow.draw(&mut self.display).unwrap(),
            KerfurFace::Clock => {
                let date_time = self.clock.now().await;
                self.display.clear();

                // Draw the time
                let time_str = date_time.format("%-I:%M %p").to_string();
                Text::with_baseline(&time_str, Point::new(40, 20), self.font, Baseline::Top)
                    .draw(&mut self.display)
                    .unwrap();

                // Draw the date
                let date_str = date_time.format("%m/%d/%Y").to_string();
                Text::with_baseline(&date_str, Point::new(10, 35), self.font, Baseline::Top)
                    .draw(&mut self.display)
                    .unwrap();
            }
        }
        self.display.flush()?;
        Ok(())
    }
}

/// An alias for this abomination of a display type.
type DisplayInterface<'d> = GraphicsMode<I2cInterface<I2cWrapper<'d, Blocking>>>;

// -------------------------------------------------------------------------------------------------

/// A wrapper for compatibility between [`esp_hal`]'s
/// i2c driver and legacy [`embedded_hal`]'s i2c traits.
///
/// # Note
/// This will no longer be necessary once [`sh1106`]
/// updates to depend on [`embedded_hal`] `v1.0.0`.
#[repr(transparent)]
struct I2cWrapper<'d, Dm: DriverMode>(I2c<'d, Dm>);

impl<Dm: DriverMode> embedded_hal::blocking::i2c::Write for I2cWrapper<'_, Dm> {
    type Error = I2cError;

    #[inline]
    fn write(&mut self, address: u8, buffer: &[u8]) -> Result<(), Self::Error> {
        self.0.write::<u8>(address, buffer)
    }
}
