//! TODO
#![no_std]

extern crate alloc;

use alloc::{vec, vec::Vec};
use core::slice::from_mut;

use defmt::Format;
use embedded_hal_async::i2c::I2c;

/// A driver for the GT911 touch controller.
pub struct Gt911<I2C: I2c> {
    i2c: I2C,
    addr: u8,
}

#[rustfmt::skip]
#[allow(dead_code, reason = "Register definitions")]
impl<I2C: I2c> Gt911<I2C> {
    // Sourced from esp-bsp gt911 driver

    const ESP_LCD_TOUCH_GT911_ENTER_SLEEP: u16 = 0x8040;
    const ESP_LCD_TOUCH_GT911_CONFIG_REG: u16 = 0x8047;
    const ESP_LCD_TOUCH_GT911_READ_KEY_REG: u16 = 0x8093;
    const ESP_LCD_TOUCH_GT911_PRODUCT_ID_REG: u16 = 0x8140;
    const ESP_LCD_TOUCH_GT911_READ_XY_REG: u16 = 0x814E;
    const ESP_LCD_TOUCH_GT911_READ_ALL_REG: u16 = 0x814F;
}

impl<I2C: I2c> Gt911<I2C> {
    /// Create a new [`Gt911`] driver.
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
        let mut product_id = Gt911Command::ProductId([0u8; 4], 0);
        self.execute(from_mut(&mut product_id)).await?;
        match product_id {
            Gt911Command::ProductId(id, _) if id == [b'9', b'1', b'1', b'\0'] => Ok(()),
            Gt911Command::ProductId(id, cfg) => Err(Gt911Error::ProductIdMismatch(id, cfg)),
            _ => unreachable!(),
        }
    }

    /// Get the current number of touch points.
    pub async fn touch_count(&mut self) -> Result<u8, Gt911Error<I2C::Error>> {
        let mut touch_point_count = Gt911Command::TouchPointCount(0);
        self.execute(from_mut(&mut touch_point_count)).await?;
        if let Gt911Command::TouchPointCount(count) = touch_point_count {
            Ok(count)
        } else {
            unreachable!()
        }
    }

    /// Query the status of a specific touch point.
    pub async fn touch_query(
        &mut self,
        point_id: u8,
    ) -> Result<TouchStatus, Gt911Error<I2C::Error>> {
        let mut command = Gt911Command::TouchPointQuery(TouchStatus::new(point_id));
        self.execute(from_mut(&mut command)).await?;
        if let Gt911Command::TouchPointQuery(status) = command {
            Ok(status)
        } else {
            unreachable!()
        }
    }

    /// Get the status of all touch points.
    pub async fn touch_list(
        &mut self,
        list: Vec<TouchStatus>,
    ) -> Result<Vec<TouchStatus>, (Vec<TouchStatus>, Gt911Error<I2C::Error>)> {
        let mut command = Gt911Command::TouchPointList(list);
        match self.execute(from_mut(&mut command)).await {
            Ok(()) => {
                if let Gt911Command::TouchPointList(statuses) = command {
                    Ok(statuses)
                } else {
                    unreachable!()
                }
            }
            Err(err) => {
                if let Gt911Command::TouchPointList(statuses) = command {
                    Err((statuses, err))
                } else {
                    unreachable!()
                }
            }
        }
    }

    /// Execute a series of commands on the touch controller.
    #[inline]
    pub async fn execute(
        &mut self,
        commands: &mut [Gt911Command],
    ) -> Result<(), Gt911Error<I2C::Error>> {
        // SAFETY: No cached point count provided
        unsafe { self.execute_with_count(commands, None).await }
    }

    /// Execute a series of commands on the touch controller.
    ///
    /// Providing a cached number of points prevents redundant reads.
    ///
    /// # Safety
    ///
    /// The caller must ensure that if `point_count` is provided, it is
    /// accurate.
    pub async unsafe fn execute_with_count(
        &mut self,
        commands: &mut [Gt911Command],
        mut point_count: Option<u8>,
    ) -> Result<(), Gt911Error<I2C::Error>> {
        // Enter command execution mode
        self.write_register(Self::ESP_LCD_TOUCH_GT911_ENTER_SLEEP, 0).await?;

        // Execute each command
        for command in commands {
            match command {
                Gt911Command::ProductId(id, cfg) => {
                    // Read the product ID register
                    self.read_register(Self::ESP_LCD_TOUCH_GT911_PRODUCT_ID_REG, id.as_mut_slice())
                        .await?;
                    // Read the config version register
                    self.read_register(Self::ESP_LCD_TOUCH_GT911_CONFIG_REG, from_mut(cfg)).await?;
                }
                Gt911Command::TouchPointCount(buf) => {
                    // If not stored yet, get and cache the point count
                    if point_count.is_none() {
                        point_count = Some(Self::read_count(self).await?);
                    }
                    *buf = point_count.unwrap();
                }
                Gt911Command::TouchPointQuery(status) => {
                    // If not stored yet, get and cache the point count
                    if point_count.is_none() {
                        point_count = Some(Self::read_count(self).await?);
                    }
                    let point_count = point_count.unwrap();

                    // Check if the requested point ID is valid
                    if status.point_id > point_count {
                        return Err(Gt911Error::InvalidTouchPoint(status.point_id));
                    }

                    // Create a buffer to read all touch points
                    let mut buf =
                        vec![0u8; core::mem::size_of::<TouchStatus>() * status.point_id as usize];
                    let range = status.point_id as usize * 8..(status.point_id as usize + 1) * 8;
                    // Read the touch point data
                    self.read_register(Self::ESP_LCD_TOUCH_GT911_READ_ALL_REG, &mut buf).await?;
                    *status = TouchStatus::from_bytes(buf[range].try_into().unwrap());
                }
                Gt911Command::TouchPointList(statuses) => {
                    // If not stored yet, get and cache the point count
                    if point_count.is_none() {
                        point_count = Some(Self::read_count(self).await?);
                    }
                    let point_count = point_count.unwrap();

                    // Create a buffer to read all touch points
                    let mut buf =
                        vec![0u8; core::mem::size_of::<TouchStatus>() * point_count as usize];
                    // Read the touch point data
                    self.read_register(Self::ESP_LCD_TOUCH_GT911_READ_ALL_REG, &mut buf).await?;

                    // Copy each touch status into the output vector
                    statuses.clear();
                    for i in 0..point_count {
                        let range = i as usize * 8..(i as usize + 1) * 8;
                        statuses.push(TouchStatus::from_bytes(buf[range].try_into().unwrap()));
                    }
                }
            }
        }

        // Clear all touch points
        self.write_register(Self::ESP_LCD_TOUCH_GT911_READ_XY_REG, 0).await
    }

    /// Read the current touch point count.
    async fn read_count(&mut self) -> Result<u8, Gt911Error<I2C::Error>> {
        // Read the touch status register
        let mut buf = 0u8;
        self.read_register(Self::ESP_LCD_TOUCH_GT911_READ_XY_REG, from_mut(&mut buf)).await?;

        if (buf & 0x80) > 0 {
            // Return the number of touch points
            Ok(buf & 0x0F)
        } else {
            //  Return `NotReady` if not ready
            Err(Gt911Error::NotReady)
        }
    }

    /// Read a register from the GT911.
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

    /// Write to a register on the GT911.
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

/// An error that can occur when interacting with the [`Gt911`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Format)]
pub enum Gt911Error<I2C> {
    /// The device is not ready.
    NotReady,
    /// Invalid touch point id.
    InvalidTouchPoint(u8),
    /// The product id does not match.
    ProductIdMismatch([u8; 4], u8),
    /// I2C communication error.
    I2C(I2C),
}

// -------------------------------------------------------------------------------------------------

/// A command for the touch controller.
///
/// Contains the buffer that will be filled with the result.
#[derive(Debug, PartialEq, Eq)]
pub enum Gt911Command {
    /// Read the product ID and version.
    ProductId([u8; 4], u8),
    /// Read the number of touch points.
    TouchPointCount(u8),
    /// Read the status of a touch point.
    TouchPointQuery(TouchStatus),
    /// Read the status of all touch points.
    TouchPointList(Vec<TouchStatus>),
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
