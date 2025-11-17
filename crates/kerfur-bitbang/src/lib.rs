//! TODO
#![no_std]

use core::fmt::Debug;

use embedded_hal::{
    digital::{ErrorType as DigitalErrorType, InputPin, OutputPin},
    spi::{MODE_0, MODE_1, MODE_2, MODE_3},
};
use embedded_hal_async::{
    delay::DelayNs,
    spi::{Error, ErrorKind, ErrorType as SpiErrorType, Mode, SpiBus},
};

/// A bit-banged write-only SPI implementation.
pub struct BitBangSpi<SCK: OutputPin, SDIO: InputPin + OutputPin, DELAY: DelayNs> {
    sck: SCK,
    sdio: SDIO,
    delay: DELAY,
    delay_us: u32,
    mode: Mode,
}

impl<SCK: OutputPin, SDIO: InputPin + OutputPin, DELAY: DelayNs> BitBangSpi<SCK, SDIO, DELAY> {
    /// Create a new [`BitBangSpi`].
    #[inline]
    #[must_use]
    pub const fn new(sck: SCK, sdio: SDIO, delay: DELAY, delay_us: u32, mode: Mode) -> Self {
        Self { sck, sdio, delay, delay_us, mode }
    }

    /// Write a single byte to the SPI bus.
    pub async fn write_word(&mut self, byte: u8) -> Result<(), SpiError<SCK, SDIO>> {
        for bit in 0..8 {
            // Set data line according to the current bit
            let bit_value = (byte >> (7 - bit)) & 0x01;
            if bit_value == 1 {
                self.sdio.set_high().map_err(SpiError::DataError)?;
            } else {
                self.sdio.set_low().map_err(SpiError::DataError)?;
            }

            // Toggle clock line based on SPI mode
            match self.mode {
                MODE_0 => {
                    self.delay.delay_us(self.delay_us).await;
                    self.sck.set_high().map_err(SpiError::ClockError)?;
                    self.delay.delay_us(self.delay_us).await;
                    self.sck.set_low().map_err(SpiError::ClockError)?;
                }
                MODE_1 => {
                    self.sck.set_high().map_err(SpiError::ClockError)?;
                    self.delay.delay_us(self.delay_us).await;
                    self.sck.set_low().map_err(SpiError::ClockError)?;
                    self.delay.delay_us(self.delay_us).await;
                }
                MODE_2 => {
                    self.delay.delay_us(self.delay_us).await;
                    self.sck.set_low().map_err(SpiError::ClockError)?;
                    self.delay.delay_us(self.delay_us).await;
                    self.sck.set_high().map_err(SpiError::ClockError)?;
                }
                MODE_3 => {
                    self.sck.set_low().map_err(SpiError::ClockError)?;
                    self.delay.delay_us(self.delay_us).await;
                    self.sck.set_high().map_err(SpiError::ClockError)?;
                    self.delay.delay_us(self.delay_us).await;
                }
            }
        }
        Ok(())
    }

    /// Release the owned pins and delay provider.
    #[inline]
    #[must_use]
    pub fn release(self) -> (SCK, SDIO, DELAY) { (self.sck, self.sdio, self.delay) }
}

// -------------------------------------------------------------------------------------------------

impl<SCK: OutputPin, SDIO: InputPin + OutputPin, DELAY: DelayNs> SpiErrorType
    for BitBangSpi<SCK, SDIO, DELAY>
{
    type Error = SpiError<SCK, SDIO>;
}

impl<SCK: OutputPin, SDIO: InputPin + OutputPin, DELAY: DelayNs> SpiBus
    for BitBangSpi<SCK, SDIO, DELAY>
{
    #[inline]
    async fn read(&mut self, _: &mut [u8]) -> Result<(), Self::Error> { Err(SpiError::WriteOnly) }

    async fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        for &word in words {
            self.write_word(word).await?;
        }
        Ok(())
    }

    async fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        if !read.is_empty() && write.is_empty() {
            Err(SpiError::WriteOnly)
        } else {
            self.write(write).await
        }
    }

    #[inline]
    async fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.write(words).await
    }

    #[inline]
    async fn flush(&mut self) -> Result<(), Self::Error> { Ok(()) }
}

// -------------------------------------------------------------------------------------------------

/// An error type for [`BitBangSpi`].
pub enum SpiError<SCK: DigitalErrorType, SDIO: DigitalErrorType> {
    /// Attempted to read from a write-only SPI bus.
    WriteOnly,
    /// An error occurred on the clock pin.
    ClockError(SCK::Error),
    /// An error occurred on the data pin.
    DataError(SDIO::Error),
}

impl<SCK: DigitalErrorType, SDIO: DigitalErrorType> Error for SpiError<SCK, SDIO> {
    fn kind(&self) -> ErrorKind { ErrorKind::Other }
}

impl<SCK: DigitalErrorType, SDIO: DigitalErrorType> Debug for SpiError<SCK, SDIO> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SpiError::WriteOnly => f.write_str("SpiError::WriteOnly"),
            SpiError::ClockError(_) => f.write_str("SpiError::ClockError"),
            SpiError::DataError(_) => f.write_str("SpiError::DataError"),
        }
    }
}
