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
        self.nose.into_sector().draw_styled(&style.nose, display)?;
        self.mouth_bottom.into_arc().draw_styled(&style.mouth_bottom, display)?;
        self.mouth_left.into_arc().draw_styled(&style.mouth, display)?;
        self.mouth_right.into_arc().draw_styled(&style.mouth, display)
    }

    pub(super) fn interpolate(&mut self, other: &Self, tick: f32) {
        interp_point(&mut self.nose.top_left, other.nose.top_left, tick);
        interp_angle(&mut self.nose.angle_start, other.nose.angle_start, tick);
        interp_angle(&mut self.nose.angle_sweep, other.nose.angle_sweep, tick);
        interp_diameter(&mut self.nose.diameter, other.nose.diameter, tick);

        interp_point(&mut self.mouth_left.top_left, other.mouth_left.top_left, tick);
        interp_angle(&mut self.mouth_left.angle_start, other.mouth_left.angle_start, tick);
        interp_angle(&mut self.mouth_left.angle_sweep, other.mouth_left.angle_sweep, tick);
        interp_diameter(&mut self.mouth_left.diameter, other.mouth_left.diameter, tick);

        interp_point(&mut self.mouth_right.top_left, other.mouth_right.top_left, tick);
        interp_angle(&mut self.mouth_right.angle_start, other.mouth_right.angle_start, tick);
        interp_angle(&mut self.mouth_right.angle_sweep, other.mouth_right.angle_sweep, tick);
        interp_diameter(&mut self.mouth_right.diameter, other.mouth_right.diameter, tick);

        interp_point(&mut self.mouth_bottom.top_left, other.mouth_bottom.top_left, tick);
        interp_angle(&mut self.mouth_bottom.angle_start, other.mouth_bottom.angle_start, tick);
        interp_angle(&mut self.mouth_bottom.angle_sweep, other.mouth_bottom.angle_sweep, tick);
        interp_diameter(&mut self.mouth_bottom.diameter, other.mouth_bottom.diameter, tick);
    }
}

// Interpolate the diameter of an arc.
// TODO: Stop being lazy and make a proper function.
fn interp_diameter(a: &mut u32, b: u32, tick: f32) {
    let mut size = Size::new(*a, 0);
    interp_size(&mut size, Size::new(b, 0), tick);
    *a = size.width;
}
