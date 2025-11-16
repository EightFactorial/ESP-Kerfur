//! TODO
#![expect(non_camel_case_types, reason = "Pin Names")]

use core::convert::Infallible;

use embassy_sync::{blocking_mutex::raw::RawMutex, mutex::Mutex};
use embedded_hal::digital::{ErrorType, InputPin, OutputPin};
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
                crate::action::placeholder::<_, _, Self>(self.i2c, self.addr)
            }
        }

        impl<M: RawMutex, I2C: I2c> TCAPin for $ty<'_, M, I2C> {
            const MASK: u8 = $mask;
        }

        impl<M: RawMutex, I2C: I2c> ErrorType for $ty<'_, M, I2C> {
            type Error = Infallible; // TODO: Create proper error type
        }
        impl<M: RawMutex, I2C: I2c> InputPin for $ty<'_, M, I2C> {
            fn is_high(&mut self) -> Result<bool, Self::Error> { todo!() }

            fn is_low(&mut self) -> Result<bool, Self::Error> { todo!() }
        }
        impl<M: RawMutex, I2C: I2c> OutputPin for $ty<'_, M, I2C> {
            fn set_low(&mut self) -> Result<(), Self::Error> { todo!() }

            fn set_high(&mut self) -> Result<(), Self::Error> { todo!() }
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
