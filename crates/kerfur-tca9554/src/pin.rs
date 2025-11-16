#![expect(non_camel_case_types, reason = "Pin Names")]

use core::convert::Infallible;

use embassy_sync::{blocking_mutex::raw::RawMutex, mutex::Mutex};
use embedded_hal::digital::{ErrorType, InputPin, OutputPin};
use embedded_hal_async::i2c::{I2c, SevenBitAddress};

pub(crate) trait TCAPin {
    const MASK: u8;
}

macro_rules! create_pin {
    ($ty:ident, $mask:expr) => {
        pub struct $ty<'i2c, M: RawMutex, I2C: I2c> {
            pub(crate) i2c: &'i2c Mutex<M, I2C>,
            pub(crate) addr: SevenBitAddress,
        }

        impl<M: RawMutex, I2C: I2c> TCAPin for $ty<'_, M, I2C> {
            const MASK: u8 = $mask;
        }

        impl<M: RawMutex, I2C: I2c> ErrorType for $ty<'_, M, I2C> {
            type Error = Infallible;
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

create_pin!(TCA_P0, 0b0000_0001);
create_pin!(TCA_P1, 0b0000_0010);
create_pin!(TCA_P2, 0b0000_0100);
create_pin!(TCA_P3, 0b0000_1000);
create_pin!(TCA_P4, 0b0001_0000);
create_pin!(TCA_P5, 0b0010_0000);
create_pin!(TCA_P6, 0b0100_0000);
create_pin!(TCA_P7, 0b1000_0000);
