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
}

impl EyeState {
    pub(super) fn draw<D: DrawTargetExt>(
        &mut self,
        display: &mut D,
        style: &KerfurStyle<D::Color>,
    ) -> Result<(), D::Error> {
        Self::draw_eye(&mut self.left, display, &style.left_eye_inner, &style.left_eye_outer)?;
        Self::draw_eye(&mut self.right, display, &style.right_eye_inner, &style.right_eye_outer)?;
        Ok(())
    }

    fn draw_eye<D: DrawTargetExt>(
        eye: &mut KerfurEyeType,
        display: &mut D,
        inner: &PrimitiveStyle<D::Color>,
        outer: &PrimitiveStyle<D::Color>,
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
            KerfurEyeType::Line(line) => line.draw_styled(outer, display),
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
                interp_point(&mut a2.top_left, b2.top_left, tick);
                interp_angle(&mut a2.angle_start, b2.angle_start, tick);
                interp_angle(&mut a2.angle_sweep, b2.angle_sweep, tick);
            }
            (KerfurEyeType::Swirl(a), KerfurEyeType::Swirl(b)) => {
                interp_point(&mut a.circle.top_left, b.circle.top_left, tick);
            }
            // Immediately use the new eye shape and snap to the final position
            (current, other) => *current = *other,
        }
    }
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
