//! TODO
#![no_std]

#[cfg(feature = "std")]
extern crate std;

use core::ops::{Deref, DerefMut};

use embedded_graphics::{
    pixelcolor::{BinaryColor, Rgb888},
    prelude::*,
};

pub mod primitive;

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
    animating: bool,
}

impl<D: DrawTargetExt> KerfurDisplay<D> {
    /// Create a new [`KerfurDisplay`].
    #[inline]
    #[must_use]
    pub fn new_with_style<E: KerfurExpression>(
        display: D,
        style: KerfurStyle<D::Color>,
        expression: E,
    ) -> Self {
        Self::const_new_with_style(display, style, expression.into_elements())
    }

    /// Create a new [`KerfurDisplay`].
    #[must_use]
    pub const fn const_new_with_style(
        display: D,
        style: KerfurStyle<D::Color>,
        elements: KerfurElements,
    ) -> Self {
        Self { display, style, current: elements, target: elements, animating: false }
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
        self.animating = true;
        self.target = expression.into_elements();
    }

    /// Set Kerfur's current expression.
    ///
    /// # Note
    ///
    /// This does not change the target expression,
    /// and will continue to animate toward it.
    pub fn set_expression_immediate<E: KerfurExpression>(&mut self, expression: E) {
        self.animating = true;
        self.current = expression.into_elements();
    }

    /// Returns `true` if Kerfur is currently animating between expressions.
    #[must_use]
    pub fn is_animating(&self) -> bool { self.animating }

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
        if self.animating {
            self.current.interpolate(&self.target, tick);
            self.animating = self.current != self.target;
        }
        self.current.draw(&mut self.display, &self.style)
    }
}

impl<D: DrawTargetExt> Deref for KerfurDisplay<D> {
    type Target = D;

    #[inline]
    fn deref(&self) -> &Self::Target { &self.display }
}

impl<D: DrawTargetExt> DerefMut for KerfurDisplay<D> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.display }
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
