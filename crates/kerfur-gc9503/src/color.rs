//! TODO

/// A trait for color types supported by the GC9503 display controller.
pub trait Gc9503Color: sealed::Sealed {
    /// Whether the color format is RGB (false) or BGR (true).
    const COLOR_RGB: bool;
    /// The color format value used during initialization.
    const COLOR_VALUE: u8;
}

impl Gc9503Color for Rgb565 {
    const COLOR_RGB: bool = false;
    const COLOR_VALUE: u8 = 0x50;
}
impl Gc9503Color for Rgb666 {
    const COLOR_RGB: bool = false;
    const COLOR_VALUE: u8 = 0x60;
}
impl Gc9503Color for Rgb888 {
    const COLOR_RGB: bool = false;
    const COLOR_VALUE: u8 = 0x70;
}

impl Gc9503Color for Bgr565 {
    const COLOR_RGB: bool = true;
    const COLOR_VALUE: u8 = 0x50;
}
impl Gc9503Color for Bgr666 {
    const COLOR_RGB: bool = true;
    const COLOR_VALUE: u8 = 0x60;
}
impl Gc9503Color for Bgr888 {
    const COLOR_RGB: bool = true;
    const COLOR_VALUE: u8 = 0x70;
}

// -------------------------------------------------------------------------------------------------

// Re-export color types from `embedded-graphics`
#[cfg(feature = "embedded-graphics")]
pub use embedded_graphics_core::pixelcolor::{Bgr565, Bgr666, Bgr888, Rgb565, Rgb666, Rgb888};

// Define the supported color types
#[cfg(not(feature = "embedded-graphics"))]
pub use crate::crate_def::{Bgr565, Bgr666, Bgr888, Rgb565, Rgb666, Rgb888};
#[cfg(not(feature = "embedded-graphics"))]
mod crate_def {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Rgb565;
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Rgb666;
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Rgb888;
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Bgr565;
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Bgr666;
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Bgr888;
}

mod sealed {
    #[cfg(feature = "embedded-graphics")]
    pub trait Sealed: embedded_graphics_core::pixelcolor::PixelColor {}
    #[cfg(not(feature = "embedded-graphics"))]
    pub trait Sealed {}
    impl Sealed for super::Rgb565 {}
    impl Sealed for super::Rgb666 {}
    impl Sealed for super::Rgb888 {}
    impl Sealed for super::Bgr565 {}
    impl Sealed for super::Bgr666 {}
    impl Sealed for super::Bgr888 {}
}
