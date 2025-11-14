use core::sync::atomic::Ordering;

use embedded_graphics::{
    prelude::*,
    primitives::{Circle, Ellipse, Line, PrimitiveStyle, StyledDrawable},
};

use crate::{
    KerfurStyle,
    atomic::AtomicF32,
    element::{interp_line, interp_point, interp_size},
    primitive::Swirl,
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
    Swirl(Circle),
}

impl EyeState {
    pub(super) fn draw<D: DrawTargetExt>(
        &self,
        display: &mut D,
        style: &KerfurStyle<D::Color>,
    ) -> Result<(), D::Error> {
        Self::draw_eye(&self.left, display, &style.left_eye)?;
        Self::draw_eye(&self.right, display, &style.right_eye)?;
        Ok(())
    }

    fn draw_eye<D: DrawTargetExt>(
        eye: &KerfurEyeType,
        display: &mut D,
        style: &PrimitiveStyle<D::Color>,
    ) -> Result<(), D::Error> {
        match eye {
            KerfurEyeType::Ellipse(ellipse) => ellipse.draw_styled(style, display),
            KerfurEyeType::Line(line) => line.draw_styled(style, display),
            KerfurEyeType::Arrow(line_a, line_b) => {
                line_a.draw_styled(style, display)?;
                line_b.draw_styled(style, display)?;
                Ok(())
            }
            KerfurEyeType::Swirl(circle) => {
                /// The rotation of the eye's swirl
                static ROTATION: AtomicF32 = AtomicF32::new(0.0);

                // Get the current rotation and increment it for next time
                let rotation = ROTATION.load(Ordering::Relaxed);
                ROTATION.store(rotation + 0.1, Ordering::Relaxed);

                Swirl::with_center(circle.center(), Angle::from_radians(rotation), circle.diameter)
                    .draw_styled(style, display)?;

                Ok(())
            }
        }
    }

    pub(super) fn interpolate(&mut self, other: &Self, tick: f32) {
        Self::interpolate_eye(&mut self.left, &other.left, tick);
        Self::interpolate_eye(&mut self.right, &other.right, tick);
    }

    fn interpolate_eye(a: &mut KerfurEyeType, b: &KerfurEyeType, tick: f32) {
        match (a, b) {
            (KerfurEyeType::Ellipse(a), KerfurEyeType::Ellipse(b)) => {
                interp_size(&mut a.size, b.size, tick);
                interp_point(&mut a.top_left, b.top_left, tick);
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
