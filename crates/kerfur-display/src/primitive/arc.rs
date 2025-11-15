use embedded_graphics::{
    prelude::{Angle, Point},
    primitives::{Arc, Circle},
};

/// An [`Arc`] primitive with `const` constructors
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct ConstArc {
    /// Top-left point of the bounding-box of the circle supporting the arc
    pub top_left: Point,

    /// Diameter of the circle supporting the arc
    pub diameter: u32,

    /// f32 at which the arc starts
    pub angle_start: f32,

    /// f32 defining the arc sweep starting at angle_start
    pub angle_sweep: f32,
}

impl ConstArc {
    /// Create a new arc delimited with a top-left point with a specific
    /// diameter and start and sweep angles
    pub const fn new(top_left: Point, diameter: u32, angle_start: f32, angle_sweep: f32) -> Self {
        ConstArc { top_left, diameter, angle_start, angle_sweep }
    }

    /// Create a new arc centered around a given point with a specific diameter
    /// and start and sweep angles
    pub const fn with_center(
        center: Point,
        diameter: u32,
        angle_start: f32,
        angle_sweep: f32,
    ) -> Self {
        Self::from_circle(Circle::with_center(center, diameter), angle_start, angle_sweep)
    }

    /// Creates an arc based on a circle.
    ///
    /// The resulting arc will match the `top_left` and `diameter` of the base
    /// circle.
    pub const fn from_circle(circle: Circle, angle_start: f32, angle_sweep: f32) -> Self {
        Self { top_left: circle.top_left, diameter: circle.diameter, angle_start, angle_sweep }
    }

    /// Convert this [`ConstArc`] into an [`Arc`].
    #[must_use]
    pub fn into_arc(self) -> Arc {
        Arc {
            top_left: self.top_left,
            diameter: self.diameter,
            angle_start: Angle::from_radians(self.angle_start),
            angle_sweep: Angle::from_radians(self.angle_sweep),
        }
    }
}
