use embedded_graphics::{prelude::*, primitives::Line};

use crate::{KerfurStyle, element::interp_line};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct WhiskerState {
    pub(super) left: Line,
    pub(super) right: Line,
    pub(super) offset: Point,
    pub(super) count: u32,
}

impl WhiskerState {
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
