//! TODO
#![no_std]

use core::convert::Infallible;

use embedded_hal::{
    digital::OutputPin,
    spi::{ErrorType, Mode},
};
use embedded_hal_async::{delay::DelayNs, spi::SpiBus};

pub struct BitBangSpi<MOSI: OutputPin, SCK: OutputPin, DELAY: DelayNs> {
    mode: Mode,
    mosi: MOSI,
    sck: SCK,
    delay: DELAY,
}

impl<MOSI: OutputPin, SCK: OutputPin, DELAY: DelayNs> BitBangSpi<MOSI, SCK, DELAY> {
    /// Create a new [`BitBangSpi`].
    #[inline]
    #[must_use]
    pub const fn new(mode: Mode, mosi: MOSI, sck: SCK, delay: DELAY) -> Self {
        Self { mode, mosi, sck, delay }
    }

    /// Release the owned pins and delay provider.
    #[inline]
    #[must_use]
    pub fn release(self) -> (MOSI, SCK, DELAY) { (self.mosi, self.sck, self.delay) }
}

// -------------------------------------------------------------------------------------------------

impl<MOSI: OutputPin, SCK: OutputPin, DELAY: DelayNs> ErrorType for BitBangSpi<MOSI, SCK, DELAY> {
    type Error = Infallible; // TODO: Create proper error type
}

impl<MOSI: OutputPin, SCK: OutputPin, DELAY: DelayNs> SpiBus for BitBangSpi<MOSI, SCK, DELAY> {
    async fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> { todo!() }

    async fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> { todo!() }

    async fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        todo!()
    }

    async fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> { todo!() }

    async fn flush(&mut self) -> Result<(), Self::Error> { todo!() }
}
