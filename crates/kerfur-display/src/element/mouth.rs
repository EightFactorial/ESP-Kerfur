use embedded_graphics::prelude::*;

use crate::KerfurStyle;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct MouthState {
    pub(super) position: Point,
}

impl MouthState {
    pub(super) fn draw<D: DrawTargetExt>(
        &self,
        _display: &mut D,
        _style: &KerfurStyle<D::Color>,
    ) -> Result<(), D::Error> {
        Ok(())
    }

    pub(super) fn interpolate(&mut self, _other: &Self, _tick: f32) {}
}
