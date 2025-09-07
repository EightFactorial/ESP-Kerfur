//! [`KerfEmote`] and image display functionality.

use embassy_sync::lazy_lock::LazyLock;
use embedded_graphics::{image::Image, pixelcolor::BinaryColor, prelude::*};
use esp_hal::i2c::master::Error as I2cError;
use sh1106::{Error as DisplayError, interface::DisplayInterface, mode::GraphicsMode};
use tinybmp::Bmp;

/// An emote that can be displayed on Kerfur's screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KerfEmote {
    /// The "neutral" emote.
    Neutral,
    /// The "blink" emote.
    Blink,
    /// The "meow" emote.
    Meow,

    /// The "confused" emote.
    Confused,
}

impl KerfEmote {
    /// Draw the [`KerfEmote`] to the provided display.
    pub fn draw<T: DisplayInterface<Error = DisplayError<I2cError, ()>>>(
        self,
        display: &mut GraphicsMode<T>,
    ) -> Result<(), I2cError> {
        match self {
            KerfEmote::Neutral => {
                static NEUTRAL: LazyLock<Bmp<'static, BinaryColor>> = LazyLock::new(|| {
                    Bmp::from_slice(include_bytes!("../../assets/kerfur_neutral.bmp"))
                        .expect("Failed to parse `kerfur_neutral.bmp`, are you sure it's valid?")
                });

                Image::new(NEUTRAL.get(), Point::zero()).draw(display)
            }
            KerfEmote::Blink => {
                static BLINK: LazyLock<Bmp<'static, BinaryColor>> = LazyLock::new(|| {
                    Bmp::from_slice(include_bytes!("../../assets/kerfur_blink.bmp"))
                        .expect("Failed to parse `kerfur_blink.bmp`, are you sure it's valid?")
                });

                Image::new(BLINK.get(), Point::zero()).draw(display)
            }
            KerfEmote::Meow => {
                static MEOW: LazyLock<Bmp<'static, BinaryColor>> = LazyLock::new(|| {
                    Bmp::from_slice(include_bytes!("../../assets/kerfur_meow.bmp"))
                        .expect("Failed to parse `kerfur_meow.bmp`, are you sure it's valid?")
                });

                Image::new(MEOW.get(), Point::zero()).draw(display)
            }
            KerfEmote::Confused => {
                static CONFUSED: LazyLock<Bmp<'static, BinaryColor>> = LazyLock::new(|| {
                    Bmp::from_slice(include_bytes!("../../assets/kerfur_confused.bmp"))
                        .expect("Failed to parse `kerfur_confused.bmp`, are you sure it's valid?")
                });

                Image::new(CONFUSED.get(), Point::zero()).draw(display)
            }
        }
        .unwrap();

        // Flush the display to ensure the image is shown.
        match display.flush() {
            Ok(()) => Ok(()),
            Err(DisplayError::Comm(err)) => Err(err),
            Err(DisplayError::Pin(())) => unreachable!(),
        }
    }
}
