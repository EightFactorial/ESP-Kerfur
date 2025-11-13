//! TODO
#![no_std]

use embedded_graphics::{
    pixelcolor::{BinaryColor, Rgb888},
    prelude::*,
};

pub mod element;
pub use element::KerfurElements;

mod expression;
pub use expression::{KerfurEmote, KerfurExpression};

mod style;
pub use style::KerfurStyle;

/// A display that draws Kerfur's face
pub struct KerfurDisplay<D: DrawTargetExt> {
    display: D,
    style: KerfurStyle<D::Color>,
    current: KerfurElements,
    target: KerfurElements,
    speed: f32,
}

impl<D: DrawTargetExt> KerfurDisplay<D> {
    /// Create a new [`KerfurDisplay`].
    #[must_use]
    pub fn new_with_style<E: KerfurExpression>(
        display: D,
        style: KerfurStyle<D::Color>,
        expression: E,
    ) -> Self {
        let elements = expression.into_elements();
        Self { display, style, current: elements, target: elements, speed: 1.0 }
    }

    /// Get a reference to the inner display.
    #[inline]
    #[must_use]
    pub const fn display(&self) -> &D { &self.display }

    /// Get a mutable reference to the inner display.
    #[inline]
    #[must_use]
    pub const fn display_mut(&mut self) -> &mut D { &mut self.display }

    /// Get a reference to the display style.
    #[inline]
    #[must_use]
    pub const fn style(&self) -> &KerfurStyle<D::Color> { &self.style }

    /// Get a mutable reference to the display style.
    #[inline]
    #[must_use]
    pub const fn style_mut(&mut self) -> &mut KerfurStyle<D::Color> { &mut self.style }

    /// Get the animation speed.
    #[inline]
    #[must_use]
    pub const fn speed(&mut self) -> f32 { self.speed }

    /// Set the animation speed.
    #[inline]
    pub const fn set_speed(&mut self, speed: f32) { self.speed = speed; }

    /// Get Kerfur's current expression.
    ///
    /// This is the same expression as the one drawn on the screen.
    #[inline]
    #[must_use]
    pub const fn get_expression(&self) -> KerfurElements { self.current }

    /// Get Kerfur's target expression.
    ///
    /// This is the expression that the current expression is
    /// gradually animating toward.
    #[inline]
    #[must_use]
    pub const fn get_expression_target(&self) -> KerfurElements { self.target }

    /// Set Kerfur's target expression.
    ///
    /// # Note
    ///
    /// This will not immediately change the expression,
    /// but will animate toward it over time.
    pub fn set_expression<E: KerfurExpression>(&mut self, expression: E) {
        self.target = expression.into_elements();
    }

    /// Set Kerfur's current expression.
    ///
    /// # Note
    ///
    /// This does not change the target expression,
    /// and will continue to animate toward it.
    pub fn set_expression_immediate<E: KerfurExpression>(&mut self, expression: E) {
        self.current = expression.into_elements();
    }

    /// Returns `true` if Kerfur is currently animating between expressions.
    #[must_use]
    pub fn is_animating(&self) -> bool { self.current != self.target }

    /// Clear the display with the given color.
    ///
    /// # Errors
    ///
    /// Returns an error if clearing the display fails.
    #[inline]
    pub fn clear(&mut self, color: D::Color) -> Result<(), D::Error> { self.display.clear(color) }

    /// Animate the display and draw the face
    ///
    /// # Warning
    ///
    /// This method does not flush the display!
    ///
    /// # Errors
    ///
    /// Returns an error if drawing to the display fails.
    pub fn draw(&mut self, tick: f32) -> Result<(), D::Error> {
        if self.is_animating() {
            self.current.interpolate(&self.target, tick * self.speed);
        }
        self.current.draw(&mut self.display, &self.style)
    }
}

// -------------------------------------------------------------------------------------------------

impl<D: DrawTargetExt<Color = Rgb888>> KerfurDisplay<D> {
    /// Create a new blue [`KerfurDisplay`].
    #[inline]
    pub fn blue<E: KerfurExpression>(display: D, expression: E) -> Self {
        Self::new_with_style(display, KerfurStyle::BLUE, expression)
    }

    /// Create a new pink [`KerfurDisplay`].
    #[inline]
    pub fn pink<E: KerfurExpression>(display: D, expression: E) -> Self {
        Self::new_with_style(display, KerfurStyle::PINK, expression)
    }

    /// Create a new red [`KerfurDisplay`].
    #[inline]
    pub fn red<E: KerfurExpression>(display: D, expression: E) -> Self {
        Self::new_with_style(display, KerfurStyle::RED, expression)
    }
}

impl<D: DrawTargetExt<Color = BinaryColor>> KerfurDisplay<D> {
    /// Create a new binary [`KerfurDisplay`].
    #[inline]
    pub fn binary<E: KerfurExpression>(display: D, expression: E) -> Self {
        Self::new_with_style(display, KerfurStyle::BINARY, expression)
    }
}
