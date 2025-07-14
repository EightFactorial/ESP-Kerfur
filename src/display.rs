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
use log::{error, info};
use sh1106::{
    displayrotation::DisplayRotation,
    displaysize::DisplaySize,
    interface::I2cInterface,
    mode::{GraphicsMode, displaymode::DisplayModeTrait},
    properties::DisplayProperties,
};
use static_cell::make_static;
use tinybmp::Bmp;

/// Initialize the display and spawn the display manager task.
#[expect(clippy::unwrap_used)]
pub(super) fn spawn(
    spawner: Spawner,
    rng: Rng,
    i2c: impl Into<AnyI2c<'static>>,
    sda: impl Into<AnyPin<'static>>,
    scl: impl Into<AnyPin<'static>>,
) {
    // Create the I2C interface
    let config = Config::default().with_frequency(Rate::from_khz(400));
    let i2c = I2c::new(i2c.into(), config).unwrap().with_sda(sda.into()).with_scl(scl.into());

    // Create the display
    let iface = I2cInterface::new(I2cWrapper(i2c), 0x3C);
    let properties =
        DisplayProperties::new(iface, DisplaySize::Display128x64, DisplayRotation::Rotate180);
    let display = KerfurDisplay::new(GraphicsMode::new(properties), rng);

    // Spawn the display manager task
    spawner.must_spawn(display_manager(display));
}

/// An async task that manages the [`KerfurDisplay`].
///
/// TODO: Error handling and auto-recovery.
#[embassy_executor::task]
async fn display_manager(mut kerfur: KerfurDisplay) -> ! {
    info!("Starting display manager");

    loop {
        #[expect(clippy::match_single_binding)]
        match kerfur.execute().await {
            Err(DisplayManagerError::I2C(err)) => match err {
                err => error!("Display I2C error: {err:?}"),
            },
        }
    }
}

/// An alias for this abomination of a display type.
type DisplayInterface<'d> = GraphicsMode<I2cInterface<I2cWrapper<'d, Blocking>>>;

/// Errors that can occur in the display manager.
#[non_exhaustive]
enum DisplayManagerError {
    /// An [`I2cError`] occurred while communicating with the display.
    I2C(I2cError),
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
    #[expect(clippy::unwrap_used)]
    fn new(mut display: DisplayInterface<'static>, rng: Rng) -> Self {
        // Initialize and flush the display
        display.init().unwrap();
        display.flush().unwrap();

        // Create static images for the frames
        let neutral =
            make_static!(Bmp::from_slice(include_bytes!("../assets/kerfur_neutral.bmp")).unwrap());
        let blink =
            make_static!(Bmp::from_slice(include_bytes!("../assets/kerfur_blink.bmp")).unwrap());
        let meow =
            make_static!(Bmp::from_slice(include_bytes!("../assets/kerfur_meow.bmp")).unwrap());

        Self {
            display,
            neutral: Image::new(neutral, Point::zero()),
            blink: Image::new(blink, Point::zero()),
            meow: Image::new(meow, Point::zero()),
            rng,
        }
    }

    /// Display a [`KerfurFace`] on the display.
    fn display(&mut self, face: KerfurFace) -> Result<(), DisplayManagerError> {
        match face {
            KerfurFace::Neutral => self.neutral.draw(&mut self.display).unwrap(),
            KerfurFace::Blink => self.blink.draw(&mut self.display).unwrap(),
            KerfurFace::Meow => self.meow.draw(&mut self.display).unwrap(),
        }
        match self.display.flush() {
            Err(sh1106::Error::Comm(err)) => Err(DisplayManagerError::I2C(err)),
            Ok(()) | Err(..) => Ok(()),
        }
    }

    /// Run the kerfur display loop.
    async fn execute(&mut self) -> Result<!, DisplayManagerError> {
        loop {
            // Display the neutral frame
            self.display(KerfurFace::Neutral)?;

            // Wait before showing the next frame
            {
                // Have a ~1/32nd chance to short blink
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
                // Have a ~1/16th chance to meow instead of blink
                if self.rng.random().is_multiple_of(16) {
                    // Display the meow frame
                    self.display(KerfurFace::Meow)?;
                    Timer::after_millis(500).await;
                } else {
                    // Display the blink frame
                    self.display(KerfurFace::Blink)?;
                    Timer::after_millis(100).await;
                }
            }
        }
    }
}

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
