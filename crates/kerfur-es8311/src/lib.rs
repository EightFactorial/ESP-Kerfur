//! TODO
#![no_std]

use defmt::Format;
use embedded_hal_async::i2c::I2c;

/// A driver for the ES8311 audio codec.
pub struct Es8311<I2C: I2c> {
    i2c: I2C,
    addr: u8,
}

#[rustfmt::skip]
#[allow(dead_code, reason = "Register definitions")]
impl<I2C: I2c> Es8311<I2C> {
    // Sourced from esp-bsp es8311 driver

    const ES8311_RESET_REG00: u8 = 0x00; // reset digital,csm,clock manager etc.
    // Clock Manager Registers
    const ES8311_CLK_MANAGER_REG01: u8 = 0x01; // select clk src for mclk, enable clock for codec
    const ES8311_CLK_MANAGER_REG02: u8 = 0x02; // clk divider and clk multiplier
    const ES8311_CLK_MANAGER_REG03: u8 = 0x03; // adc fsmode and osr
    const ES8311_CLK_MANAGER_REG04: u8 = 0x04; // dac osr
    const ES8311_CLK_MANAGER_REG05: u8 = 0x05; // clk divier for adc and dac
    const ES8311_CLK_MANAGER_REG06: u8 = 0x06; // bclk inverter and divider
    const ES8311_CLK_MANAGER_REG07: u8 = 0x07; // tri-state, lrck divider
    const ES8311_CLK_MANAGER_REG08: u8 = 0x08; // lrck divider
    // Serial Digital Port Registers
    const ES8311_SDPIN_REG09: u8 =  0x09; // dac serial digital port
    const ES8311_SDPOUT_REG0A: u8 = 0x0A; // adc serial digital port
    // System Registers
    const ES8311_SYSTEM_REG0B: u8 = 0x0B; // system
    const ES8311_SYSTEM_REG0C: u8 = 0x0C; // system
    const ES8311_SYSTEM_REG0D: u8 = 0x0D; // system, power up/down
    const ES8311_SYSTEM_REG0E: u8 = 0x0E; // system, power up/down
    const ES8311_SYSTEM_REG0F: u8 = 0x0F; // system, low power
    const ES8311_SYSTEM_REG10: u8 = 0x10; // system
    const ES8311_SYSTEM_REG11: u8 = 0x11; // system
    const ES8311_SYSTEM_REG12: u8 = 0x12; // system, Enable DAC
    const ES8311_SYSTEM_REG13: u8 = 0x13; // system
    const ES8311_SYSTEM_REG14: u8 = 0x14; // system, select DMIC, select analog pga gain
    // ADC Registers
    const ES8311_ADC_REG15: u8 = 0x15; // ADC, adc ramp rate, dmic sense
    const ES8311_ADC_REG16: u8 = 0x16; // ADC
    const ES8311_ADC_REG17: u8 = 0x17; // ADC, volume
    const ES8311_ADC_REG18: u8 = 0x18; // ADC, alc enable and winsize
    const ES8311_ADC_REG19: u8 = 0x19; // ADC, alc maxlevel
    const ES8311_ADC_REG1A: u8 = 0x1A; // ADC, alc automute
    const ES8311_ADC_REG1B: u8 = 0x1B; // ADC, alc automute, adc hpf s1
    const ES8311_ADC_REG1C: u8 = 0x1C; // ADC, equalizer, hpf s2
    // DAC Registers
    const ES8311_DAC_REG31: u8 = 0x31; // DAC, mute
    const ES8311_DAC_REG32: u8 = 0x32; // DAC, volume
    const ES8311_DAC_REG33: u8 = 0x33; // DAC, offset
    const ES8311_DAC_REG34: u8 = 0x34; // DAC, drc enable, drc winsize
    const ES8311_DAC_REG35: u8 = 0x35; // DAC, drc maxlevel, minilevel
    const ES8311_DAC_REG37: u8 = 0x37; // DAC, ramprate
    // GPIO Registers
    const ES8311_GPIO_REG44: u8 = 0x44; // GPIO, dac2adc for test
    const ES8311_GP_REG45: u8 = 0x45; // GP CONTROL
    // Chip Registers
    const ES8311_CHD1_REGFD: u8 = 0xFD; // CHIP ID1
    const ES8311_CHD2_REGFE: u8 = 0xFE; // CHIP ID2
    const ES8311_CHVER_REGFF: u8 = 0xFF; // VERSION
}

impl<I2C: I2c> Es8311<I2C> {
    /// Create a new [`Es8311`] driver.
    #[inline]
    #[must_use]
    pub const fn new(i2c: I2C, addr: u8) -> Self { Es8311 { i2c, addr } }

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

    /// Read a register from the ES8311.
    #[expect(dead_code, reason = "WIP")]
    async fn read_register(
        &mut self,
        _register: u8,
        _buffer: &mut [u8],
    ) -> Result<(), Es8311Error<I2C::Error>> {
        todo!()
    }

    /// Write to a register on the ES8311.
    #[expect(dead_code, reason = "WIP")]
    async fn write_register(
        &mut self,
        _register: u8,
        _data: u8,
    ) -> Result<(), Es8311Error<I2C::Error>> {
        todo!()
    }

    /// Release the inner I2C device.
    #[inline]
    #[must_use]
    pub fn release(self) -> I2C { self.i2c }
}

/// An error that can occur when interacting with the [`Es8311`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Format)]
pub enum Es8311Error<I2C> {
    /// I2C communication error.
    I2C(I2C),
}

// -------------------------------------------------------------------------------------------------
