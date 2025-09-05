//! TODO

use core::ops::{Deref, DerefMut};

use esp_hal::{
    Blocking, DriverMode,
    gpio::interconnect::PeripheralOutput,
    i2c::master::{Config, Error as I2cError, I2c, Instance as I2cInstance},
    time::Rate,
};
use sh1106::{
    Error as DisplayError, mode::displaymode::DisplayModeTrait, prelude::*,
    properties::DisplayProperties,
};

mod audio;
pub use audio::KerfAudio;

mod emote;
pub use emote::KerfEmote;

mod touch;
pub use touch::KerfTouch;

/// Kerfur's display that is drawn to.
pub struct KerfDisplay {
    /// The underlying display driver.
    display: GraphicsMode<I2cInterface<I2cWrapper<'static, Blocking>>>,
}

impl KerfDisplay {
    /// Create a new [`KerfDisplay`].
    #[must_use]
    pub fn new<
        T: I2cInstance + 'static,
        U: PeripheralOutput<'static>,
        V: PeripheralOutput<'static>,
    >(
        i2c: T,
        sda: U,
        scl: V,
    ) -> Self {
        let config = Config::default().with_frequency(Rate::from_khz(400));
        let iface =
            I2c::new(i2c, config).expect("Failed to configure I2C").with_sda(sda).with_scl(scl);
        Self {
            display: GraphicsMode::new(DisplayProperties::new(
                I2cInterface::new(I2cWrapper(iface), 0x3C),
                DisplaySize::Display128x64,
                DisplayRotation::Rotate180,
            )),
        }
    }

    /// Initialize the display.
    pub fn init(&mut self) -> Result<(), I2cError> {
        self.display.init().map_err(|err| match err {
            DisplayError::Comm(err) => err,
            DisplayError::Pin(()) => unreachable!(),
        })
    }
}

impl Deref for KerfDisplay {
    type Target = GraphicsMode<I2cInterface<I2cWrapper<'static, Blocking>>>;

    fn deref(&self) -> &Self::Target { &self.display }
}
impl DerefMut for KerfDisplay {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.display }
}

// -------------------------------------------------------------------------------------------------

/// A wrapper for compatibility between [`esp_hal`]'s
/// i2c driver and legacy [`embedded_hal`]'s i2c traits.
///
/// # Note
///
/// This will no longer be necessary once [`sh1106`]
/// updates to depend on [`embedded_hal`] `v1.0.0`.
#[repr(transparent)]
pub struct I2cWrapper<'d, Dm: DriverMode>(I2c<'d, Dm>);

impl<Dm: DriverMode> embedded_hal::blocking::i2c::Write for I2cWrapper<'_, Dm> {
    type Error = I2cError;

    #[inline]
    fn write(&mut self, address: u8, buffer: &[u8]) -> Result<(), Self::Error> {
        self.0.write::<u8>(address, buffer)
    }
}
