//! TODO
#![expect(non_camel_case_types, reason = "Pin Names")]

use core::fmt::Debug;

use defmt::Format;
use embassy_sync::{
    blocking_mutex::raw::RawMutex,
    mutex::{Mutex, TryLockError},
};
use embedded_hal::digital::{Error as DigitalError, ErrorKind, ErrorType, InputPin, OutputPin};
use embedded_hal_async::i2c::{I2c, SevenBitAddress};

macro_rules! create_pin {
    ($ty:ident, $mask:expr) => {
        /// A pin controlled by the [`TCA9554`](crate::Tca9554) IO expander.
        pub struct $ty<'i2c, M: RawMutex, I2C: I2c> {
            pub(crate) i2c: &'i2c Mutex<M, I2C>,
            pub(crate) addr: SevenBitAddress,
        }

        impl<M: RawMutex, I2C: I2c> $ty<'_, M, I2C> {
            /// Reborrow the pin, creating an owned instance with a shorter lifetime.
            #[inline]
            #[must_use]
            pub fn reborrow(&mut self) -> $ty<'_, M, I2C> { $ty { i2c: self.i2c, addr: self.addr } }

            /// Placeholder method.
            #[inline]
            pub fn placeholder<'a>(&'a mut self) -> impl Future<Output = ()> + 'a {
                crate::action::placeholder::<M, I2C, Self>(self.i2c, self.addr)
            }
        }

        impl<M: RawMutex, I2C: I2c> TCAPin for $ty<'_, M, I2C> {
            const MASK: u8 = $mask;
        }

        impl<M: RawMutex, I2C: I2c> ErrorType for $ty<'_, M, I2C> {
            type Error = Tca9554PinError<I2C::Error>;
        }
        impl<M: RawMutex, I2C: I2c> InputPin for $ty<'_, M, I2C> {
            fn is_high(&mut self) -> Result<bool, Self::Error> {
                let _i2c = self.i2c.try_lock().map_err(Tca9554PinError::Lock)?;
                todo!();
            }

            fn is_low(&mut self) -> Result<bool, Self::Error> {
                let _i2c = self.i2c.try_lock().map_err(Tca9554PinError::Lock)?;
                todo!();
            }
        }
        impl<M: RawMutex, I2C: I2c> OutputPin for $ty<'_, M, I2C> {
            fn set_low(&mut self) -> Result<(), Self::Error> {
                let _i2c = self.i2c.try_lock().map_err(Tca9554PinError::Lock)?;
                todo!();
            }

            fn set_high(&mut self) -> Result<(), Self::Error> {
                let _i2c = self.i2c.try_lock().map_err(Tca9554PinError::Lock)?;
                todo!();
            }
        }
    };
}

/// A pin controlled by a [`TCA9554`](crate::Tca9554) IO expander.
pub trait TCAPin: sealed::Sealed {
    /// The bitmask for this pin.
    const MASK: u8;
}

create_pin!(TCA_P0, 0b0000_0001);
create_pin!(TCA_P1, 0b0000_0010);
create_pin!(TCA_P2, 0b0000_0100);
create_pin!(TCA_P3, 0b0000_1000);
create_pin!(TCA_P4, 0b0001_0000);
create_pin!(TCA_P5, 0b0010_0000);
create_pin!(TCA_P6, 0b0100_0000);
create_pin!(TCA_P7, 0b1000_0000);

mod sealed {
    pub trait Sealed {}
    impl<M: super::RawMutex, I2C: super::I2c> Sealed for super::TCA_P0<'_, M, I2C> {}
    impl<M: super::RawMutex, I2C: super::I2c> Sealed for super::TCA_P1<'_, M, I2C> {}
    impl<M: super::RawMutex, I2C: super::I2c> Sealed for super::TCA_P2<'_, M, I2C> {}
    impl<M: super::RawMutex, I2C: super::I2c> Sealed for super::TCA_P3<'_, M, I2C> {}
    impl<M: super::RawMutex, I2C: super::I2c> Sealed for super::TCA_P4<'_, M, I2C> {}
    impl<M: super::RawMutex, I2C: super::I2c> Sealed for super::TCA_P5<'_, M, I2C> {}
    impl<M: super::RawMutex, I2C: super::I2c> Sealed for super::TCA_P6<'_, M, I2C> {}
    impl<M: super::RawMutex, I2C: super::I2c> Sealed for super::TCA_P7<'_, M, I2C> {}
}

// -------------------------------------------------------------------------------------------------

/// An error that can occur when using a TCA9554 pin.
#[derive(Clone, Copy, PartialEq, Eq, Format)]
pub enum Tca9554PinError<I2C> {
    /// An error occurred while trying to lock the I2C bus.
    Lock(TryLockError),
    /// An I2C communication error occurred.
    I2C(I2C),
}

impl<I2C> DigitalError for Tca9554PinError<I2C> {
    fn kind(&self) -> ErrorKind { ErrorKind::Other }
}

impl<I2C> Debug for Tca9554PinError<I2C> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Tca9554PinError::Lock(err) => write!(f, "Lock error: {err:?}"),
            Tca9554PinError::I2C(_) => f.write_str("I2C error (details not available)"),
        }
    }
}
