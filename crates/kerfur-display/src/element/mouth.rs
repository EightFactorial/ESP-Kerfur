use embedded_graphics::{prelude::*, primitives::StyledDrawable};

use crate::{
    KerfurStyle,
    element::{interp_angle, interp_point, interp_size},
    primitive::{ConstArc, ConstSector},
};

#[derive(Clone, Copy, PartialEq)]
pub(super) struct MouthState {
    pub(super) nose: ConstSector,
    pub(super) mouth_left: ConstArc,
    pub(super) mouth_right: ConstArc,
    pub(super) mouth_bottom: ConstArc,
}

#[expect(warnings, reason = "WIP")]
impl MouthState {
    pub(super) fn draw<D: DrawTargetExt>(
        &mut self,
        display: &mut D,
        style: &KerfurStyle<D::Color>,
    ) -> Result<(), D::Error> {
        self.mouth_bottom.into_arc().draw_styled(&style.mouth_bottom, display)?;
        self.mouth_left.into_arc().draw_styled(&style.mouth, display)?;
        self.mouth_right.into_arc().draw_styled(&style.mouth, display)?;
        self.nose.into_sector().draw_styled(&style.nose, display)
    }

    pub(super) fn interpolate(&mut self, other: &Self, tick: f32) {
        Self::interpolate_sector(&mut self.nose, &other.nose, tick);
        Self::interpolate_arc(&mut self.mouth_left, &other.mouth_left, tick);
        Self::interpolate_arc(&mut self.mouth_right, &other.mouth_right, tick);
        Self::interpolate_arc(&mut self.mouth_bottom, &other.mouth_bottom, tick);
    }

    fn interpolate_sector(a: &mut ConstSector, b: &ConstSector, tick: f32) {
        interp_point(&mut a.top_left, b.top_left, tick);
        interp_angle(&mut a.angle_start, b.angle_start, tick);
        interp_angle(&mut a.angle_sweep, b.angle_sweep, tick);
        interp_diameter(&mut a.diameter, b.diameter, tick);
    }

    fn interpolate_arc(a: &mut ConstArc, b: &ConstArc, tick: f32) {
        interp_point(&mut a.top_left, b.top_left, tick);
        interp_angle(&mut a.angle_start, b.angle_start, tick);
        interp_angle(&mut a.angle_sweep, b.angle_sweep, tick);
        interp_diameter(&mut a.diameter, b.diameter, tick);
    }
}

// Interpolate the diameter of an arc.
// TODO: Stop being lazy and make a proper function.
fn interp_diameter(a: &mut u32, b: u32, tick: f32) {
    let mut size = Size::new(*a, 1);
    interp_size(&mut size, Size::new(b, 1), tick);
    *a = size.width;
}
