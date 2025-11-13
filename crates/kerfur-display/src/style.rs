use embedded_graphics::{
    pixelcolor::{BinaryColor, Rgb888},
    prelude::*,
};

/// A style for drawing Kerfur
pub struct KerfurStyle<C: PixelColor> {
    _phantom: core::marker::PhantomData<C>,
}

impl<C: PixelColor> KerfurStyle<C> {}

// -------------------------------------------------------------------------------------------------

macro_rules! binary_style {
    ($on:expr, $off:expr) => {
        KerfurStyle { _phantom: core::marker::PhantomData }
    };
}

impl KerfurStyle<Rgb888> {
    /// A style that displays a blue Kerfur.
    pub const BLUE: Self = binary_style!(Rgb888::CSS_CYAN, Rgb888::CSS_BLACK);
    /// A style that displays a pink Kerfur.
    pub const PINK: Self = binary_style!(Rgb888::CSS_HOT_PINK, Rgb888::CSS_BLACK);
    /// A style that displays a red Kerfur.
    pub const RED: Self = binary_style!(Rgb888::CSS_RED, Rgb888::CSS_BLACK);
    /// A style that displays a white Kerfur.
    pub const WHITE: Self = binary_style!(Rgb888::CSS_WHITE, Rgb888::CSS_BLACK);
}

impl KerfurStyle<BinaryColor> {
    /// A style used for displays that have only two color states.
    pub const BINARY: Self = binary_style!(BinaryColor::On, BinaryColor::Off);
}
