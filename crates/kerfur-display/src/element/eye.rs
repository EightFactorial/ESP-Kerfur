use embedded_graphics::{
    prelude::*,
    primitives::{Ellipse, Line},
};

use crate::{
    KerfurStyle,
    element::{interp_line, interp_point, interp_size},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct EyeState {
    pub(super) left: KerfurEyeType,
    pub(super) right: KerfurEyeType,
}

/// The type of eye to display
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum KerfurEyeType {
    /// An ellipse
    Ellipse(Ellipse),
    /// One line
    Line(Line),
    /// Two lines
    Arrow(Line, Line),
    /// A swirl
    Swirl,
}

impl EyeState {
    pub(super) fn draw<D: DrawTargetExt>(
        &self,
        _display: &mut D,
        _style: &KerfurStyle<D::Color>,
    ) -> Result<(), D::Error> {
        Ok(())
    }

    pub(super) fn interpolate(&mut self, other: &Self, tick: f32) {
        Self::interpolate_eye(&mut self.left, &other.left, tick);
        Self::interpolate_eye(&mut self.right, &other.right, tick);
    }

    fn interpolate_eye(a: &mut KerfurEyeType, b: &KerfurEyeType, tick: f32) {
        match (a, b) {
            (KerfurEyeType::Ellipse(a), KerfurEyeType::Ellipse(b)) => {
                interp_size(&mut a.size, &b.size, tick);
                interp_point(&mut a.top_left, &b.top_left, tick);
            }
            (KerfurEyeType::Line(a), KerfurEyeType::Line(b)) => {
                interp_line(a, b, tick);
            }
            (KerfurEyeType::Arrow(a1, a2), KerfurEyeType::Arrow(b1, b2)) => {
                interp_line(a1, b1, tick);
                interp_line(a2, b2, tick);
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
        &self,
        _display: &mut D,
        _style: &KerfurStyle<D::Color>,
    ) -> Result<(), D::Error> {
        Ok(())
    }

    pub(super) fn interpolate(&mut self, other: &Self, tick: f32) {
        interp_line(&mut self.left, &other.left, tick);
        interp_line(&mut self.right, &other.right, tick);
    }
}
