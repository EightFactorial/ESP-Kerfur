//! TODO
#![no_std]

use embassy_sync::{blocking_mutex::raw::RawMutex, mutex::Mutex};
use embedded_hal_async::i2c::{I2c, SevenBitAddress};

use crate::pin::TCAPin;

mod action;
pub mod pin;

/// A driver for the TCA9554 io expander.
pub struct Tca9554<'i2c, M: RawMutex, I2C: I2c> {
    i2c: &'i2c Mutex<M, I2C>,
    addr: SevenBitAddress,

    /// [`TCA_P0`](pin::TCA_P0) pin
    pub p0: pin::TCA_P0<'i2c, M, I2C>,
    /// [`TCA_P1`](pin::TCA_P1) pin
    pub p1: pin::TCA_P1<'i2c, M, I2C>,
    /// [`TCA_P2`](pin::TCA_P2) pin
    pub p2: pin::TCA_P2<'i2c, M, I2C>,
    /// [`TCA_P3`](pin::TCA_P3) pin
    pub p3: pin::TCA_P3<'i2c, M, I2C>,
    /// [`TCA_P4`](pin::TCA_P4) pin
    pub p4: pin::TCA_P4<'i2c, M, I2C>,
    /// [`TCA_P5`](pin::TCA_P5) pin
    pub p5: pin::TCA_P5<'i2c, M, I2C>,
    /// [`TCA_P6`](pin::TCA_P6) pin
    pub p6: pin::TCA_P6<'i2c, M, I2C>,
    /// [`TCA_P7`](pin::TCA_P7) pin
    pub p7: pin::TCA_P7<'i2c, M, I2C>,
}

impl<'i2c, M: RawMutex, I2C: I2c> Tca9554<'i2c, M, I2C> {
    /// Create a new [`Tca9554`] io expander driver instance
    #[inline]
    #[must_use]
    pub const fn new(i2c: &'i2c Mutex<M, I2C>, addr: SevenBitAddress) -> Self {
        Self {
            i2c,
            addr,
            p0: pin::TCA_P0 { i2c, addr },
            p1: pin::TCA_P1 { i2c, addr },
            p2: pin::TCA_P2 { i2c, addr },
            p3: pin::TCA_P3 { i2c, addr },
            p4: pin::TCA_P4 { i2c, addr },
            p5: pin::TCA_P5 { i2c, addr },
            p6: pin::TCA_P6 { i2c, addr },
            p7: pin::TCA_P7 { i2c, addr },
        }
    }

    /// Placeholder method.
    #[inline]
    pub fn placeholder<'a, PIN: TCAPin + 'a>(&'a mut self) -> impl Future<Output = ()> + 'a {
        action::placeholder::<M, I2C, PIN>(&self.i2c, self.addr)
    }
}
