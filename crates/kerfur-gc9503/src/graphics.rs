use embedded_graphics_core::{prelude::*, primitives::Rectangle};
use embedded_hal::digital::OutputPin;
use embedded_hal_async::spi::SpiDevice;

use crate::{Gc9503, color::Gc9503Color};

impl<CLR: Gc9503Color, SPI: SpiDevice, CHNL: OutputPin> OriginDimensions
    for Gc9503<CLR, SPI, CHNL>
{
    fn size(&self) -> Size { Size::new_equal(480) }
}

impl<CLR: Gc9503Color, SPI: SpiDevice, CHNL: OutputPin> DrawTarget for Gc9503<CLR, SPI, CHNL> {
    type Color = CLR;
    type Error = ();

    fn draw_iter<I>(&mut self, _pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        todo!()
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        self.draw_iter(area.points().zip(colors).map(|(pos, color)| Pixel(pos, color)))
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        self.fill_contiguous(area, core::iter::repeat(color))
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.fill_solid(&self.bounding_box(), color)
    }
}
