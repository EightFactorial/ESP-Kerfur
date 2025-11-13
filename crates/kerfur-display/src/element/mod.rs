//! TODO

use embedded_graphics::{prelude::*, primitives::Line};

use crate::KerfurStyle;

mod eye;
pub use eye::KerfurEyeType;

mod mouth;
mod whisker;

/// A set of facial elements
#[derive(Clone, Copy, PartialEq, Eq)]
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
    /// Defaults to a dazed-looking face.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            eye: eye::EyeState { left: KerfurEyeType::Swirl, right: KerfurEyeType::Swirl },
            eyebrow: eye::EyebrowState {
                left: Line::new(Point::new_equal(0), Point::new_equal(0)),
                right: Line::new(Point::new_equal(0), Point::new_equal(0)),
            },
            mouth: mouth::MouthState { position: Point::new_equal(0) },
            whisker: whisker::WhiskerState {
                left: Line::new(Point::new_equal(0), Point::new_equal(0)),
                right: Line::new(Point::new_equal(0), Point::new_equal(0)),
                offset: Point::new(0, 8),
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
        &self,
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

fn interp(a: f32, b: f32, t: f32) -> f32 { a * (1.0 - t) + (b * t) }

fn interp_point(a: &mut Point, b: &Point, t: f32) {
    a.x = interp(a.x as f32, b.x as f32, t) as i32;
    a.y = interp(a.y as f32, b.y as f32, t) as i32;
}

fn interp_size(a: &mut Size, b: &Size, t: f32) {
    a.width = interp(a.width as f32, b.width as f32, t) as u32;
    a.height = interp(a.height as f32, b.height as f32, t) as u32;
}

fn interp_line(a: &mut Line, b: &Line, t: f32) {
    interp_point(&mut a.start, &b.start, t);
    interp_point(&mut a.end, &b.end, t);
}
