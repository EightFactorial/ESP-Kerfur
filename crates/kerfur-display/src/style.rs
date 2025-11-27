//! TODO

use embedded_graphics::{
    pixelcolor::{BinaryColor, Rgb565, Rgb666, Rgb888},
    prelude::*,
    primitives::PrimitiveStyle,
};

/// A style for drawing Kerfur
pub struct KerfurStyle<C: PixelColor> {
    /// Style for the left eye inside
    pub left_eye_inner: PrimitiveStyle<C>,
    /// Style for the left eye outside
    pub left_eye_outer: PrimitiveStyle<C>,
    /// Style for the left eye line
    pub left_eye_line: PrimitiveStyle<C>,
    /// Style for the left eyebrow
    pub left_eyebrow: PrimitiveStyle<C>,

    /// Style for the right eye inside
    pub right_eye_inner: PrimitiveStyle<C>,
    /// Style for the right eye outside
    pub right_eye_outer: PrimitiveStyle<C>,
    /// Style for the right eye line
    pub right_eye_line: PrimitiveStyle<C>,
    /// Style for the right eyebrow
    pub right_eyebrow: PrimitiveStyle<C>,

    /// Style for the nose
    pub nose: PrimitiveStyle<C>,
    /// Style for the mouth
    pub mouth: PrimitiveStyle<C>,
    /// Style for the lower mouth
    pub mouth_bottom: PrimitiveStyle<C>,

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
            left_eye_inner: binary_style!(@style fill),
            left_eye_outer: binary_style!(@style stroke),
            left_eye_line: binary_style!(@style stroke, 16),
            left_eyebrow: binary_style!(@style fill, stroke, 16),
            right_eye_inner: binary_style!(@style fill),
            right_eye_outer: binary_style!(@style stroke),
            right_eye_line: binary_style!(@style stroke, 16),
            right_eyebrow: binary_style!(@style fill, stroke, 16),
            nose: binary_style!(@style stroke),
            mouth: binary_style!(@style fill, stroke, 8),
            mouth_bottom: binary_style!(@style fill, stroke, 8),
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
    (@style $stroke:expr, $width:expr) => {{
        let mut style = PrimitiveStyle::new();
        style.stroke_color = Some($stroke);
        style.stroke_width = $width;
        style
    }};
    (@style $fill:expr) => {{
        let mut style = PrimitiveStyle::new();
        style.fill_color = Some($fill);
        style
    }};
}

/// A style that displays mimics a blue Kerfur.
pub static BLUE_565: KerfurStyle<Rgb565> = binary_style!(Rgb565::CSS_BLACK, Rgb565::CSS_CYAN);
/// A style that displays mimics a pink Kerfur.
pub static PINK_565: KerfurStyle<Rgb565> = binary_style!(Rgb565::CSS_BLACK, Rgb565::CSS_HOT_PINK);
/// A style that displays mimics a red Kerfur (has a green display).
pub static RED_565: KerfurStyle<Rgb565> = binary_style!(Rgb565::CSS_BLACK, Rgb565::CSS_GREEN);
/// A style that displays a white Kerfur.
pub static WHITE_565: KerfurStyle<Rgb565> = binary_style!(Rgb565::CSS_BLACK, Rgb565::CSS_WHITE);

/// A style that displays mimics a blue Kerfur.
pub static BLUE_666: KerfurStyle<Rgb666> = binary_style!(Rgb666::CSS_BLACK, Rgb666::CSS_CYAN);
/// A style that displays mimics a pink Kerfur.
pub static PINK_666: KerfurStyle<Rgb666> = binary_style!(Rgb666::CSS_BLACK, Rgb666::CSS_HOT_PINK);
/// A style that displays mimics a red Kerfur (has a green display).
pub static RED_666: KerfurStyle<Rgb666> = binary_style!(Rgb666::CSS_BLACK, Rgb666::CSS_GREEN);
/// A style that displays a white Kerfur.
pub static WHITE_666: KerfurStyle<Rgb666> = binary_style!(Rgb666::CSS_BLACK, Rgb666::CSS_WHITE);

/// A style that displays mimics a blue Kerfur.
pub static BLUE_888: KerfurStyle<Rgb888> = binary_style!(Rgb888::CSS_BLACK, Rgb888::CSS_CYAN);
/// A style that displays mimics a pink Kerfur.
pub static PINK_888: KerfurStyle<Rgb888> = binary_style!(Rgb888::CSS_BLACK, Rgb888::CSS_HOT_PINK);
/// A style that displays mimics a red Kerfur (has a green display).
pub static RED_888: KerfurStyle<Rgb888> = binary_style!(Rgb888::CSS_BLACK, Rgb888::CSS_GREEN);
/// A style that displays a white Kerfur.
pub static WHITE_888: KerfurStyle<Rgb888> = binary_style!(Rgb888::CSS_BLACK, Rgb888::CSS_WHITE);

/// A style that displays a white Kerfur (used by mono-color displays).
pub static BINARY_ON: KerfurStyle<BinaryColor> = binary_style!(BinaryColor::Off, BinaryColor::On);
/// A style that displays a black Kerfur (used by mono-color displays).
pub static BINARY_OFF: KerfurStyle<BinaryColor> = binary_style!(BinaryColor::On, BinaryColor::Off);
