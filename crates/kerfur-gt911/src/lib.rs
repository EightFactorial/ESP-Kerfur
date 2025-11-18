//! TODO
#![no_std]

use defmt::Format;
use embedded_hal_async::i2c::I2c;

/// A driver for the GT911 touch controller.
pub struct Gt911<I2C: I2c> {
    i2c: I2C,
    addr: u8,
}

impl<I2C: I2c> Gt911<I2C> {
    /// The register address to enter command mode.
    pub const COMMAND_REG: u16 = 0x8040;
    /// The register address for the product id.
    pub const PRODUCT_ID_REG: u16 = 0x8140;
    /// The register address for the first touch point.
    pub const TOUCH_POINT1_REG: u16 = 0x814F;
    /// The register address for the touch status.
    pub const TOUCH_STATUS_REG: u16 = 0x814E;

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

    /// Initialize the GT911 touch controller.
    pub async fn init(&mut self) -> Result<(), Gt911Error<I2C::Error>> {
        let mut product_id = Gt911Command::ProductId([0u8; 4]);
        self.execute(core::slice::from_mut(&mut product_id)).await?;
        let Gt911Command::ProductId(product_id) = product_id else { unreachable!() };
        if matches!(product_id, [b'9', b'1', b'1', b'\0']) {
            Ok(())
        } else {
            Err(Gt911Error::ProductIdMismatch(product_id))
        }
    }

    /// Get the current number of touch points.
    pub async fn touch_count(&mut self) -> Result<u8, Gt911Error<I2C::Error>> {
        let mut touch_point_count = Gt911Command::TouchPointCount(0);
        self.execute(core::slice::from_mut(&mut touch_point_count)).await?;
        let Gt911Command::TouchPointCount(count) = touch_point_count else { unreachable!() };
        Ok(count)
    }

    /// Query the status of a specific touch point.
    ///
    /// # Warning
    ///
    /// This does not check if the point ID is valid!
    ///
    /// Call [`Gt911::touch_count`] first to get the number of touch points!
    pub async fn touch_query(
        &mut self,
        point_id: u8,
    ) -> Result<TouchStatus, Gt911Error<I2C::Error>> {
        let mut command = Gt911Command::TouchPointStatus(TouchStatus::new(point_id));
        self.execute(core::slice::from_mut(&mut command)).await?;
        let Gt911Command::TouchPointStatus(status) = command else { unreachable!() };
        Ok(status)
    }

    /// Execute a series of commands on the touch controller.
    pub async fn execute(
        &mut self,
        commands: &mut [Gt911Command],
    ) -> Result<(), Gt911Error<I2C::Error>> {
        // Enter command execution mode
        self.write_register(Self::COMMAND_REG, 0).await?;

        // Execute each command
        for command in commands {
            match command {
                Gt911Command::ProductId(buf) => {
                    // Read the product ID register
                    self.read_register(Self::PRODUCT_ID_REG, buf.as_mut_slice()).await?;
                }
                Gt911Command::TouchPointCount(buf) => {
                    // Read the touch status register
                    self.read_register(Self::TOUCH_STATUS_REG, core::slice::from_mut(buf)).await?;

                    if (*buf & 0x80) > 0 {
                        // Return the number of touch points
                        *buf = *buf & 0x0F;
                    } else {
                        // Return `NotReady` if not ready
                        return Err(Gt911Error::NotReady);
                    }
                }
                Gt911Command::TouchPointStatus(status) => {
                    // Calculate the register for the touch point
                    let register = Self::TOUCH_POINT1_REG
                        + (size_of::<TouchStatus>() as u16 * u16::from(status.point_id));
                    let mut buf = [0u8; core::mem::size_of::<TouchStatus>()];
                    // Read the touch point status
                    self.read_register(register, &mut buf).await?;
                    *status = TouchStatus::from_bytes(buf);
                }
            }
        }

        // Clear the touch status register
        self.write_register(Self::TOUCH_STATUS_REG, 0).await
    }

    /// Read a register from the touch controller.
    async fn read_register(
        &mut self,
        register: u16,
        buffer: &mut [u8],
    ) -> Result<(), Gt911Error<I2C::Error>> {
        self.i2c
            .write_read(self.addr, &register.to_be_bytes(), buffer)
            .await
            .map_err(Gt911Error::I2C)
    }

    /// Write to a register on the touch controller.
    async fn write_register(
        &mut self,
        register: u16,
        data: u8,
    ) -> Result<(), Gt911Error<I2C::Error>> {
        let register = register.to_be_bytes();
        let buffer = [register[0], register[1], data];
        self.i2c.write(self.addr, &buffer).await.map_err(Gt911Error::I2C)
    }

    /// Release the inner I2C device.
    #[inline]
    #[must_use]
    pub fn release(self) -> I2C { self.i2c }
}

/// Errors that can occur when interacting with the touch controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Format)]
pub enum Gt911Error<I2C> {
    /// The device is not ready.
    NotReady,
    /// Invalid touch point id.
    InvalidTouchPoint(u8),
    /// The product id does not match.
    ProductIdMismatch([u8; 4]),
    /// I2C communication error.
    I2C(I2C),
}

// -------------------------------------------------------------------------------------------------

/// A command for the touch controller.
///
/// Contains the buffer that will be filled with the result.
#[derive(Debug, PartialEq, Eq, Format)]
pub enum Gt911Command {
    /// Read the Product ID register.
    ProductId([u8; 4]),
    /// Read the number of touch points.
    TouchPointCount(u8),
    /// Read the status of a touch point.
    TouchPointStatus(TouchStatus),
}

/// The status of a touch point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Format)]
pub struct TouchStatus {
    /// The touch point ID.
    pub point_id: u8,
    /// The X coordinate
    pub x: u16,
    /// The Y coordinate
    pub y: u16,
    /// The area of the touch.
    pub area: u16,
    /// An additional padding byte.
    pub padding: u8,
}

impl TouchStatus {
    /// Create a new [`TouchStatus`] to query a specific touch point.
    #[must_use]
    pub const fn new(point_id: u8) -> Self {
        TouchStatus { point_id, area: 0, x: 0, y: 0, padding: 0 }
    }

    /// Convert a byte array into a [`TouchStatus`].
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 8]) -> Self {
        Self {
            point_id: bytes[0],
            x: u16::from_le_bytes([bytes[1], bytes[2]]),
            y: u16::from_le_bytes([bytes[3], bytes[4]]),
            area: u16::from_le_bytes([bytes[5], bytes[6]]),
            padding: bytes[7],
        }
    }
}
