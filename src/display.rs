//! TODO

use embassy_executor::Spawner;
use embassy_time::Timer;
use embedded_graphics::{image::Image, pixelcolor::BinaryColor, prelude::*};
use esp_hal::{
    Blocking, DriverMode,
    gpio::AnyPin,
    i2c::{
        AnyI2c,
        master::{Config, Error as I2cError, I2c},
    },
    rng::Rng,
    time::Rate,
};
use log::{error, info, warn};
use sh1106::{
    Error as DisplayError, mode::displaymode::DisplayModeTrait, prelude::*,
    properties::DisplayProperties,
};
use static_cell::make_static;
use tinybmp::Bmp;

/// Initialize the display and spawn the display manager task.
pub(super) fn spawn(
    spawner: Spawner,
    rng: Rng,
    i2c: impl Into<AnyI2c<'static>>,
    sda: impl Into<AnyPin<'static>>,
    scl: impl Into<AnyPin<'static>>,
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
    let kerfur = KerfurDisplay::new(GraphicsMode::new(properties), rng);

    // Spawn the display manager task
    spawner.must_spawn(display_manager(kerfur));
}

/// An async task that manages the [`KerfurDisplay`].
#[embassy_executor::task]
async fn display_manager(mut kerfur: KerfurDisplay) -> ! {
    info!("Starting display manager");

    loop {
        // Execute the display loop, which will run until an error occurs.
        #[expect(clippy::match_single_binding)]
        match kerfur.execute().await {
            Err(DisplayManagerError::DisplayI2c(err)) => match err {
                err => error!("Display I2C error: {err:?}"),
            },
        }

        // Wait 30 seconds before restarting the display manager
        Timer::after_secs(30).await;
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

    /// Images to be drawn on the display.
    neutral: Image<'static, Bmp<'static, BinaryColor>>,
    blink: Image<'static, Bmp<'static, BinaryColor>>,
    meow: Image<'static, Bmp<'static, BinaryColor>>,

    /// A source of random numbers for timing.
    rng: Rng,
}

/// A face that the Kerfur can display.
enum KerfurFace {
    /// The neutral face, displayed most of the time.
    Neutral,
    /// The neutral face with eyes closed, displayed when blinking.
    Blink,
    /// The meow face, displayed when the Kerfur meows.
    Meow,
}

impl KerfurDisplay {
    /// Create a new [`KerfurDisplay`].
    ///
    /// # Panics
    /// Only one display can ever exist.
    /// Calling this function twice will immediately panic.
    #[must_use]
    fn new(display: DisplayInterface<'static>, rng: Rng) -> Self {
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
            neutral: Image::new(neutral, Point::zero()),
            blink: Image::new(blink, Point::zero()),
            meow: Image::new(meow, Point::zero()),
            rng,
        }
    }

    /// Display a [`KerfurFace`] on the display.
    fn display_face(&mut self, face: KerfurFace) -> Result<(), DisplayManagerError> {
        match face {
            KerfurFace::Neutral => self.neutral.draw(&mut self.display).unwrap(),
            KerfurFace::Blink => self.blink.draw(&mut self.display).unwrap(),
            KerfurFace::Meow => self.meow.draw(&mut self.display).unwrap(),
        }
        self.display.flush()?;
        Ok(())
    }

    // ---------------------------------------------------------------------------------------------

    /// Initialize and clear the display.
    fn init(&mut self) -> Result<(), DisplayManagerError> {
        self.display.init()?;
        self.display.clear();
        self.display.flush()?;
        Ok(())
    }

    /// Run the kerfur display loop.
    async fn execute(&mut self) -> Result<!, DisplayManagerError> {
        // Initialize the display
        self.init()?;

        loop {
            // Display the neutral frame
            self.display_face(KerfurFace::Neutral)?;

            // Wait before showing the next frame
            {
                // Have a ~1/32 chance to short blink
                let delay = self.rng.random();
                if delay.is_multiple_of(32) {
                    // Wait between 1.0 (0+1000) and 1.099 (99+1000) seconds before the next frame
                    Timer::after_millis(u64::from(delay % 100 + 1000)).await;
                } else {
                    // Wait between 3.0 (0+3000) and 6.999 (3999+3000) seconds before the next frame
                    Timer::after_millis(u64::from(delay % 4000 + 3000)).await;
                }
            }

            // Show the blink or meow frame
            {
                // Have a ~1/16 chance to meow instead of blink
                if self.rng.random().is_multiple_of(16) {
                    // Display the meow frame
                    self.display_face(KerfurFace::Meow)?;
                    Timer::after_millis(500).await;
                } else {
                    // Display the blink frame
                    self.display_face(KerfurFace::Blink)?;
                    Timer::after_millis(100).await;
                }
            }
        }
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
struct I2cWrapper<'d, Dm: DriverMode>(I2c<'d, Dm>);

impl<Dm: DriverMode> embedded_hal::blocking::i2c::Write for I2cWrapper<'_, Dm> {
    type Error = I2cError;

    fn write(&mut self, address: u8, buffer: &[u8]) -> Result<(), Self::Error> {
        self.0.write::<u8>(address, buffer)
    }
}
