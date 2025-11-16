//! TODO
#![no_std]

use embedded_hal_async::i2c::I2c;

/// A driver for the GT911 touch controller.
pub struct Gt911<I2C: I2c> {
    i2c: I2C,
    addr: u8,
}

impl<I2C: I2c> Gt911<I2C> {
    /// Create a new [`Gt911`] touch driver instance
    #[inline]
    #[must_use]
    pub const fn new(i2c: I2C, addr: u8) -> Self { Gt911 { i2c, addr } }

    /// Get the I2C address of the device.
    #[inline]
    #[must_use]
    pub const fn address(&self) -> u8 { self.addr }

    /// Get a reference to the inner I2C device.
    #[inline]
    #[must_use]
    pub const fn i2c(&self) -> &I2C { &self.i2c }

    /// Get a mutable reference to the inner I2C device.
    #[inline]
    #[must_use]
    pub fn i2c_mut(&mut self) -> &mut I2C { &mut self.i2c }

    /// Release the inner I2C device.
    #[inline]
    #[must_use]
    pub fn release(self) -> I2C { self.i2c }
}
