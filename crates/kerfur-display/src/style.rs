use embedded_graphics::{
    pixelcolor::{BinaryColor, Rgb888},
    prelude::*,
    primitives::PrimitiveStyle,
};

/// A style for drawing Kerfur
pub struct KerfurStyle<C: PixelColor> {
    /// Style for the left eye
    pub left_eye: PrimitiveStyle<C>,
    /// Style for the left eyebrow
    pub left_eyebrow: PrimitiveStyle<C>,

    /// Style for the right eye
    pub right_eye: PrimitiveStyle<C>,
    /// Style for the right eyebrow
    pub right_eyebrow: PrimitiveStyle<C>,

    /// Style for the whiskers
    pub whisker: PrimitiveStyle<C>,
}

impl<C: PixelColor> KerfurStyle<C> {}

// -------------------------------------------------------------------------------------------------

macro_rules! binary_style {
    ($fill:expr, $stroke:expr) => {{
        let fill = $fill;
        let stroke = $stroke;
        KerfurStyle {
            left_eye: binary_style!(@style fill, stroke, 24),
            left_eyebrow: binary_style!(@style fill, stroke, 16),
            right_eye: binary_style!(@style fill, stroke, 24),
            right_eyebrow: binary_style!(@style fill, stroke, 16),
            whisker: binary_style!(@style fill, stroke, 10),
        }
    }};
    (@style $fill:expr, $stroke:expr, $width:expr) => {{
        let mut style = PrimitiveStyle::new();
        style.fill_color = Some($fill);
        style.stroke_color = Some($stroke);
        style.stroke_width = $width;
        style
    }};
}

impl KerfurStyle<Rgb888> {
    /// A style that displays a blue Kerfur.
    pub const BLUE: Self = binary_style!(Rgb888::CSS_BLACK, Rgb888::CSS_CYAN);
    /// A style that displays a pink Kerfur.
    pub const PINK: Self = binary_style!(Rgb888::CSS_BLACK, Rgb888::CSS_HOT_PINK);
    /// A style that displays a red Kerfur.
    pub const RED: Self = binary_style!(Rgb888::CSS_BLACK, Rgb888::CSS_RED);
    /// A style that displays a white Kerfur.
    pub const WHITE: Self = binary_style!(Rgb888::CSS_BLACK, Rgb888::CSS_WHITE);
}

impl KerfurStyle<BinaryColor> {
    /// A style used for displays that have only two color states.
    pub const BINARY: Self = binary_style!(BinaryColor::Off, BinaryColor::On);
}
