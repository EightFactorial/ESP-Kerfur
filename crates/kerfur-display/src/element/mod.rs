//! TODO
#![expect(clippy::many_single_char_names, reason = "Math")]

use core::{
    cmp::Ordering,
    f32::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_6, PI},
};

use embedded_graphics::{prelude::*, primitives::Line};

use crate::{
    KerfurStyle,
    primitive::{ConstArc, ConstSector},
};

mod eye;
pub use eye::KerfurEyeType;

mod mouth;
mod whisker;

/// A set of facial elements
#[derive(Clone, Copy, PartialEq)]
pub struct KerfurElements {
    eye: eye::EyeState,
    eyebrow: eye::EyebrowState,
    mouth: mouth::MouthState,
    whisker: whisker::WhiskerState,
}

impl Default for KerfurElements {
    fn default() -> Self { Self::new() }
}

impl KerfurElements {
    /// Create a new set of facial elements.
    ///
    /// Defaults to a neutral-looking face.
    #[must_use]
    #[expect(clippy::erasing_op, reason = "Used for consistency")]
    pub const fn new() -> Self {
        Self {
            eye: eye::EyeState {
                left: KerfurEyeType::NEUTRAL_LEFT,
                right: KerfurEyeType::NEUTRAL_RIGHT,
            },
            eyebrow: eye::EyebrowState {
                left: Line::new(
                    Point::new(480 * 42 / 100, 480 * 29 / 100),
                    Point::new(480 * 35 / 100, 480 * 29 / 100),
                ),
                right: Line::new(
                    Point::new(480 * 58 / 100, 480 * 29 / 100),
                    Point::new(480 * 65 / 100, 480 * 29 / 100),
                ),
            },
            mouth: mouth::MouthState {
                nose: ConstSector::with_center(
                    Point::new(240, 480 * 58 / 100),
                    40,
                    3. * FRAC_PI_2 - FRAC_PI_4,
                    2. * FRAC_PI_4,
                ),
                mouth_left: ConstArc::with_center(Point::new(225, 480 * 59 / 100), 30, 0., PI),
                mouth_right: ConstArc::with_center(Point::new(255, 480 * 59 / 100), 30, PI, -PI),
                mouth_bottom: ConstArc::with_center(
                    Point::new(240, 480 * 64 / 100),
                    20,
                    -FRAC_PI_6,
                    0.,
                ),
            },
            whisker: whisker::WhiskerState {
                left: Line::new(
                    Point::new(480 * 7 / 100, 480 * 63 / 100),
                    Point::new(480 * 0 / 100, 480 * 63 / 100),
                ),
                right: Line::new(
                    Point::new(480 * 93 / 100, 480 * 63 / 100),
                    Point::new(480 * 100 / 100, 480 * 63 / 100),
                ),
                offset: Point::new(0, 24),
                count: 2,
            },
        }
    }

    /// Use the given eyes in the set of facial elements.
    #[inline]
    #[must_use]
    pub const fn with_eyes(mut self, left: KerfurEyeType, right: KerfurEyeType) -> Self {
        self.eye.left = left;
        self.eye.right = right;
        self
    }

    /// Use the given eyebrows in the set of facial elements.
    #[inline]
    #[must_use]
    pub const fn with_eyebrows(mut self, left: Line, right: Line) -> Self {
        self.eyebrow.left = left;
        self.eyebrow.right = right;
        self
    }

    /// Use the given nose in the set of facial elements.
    #[inline]
    #[must_use]
    pub const fn with_nose(mut self, nose: ConstSector) -> Self {
        self.mouth.nose = nose;
        self
    }

    /// Use the given mouth sides in the set of facial elements.
    #[inline]
    #[must_use]
    pub const fn with_mouth_sides(mut self, mouth_left: ConstArc, mouth_right: ConstArc) -> Self {
        self.mouth.mouth_left = mouth_left;
        self.mouth.mouth_right = mouth_right;
        self
    }

    /// Use the given mouth bottom in the set of facial elements.
    #[inline]
    #[must_use]
    pub const fn with_mouth_bottom(mut self, mouth_bottom: ConstArc) -> Self {
        self.mouth.mouth_bottom = mouth_bottom;
        self
    }

    /// Use the given whiskers in the set of facial elements.
    #[inline]
    #[must_use]
    pub const fn with_whiskers(mut self, left: Line, right: Line) -> Self {
        self.whisker.left = left;
        self.whisker.right = right;
        self
    }

    /// Use the given whisker settings in the set of facial elements.
    #[inline]
    #[must_use]
    pub const fn with_whisker_settings(mut self, offset: Point, count: u32) -> Self {
        self.whisker.offset = offset;
        self.whisker.count = count;
        self
    }

    /// Draw this set of elements on the given display.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the elements fail to draw.
    pub(super) fn draw<D: DrawTargetExt>(
        &mut self,
        display: &mut D,
        style: &KerfurStyle<D::Color>,
    ) -> Result<(), D::Error> {
        self.eye.draw(display, style)?;
        self.eyebrow.draw(display, style)?;
        self.mouth.draw(display, style)?;
        self.whisker.draw(display, style)?;
        Ok(())
    }

    /// Interpolate this set of elements toward the target set.
    pub(super) fn interpolate(&mut self, target: &Self, tick: f32) {
        self.eye.interpolate(&target.eye, tick);
        self.eyebrow.interpolate(&target.eyebrow, tick);
        self.mouth.interpolate(&target.mouth, tick);
        self.whisker.interpolate(&target.whisker, tick);
    }
}

// -------------------------------------------------------------------------------------------------

// TODO: Fix negative?
fn interp(a_x: f32, a_y: f32, b_x: f32, b_y: f32, t: f32) -> (f32, f32) {
    let (diff_x, diff_y) = (b_x - a_x, b_y - a_y);
    let dot = (diff_x * diff_x) + (diff_y * diff_y);

    #[cfg(feature = "libm")]
    let len = libm::sqrtf(dot);
    #[cfg(not(feature = "libm"))]
    let len = dot.sqrt();

    if len <= t || len <= 1e-4 {
        (b_x, b_y)
    } else {
        (a_x + diff_x / len * t, a_y + diff_y / len * t)
    }
}

#[expect(clippy::cast_precision_loss, reason = "Positions will never be that large")]
#[expect(clippy::cast_possible_truncation, reason = "Positions will never be that large")]
fn interp_point(a: &mut Point, b: Point, t: f32) {
    let ceil_or_floor = matches!(Point::cmp(a, &b), Ordering::Less);
    let (x, y) = interp(a.x as f32, a.y as f32, b.x as f32, b.y as f32, t);
    if ceil_or_floor {
        #[cfg(feature = "libm")]
        {
            (a.x, a.y) = (libm::ceilf(x) as i32, libm::ceilf(y) as i32);
        }
        #[cfg(not(feature = "libm"))]
        {
            (a.x, a.y) = (x.ceil() as i32, y.ceil() as i32);
        }
    } else {
        #[cfg(feature = "libm")]
        {
            (a.x, a.y) = (libm::floorf(x) as i32, libm::floorf(y) as i32);
        }
        #[cfg(not(feature = "libm"))]
        {
            (a.x, a.y) = (x.floor() as i32, y.floor() as i32);
        }
    }
}

#[expect(clippy::cast_sign_loss, reason = "Size will never be negative")]
#[expect(clippy::cast_precision_loss, reason = "Size will never be that large")]
#[expect(clippy::cast_possible_truncation, reason = "Size will never be that large")]
fn interp_size(a: &mut Size, b: Size, t: f32) {
    let ceil_or_floor = matches!(Size::cmp(a, &b), Ordering::Less);
    let (w, h) = interp(a.width as f32, a.height as f32, b.width as f32, b.height as f32, t);
    if ceil_or_floor {
        #[cfg(feature = "libm")]
        {
            (a.width, a.height) = (libm::ceilf(w) as u32, libm::ceilf(h) as u32);
        }
        #[cfg(not(feature = "libm"))]
        {
            (a.width, a.height) = (w.ceil() as u32, h.ceil() as u32);
        }
    } else {
        #[cfg(feature = "libm")]
        {
            (a.width, a.height) = (libm::floorf(w) as u32, libm::floorf(h) as u32);
        }
        #[cfg(not(feature = "libm"))]
        {
            (a.width, a.height) = (w.floor() as u32, h.floor() as u32);
        }
    }
}

fn interp_angle(a: &mut f32, b: f32, t: f32) {
    let (a_rad, b_rad) = (a.to_radians(), b.to_radians());
    let diff = b_rad - a_rad;
    let dot = diff * diff;

    #[cfg(feature = "libm")]
    let len = libm::sqrtf(dot);
    #[cfg(not(feature = "libm"))]
    let len = dot.sqrt();

    if len <= t || len <= 1e-4 {
        *a = b;
    } else {
        *a = a_rad + diff / len * t;
    }
}

fn interp_line(a: &mut Line, b: &Line, t: f32) {
    interp_point(&mut a.start, b.start, t);
    interp_point(&mut a.end, b.end, t);
}
