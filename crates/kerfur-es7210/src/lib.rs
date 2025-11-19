//! TODO
#![no_std]

use defmt::Format;
use embedded_hal_async::i2c::I2c;

/// A driver for the ES7210 ADC
pub struct Es7210<I2C: I2c> {
    i2c: I2C,
    addr: u8,
}

#[rustfmt::skip]
#[allow(dead_code, reason = "Register definitions")]
impl<I2C: I2c> Es7210<I2C> {
    // Sourced from esp-bsp es7210 driver

    const ES7210_RESET_REG00: u8 = 0x00; // Reset control
    const ES7210_CLOCK_OFF_REG01: u8 = 0x01; // Used to turn off the ADC clock
    const ES7210_MAINCLK_REG02: u8 = 0x02; // Set ADC clock frequency division
    const ES7210_MASTER_CLK_REG03: u8 = 0x03; // MCLK source $ SCLK division
    const ES7210_LRCK_DIVH_REG04: u8 = 0x04; // lrck_divh
    const ES7210_LRCK_DIVL_REG05: u8 = 0x05; // lrck_divl
    const ES7210_POWER_DOWN_REG06: u8 = 0x06; // power down
    const ES7210_OSR_REG07: u8 = 0x07;
    const ES7210_MODE_CONFIG_REG08: u8 = 0x08; // Set master/slave & channels
    const ES7210_TIME_CONTROL0_REG09: u8 = 0x09; // Set Chip initial state period
    const ES7210_TIME_CONTROL1_REG0A: u8 = 0x0A; // Set Power up state period
    // Serial Digital Port Registers
    const ES7210_SDP_INTERFACE1_REG11: u8 = 0x11; // Set sample & fmt
    const ES7210_SDP_INTERFACE2_REG12: u8 = 0x12; // Pins state
    // ADC Registers
    const ES7210_ADC_AUTOMUTE_REG13: u8 = 0x13; // Set mute
    const ES7210_ADC34_MUTERANGE_REG14: u8 = 0x14; // Set mute range
    const ES7210_ALC_SEL_REG16: u8 = 0x16; // Set ALC mode
    const ES7210_ADC1_DIRECT_DB_REG1B: u8 = 0x1B; // ADC direct dB when ALC close, ALC max gain when ALC open
    const ES7210_ADC2_DIRECT_DB_REG1C: u8 = 0x1C;
    const ES7210_ADC3_DIRECT_DB_REG1D: u8 = 0x1D;
    const ES7210_ADC4_DIRECT_DB_REG1E: u8 = 0x1E;
    const ES7210_ADC34_HPF2_REG20: u8 = 0x20; // HPF
    const ES7210_ADC34_HPF1_REG21: u8 = 0x21;
    const ES7210_ADC12_HPF2_REG22: u8 = 0x22;
    const ES7210_ADC12_HPF1_REG23: u8 = 0x23;
    // Chip Registers
    const ES7210_CHIP_ID1_REG3C: u8 = 0x3C; // Chip ID1
    const ES7210_CHIP_ID2_REG3D: u8 = 0x3D; // Chip ID2
    const ES7210_CHIP_VERSION_REG3E: u8 = 0x3E; // Chip Version
    // Analog Registers
    const ES7210_ANALOG_REG40: u8 = 0x40; // ANALOG Power
    // Microphone Registers
    const ES7210_MIC12_BIAS_REG41: u8 = 0x41;
    const ES7210_MIC34_BIAS_REG42: u8 = 0x42;
    const ES7210_MIC1_GAIN_REG43: u8 = 0x43;
    const ES7210_MIC2_GAIN_REG44: u8 = 0x44;
    const ES7210_MIC3_GAIN_REG45: u8 = 0x45;
    const ES7210_MIC4_GAIN_REG46: u8 = 0x46;
    const ES7210_MIC1_POWER_REG47: u8 = 0x47;
    const ES7210_MIC2_POWER_REG48: u8 = 0x48;
    const ES7210_MIC3_POWER_REG49: u8 = 0x49;
    const ES7210_MIC4_POWER_REG4A: u8 = 0x4A;
    const ES7210_MIC12_POWER_REG4B: u8 = 0x4B; // MICBias & ADC & PGA Power
    const ES7210_MIC34_POWER_REG4C: u8 = 0x4C;
}

impl<I2C: I2c> Es7210<I2C> {
    /// Create a new [`Es7210`] driver.
    #[inline]
    #[must_use]
    pub const fn new(i2c: I2C, addr: u8) -> Self { Es7210 { i2c, addr } }

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

    /// Initialize the ES7210.
    pub async fn init(&mut self) -> Result<(), Es7210Error<I2C::Error>> {
        self.software_reset().await?;
        self.enable_device().await
    }

    /// Perform a software reset of the ES7210.
    pub async fn software_reset(&mut self) -> Result<(), Es7210Error<I2C::Error>> {
        self.write_register(Self::ES7210_RESET_REG00, 0xFF).await?;
        self.write_register(Self::ES7210_RESET_REG00, 0x32).await
    }

    /// Enable the ES7210 device.
    pub async fn enable_device(&mut self) -> Result<(), Es7210Error<I2C::Error>> {
        self.write_register(Self::ES7210_RESET_REG00, 0x71).await?;
        self.write_register(Self::ES7210_RESET_REG00, 0x41).await
    }

    /// Read a register from the ES7210.
    #[expect(dead_code, reason = "WIP")]
    async fn read_register(
        &mut self,
        _register: u8,
        _buffer: &mut [u8],
    ) -> Result<(), Es7210Error<I2C::Error>> {
        todo!()
    }

    /// Write to a register on the ES7210.
    async fn write_register(
        &mut self,
        _register: u8,
        _data: u8,
    ) -> Result<(), Es7210Error<I2C::Error>> {
        todo!()
    }

    /// Release the inner I2C device.
    #[inline]
    #[must_use]
    pub fn release(self) -> I2C { self.i2c }
}

/// An error that can occur when interacting with the [`Es7210`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Format)]
pub enum Es7210Error<I2C> {
    /// I2C communication error.
    I2C(I2C),
}

// -------------------------------------------------------------------------------------------------
