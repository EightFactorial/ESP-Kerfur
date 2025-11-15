use embedded_graphics::{
    prelude::{Angle, Point, Size},
    primitives::{Rectangle, Sector},
};

/// A [`Sector`] primitive with `const` constructors
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct ConstSector {
    /// Top-left point of the bounding-box of the circle supporting the sector
    pub top_left: Point,

    /// Diameter of the circle supporting the sector
    pub diameter: u32,

    /// f32 at which the sector starts
    pub angle_start: f32,

    /// f32 defining the sector sweep starting at angle_start
    pub angle_sweep: f32,
}

impl ConstSector {
    /// Create a new sector delimited with a top-left point with a specific
    /// diameter and start and sweep angles
    #[must_use]
    pub const fn new(top_left: Point, diameter: u32, angle_start: f32, angle_sweep: f32) -> Self {
        ConstSector { top_left, diameter, angle_start, angle_sweep }
    }

    /// Create a new sector centered around a given point with a specific
    /// diameter and start and sweep angles
    #[must_use]
    pub const fn with_center(
        center: Point,
        diameter: u32,
        angle_start: f32,
        angle_sweep: f32,
    ) -> Self {
        let top_left = Rectangle::with_center(center, Size::new_equal(diameter)).top_left;

        ConstSector { top_left, diameter, angle_start, angle_sweep }
    }

    /// Convert to an `embedded_graphics` [`Sector`].
    #[must_use]
    pub fn into_sector(self) -> Sector {
        Sector {
            top_left: self.top_left,
            diameter: self.diameter,
            angle_start: Angle::from_radians(self.angle_start),
            angle_sweep: Angle::from_radians(self.angle_sweep),
        }
    }
}
