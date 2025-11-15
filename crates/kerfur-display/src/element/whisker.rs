use embedded_graphics::{
    prelude::*,
    primitives::{Line, PrimitiveStyle, StyledDrawable},
};

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
        &mut self,
        display: &mut D,
        style: &KerfurStyle<D::Color>,
    ) -> Result<(), D::Error> {
        Self::draw_whisker(self.left, self.offset, self.count, display, &style.whisker)?;
        Self::draw_whisker(self.right, self.offset, self.count, display, &style.whisker)?;
        Ok(())
    }

    fn draw_whisker<D: DrawTargetExt>(
        mut whisker: Line,
        offset: Point,
        count: u32,
        display: &mut D,
        style: &PrimitiveStyle<D::Color>,
    ) -> Result<(), D::Error> {
        for _ in 0..count {
            whisker.draw_styled(style, display)?;
            whisker.translate_mut(offset);
        }
        Ok(())
    }

    pub(super) fn interpolate(&mut self, other: &Self, tick: f32) {
        interp_line(&mut self.left, &other.left, tick);
        interp_line(&mut self.right, &other.right, tick);
    }
}
