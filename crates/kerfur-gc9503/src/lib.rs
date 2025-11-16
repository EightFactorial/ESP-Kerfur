//! TODO
#![no_std]

use core::marker::PhantomData;

use display_interface_spi::SPIInterface;
use embedded_hal::digital::OutputPin;
use embedded_hal_async::spi::SpiDevice;

pub mod color;
use color::Gc9503Color;

#[cfg(feature = "embedded-graphics")]
mod graphics;

/// A driver for the GC9503 display controller.
pub struct Gc9503<CLR: Gc9503Color, SPI: SpiDevice, DC: OutputPin, CHNL: OutputPin> {
    interface: SPIInterface<SPI, DC>,
    channels: Gc9503Channels<CHNL>,
    _color: PhantomData<CLR>,
}

impl<CLR: Gc9503Color, SPI: SpiDevice, DC: OutputPin, CHNL: OutputPin> Gc9503<CLR, SPI, DC, CHNL> {
    /// Create a new [`Gc9503`] driver instance.
    #[inline]
    #[must_use]
    pub fn new(spi: SPI, dc: DC, channels: Gc9503Channels<CHNL>) -> Self {
        Self { interface: SPIInterface::new(spi, dc), channels, _color: PhantomData }
    }

    /// Get a reference to the inner [`SPIInterface`].
    #[inline]
    #[must_use]
    pub const fn interface(&self) -> &SPIInterface<SPI, DC> { &self.interface }

    /// Get a mutable reference to the inner [`SPIInterface`].
    #[inline]
    #[must_use]
    pub const fn interface_mut(&mut self) -> &mut SPIInterface<SPI, DC> { &mut self.interface }

    /// Get a reference to the display channels.
    #[inline]
    #[must_use]
    pub const fn channels(&self) -> &Gc9503Channels<CHNL> { &self.channels }

    /// Get a mutable reference to the display channels.
    #[inline]
    #[must_use]
    pub const fn channels_mut(&mut self) -> &mut Gc9503Channels<CHNL> { &mut self.channels }

    /// Release the inner SPI device, DC pin, and display pins.
    #[inline]
    #[must_use]
    pub fn release(self) -> (SPI, DC, Gc9503Channels<CHNL>) {
        let (spi, dc) = self.interface.release();
        (spi, dc, self.channels)
    }
}

/// The display pins for the [`Gc9503`] driver.
pub struct Gc9503Channels<P: OutputPin> {
    /// The display enable pin.
    pub enable: P,
    /// The pixel clock pin.
    pub p_clck: P,
    /// The vertical sync pin.
    pub v_sync: P,
    /// The horizontal sync pin.
    pub h_sync: P,
    /// Unknown, unused pin.
    pub display: Option<P>,
    /// The display data pins.
    pub display_data: [P; 16],
}

// -------------------------------------------------------------------------------------------------
