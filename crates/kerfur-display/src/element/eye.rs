use embedded_graphics::{
    prelude::*,
    primitives::{Ellipse, Line, PrimitiveStyle, StyledDrawable},
};

use crate::{
    KerfurStyle,
    element::{interp_angle, interp_line, interp_point, interp_size},
    primitive::{ConstSector, Swirl},
};

#[derive(Clone, Copy, PartialEq)]
pub(super) struct EyeState {
    pub(super) left: KerfurEyeType,
    pub(super) right: KerfurEyeType,
}

/// The type of eye to display
#[derive(Clone, Copy, PartialEq)]
pub enum KerfurEyeType {
    /// An outer and inner ellipse
    Ellipse(Ellipse, Ellipse),
    /// An outer and inner sector
    Arrow(ConstSector, ConstSector),
    /// A line
    Line(Line),
    /// A swirl
    Swirl(Swirl),
}

impl KerfurEyeType {
    /// The default, blinking left eye.
    pub const BLINK_LEFT: KerfurEyeType = KerfurEyeType::Line(Line::new(
        Point::new(480 * 8 / 100, 240),
        Point::new(480 * 40 / 100, 240),
    ));
    /// The default, blinking right eye.
    pub const BLINK_RIGHT: KerfurEyeType = KerfurEyeType::Line(Line::new(
        Point::new(480 * 60 / 100, 240),
        Point::new(480 * 92 / 100, 240),
    ));
    /// The default, neutral left eye.
    pub const NEUTRAL_LEFT: KerfurEyeType = KerfurEyeType::Ellipse(
        Ellipse::with_center(Point::new(480 * 24 / 100, 240), Size::new_equal(480 * 32 / 100)),
        Ellipse::with_center(Point::new(480 * 24 / 100, 240), Size::new_equal(480 * 22 / 100)),
    );
    /// The default, neutral right eye.
    pub const NEUTRAL_RIGHT: KerfurEyeType = KerfurEyeType::Ellipse(
        Ellipse::with_center(Point::new(480 * 76 / 100, 240), Size::new_equal(480 * 32 / 100)),
        Ellipse::with_center(Point::new(480 * 76 / 100, 240), Size::new_equal(480 * 22 / 100)),
    );

    /// Returns the eye with the pupil translated by the given amount.
    ///
    /// # Panics
    ///
    /// Panics if the eye is not of type [`KerfurEyeType::Ellipse`].
    #[must_use]
    pub const fn with_pupil_translated(mut self, by: Point) -> Self {
        let KerfurEyeType::Ellipse(_, inner) = &mut self else {
            panic!("Eye is not of type `KerfurEyeType::Ellipse`!")
        };

        inner.top_left.x += by.x;
        inner.top_left.y += by.y;

        self
    }

    /// Returns the eye with the pupil resized by the given amount.
    ///
    /// # Panics
    ///
    /// Panics if the eye is not of type [`KerfurEyeType::Ellipse`].
    #[must_use]
    pub const fn with_pupil_resized(mut self, by: Point) -> Self {
        let KerfurEyeType::Ellipse(_, inner) = &mut self else {
            panic!("Eye is not of type `KerfurEyeType::Ellipse`!")
        };

        inner.top_left.x -= by.x / 2;
        inner.top_left.y -= by.y / 2;

        if let Some(result) = inner.size.width.checked_add_signed(by.x) {
            inner.size.width = result;
        } else {
            inner.size.width = 0;
        }
        if let Some(result) = inner.size.height.checked_add_signed(by.y) {
            inner.size.height = result;
        } else {
            inner.size.height = 0;
        }

        self
    }
}

impl EyeState {
    pub(super) fn draw<D: DrawTargetExt>(
        &mut self,
        display: &mut D,
        style: &KerfurStyle<D::Color>,
    ) -> Result<(), D::Error> {
        Self::draw_eye(
            &mut self.left,
            display,
            &style.left_eye_inner,
            &style.left_eye_outer,
            &style.left_eye_line,
        )?;
        Self::draw_eye(
            &mut self.right,
            display,
            &style.right_eye_inner,
            &style.right_eye_outer,
            &style.right_eye_line,
        )
    }

    fn draw_eye<D: DrawTargetExt>(
        eye: &mut KerfurEyeType,
        display: &mut D,
        inner: &PrimitiveStyle<D::Color>,
        outer: &PrimitiveStyle<D::Color>,
        line: &PrimitiveStyle<D::Color>,
    ) -> Result<(), D::Error> {
        match eye {
            KerfurEyeType::Ellipse(ellipse_a, ellipse_b) => {
                ellipse_a.draw_styled(outer, display)?;
                ellipse_b.draw_styled(inner, display)
            }
            KerfurEyeType::Arrow(sector_a, sector_b) => {
                sector_a.into_sector().draw_styled(outer, display)?;
                sector_b.into_sector().draw_styled(inner, display)
            }
            KerfurEyeType::Line(eye) => eye.draw_styled(line, display),
            KerfurEyeType::Swirl(swirl) => swirl.draw_styled(outer, display),
        }
    }

    pub(super) fn interpolate(&mut self, other: &Self, tick: f32) {
        Self::interpolate_eye(&mut self.left, &other.left, tick);
        Self::interpolate_eye(&mut self.right, &other.right, tick);
    }

    fn interpolate_eye(a: &mut KerfurEyeType, b: &KerfurEyeType, tick: f32) {
        match (a, b) {
            (KerfurEyeType::Ellipse(a1, a2), KerfurEyeType::Ellipse(b1, b2)) => {
                interp_size(&mut a1.size, b1.size, tick);
                interp_point(&mut a1.top_left, b1.top_left, tick);
                interp_size(&mut a2.size, b2.size, tick);
                interp_point(&mut a2.top_left, b2.top_left, tick);
            }
            (KerfurEyeType::Line(a), KerfurEyeType::Line(b)) => {
                interp_line(a, b, tick);
            }
            (KerfurEyeType::Arrow(a1, a2), KerfurEyeType::Arrow(b1, b2)) => {
                interp_point(&mut a1.top_left, b1.top_left, tick);
                interp_angle(&mut a1.angle_start, b1.angle_start, tick);
                interp_angle(&mut a1.angle_sweep, b1.angle_sweep, tick);
                interp_diameter(&mut a1.diameter, b1.diameter, tick);
                interp_point(&mut a2.top_left, b2.top_left, tick);
                interp_angle(&mut a2.angle_start, b2.angle_start, tick);
                interp_angle(&mut a2.angle_sweep, b2.angle_sweep, tick);
                interp_diameter(&mut a2.diameter, b2.diameter, tick);
            }
            (KerfurEyeType::Swirl(a), KerfurEyeType::Swirl(b)) => {
                interp_point(&mut a.circle.top_left, b.circle.top_left, tick);
            }
            // Immediately use the new eye shape and snap to the final position
            (current, other) => *current = *other,
        }
    }
}

// Interpolate the diameter of an arc.
// TODO: Stop being lazy and make a proper function.
fn interp_diameter(a: &mut u32, b: u32, tick: f32) {
    let mut size = Size::new(*a, 0);
    interp_size(&mut size, Size::new(b, 0), tick);
    *a = size.width;
}

// -------------------------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct EyebrowState {
    pub(super) left: Line,
    pub(super) right: Line,
}

impl EyebrowState {
    pub(super) fn draw<D: DrawTargetExt>(
        &mut self,
        display: &mut D,
        style: &KerfurStyle<D::Color>,
    ) -> Result<(), D::Error> {
        self.left.draw_styled(&style.left_eyebrow, display)?;
        self.right.draw_styled(&style.right_eyebrow, display)?;
        Ok(())
    }

    pub(super) fn interpolate(&mut self, other: &Self, tick: f32) {
        interp_line(&mut self.left, &other.left, tick);
        interp_line(&mut self.right, &other.right, tick);
    }
}
