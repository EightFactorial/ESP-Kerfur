use embedded_graphics::{
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Rectangle, StyledDrawable},
};

/// Swirl primitive
#[derive(Clone, Copy, PartialEq)]
pub struct Swirl {
    /// The angle of the swirl
    pub angle: Angle,
    /// The circle to draw the swirl in
    pub circle: Circle,
}

impl Swirl {
    /// Create a new [`Swirl`].
    #[must_use]
    pub const fn new(top_left: Point, angle: Angle, diameter: u32) -> Self {
        Self { angle, circle: Circle::new(top_left, diameter) }
    }

    /// Create a new [`Swirl`] with the given center point.
    #[must_use]
    pub const fn with_center(center: Point, angle: Angle, diameter: u32) -> Self {
        Self { angle, circle: Circle::with_center(center, diameter) }
    }
}

// -------------------------------------------------------------------------------------------------

impl Primitive for Swirl {}

impl ContainsPoint for Swirl {
    #[inline]
    fn contains(&self, point: Point) -> bool { Circle::contains(&self.circle, point) }
}

impl Dimensions for Swirl {
    #[inline]
    fn bounding_box(&self) -> Rectangle { Circle::bounding_box(&self.circle) }
}

impl Transform for Swirl {
    fn translate(&self, by: Point) -> Self {
        Self { angle: self.angle, circle: self.circle.translate(by) }
    }

    #[inline]
    fn translate_mut(&mut self, by: Point) -> &mut Self {
        Circle::translate_mut(&mut self.circle, by);
        self
    }
}

// -------------------------------------------------------------------------------------------------

pub struct SwirlPointIter {}

impl PointsIter for Swirl {
    type Iter = SwirlPointIter;

    fn points(&self) -> Self::Iter { todo!() }
}

impl Iterator for SwirlPointIter {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> { todo!() }
}

// -------------------------------------------------------------------------------------------------

impl<S: PixelColor> StyledDrawable<PrimitiveStyle<S>> for Swirl {
    type Color = S;
    type Output = ();

    fn draw_styled<D>(
        &self,
        _style: &PrimitiveStyle<S>,
        _target: &mut D,
    ) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        todo!()
    }
}
