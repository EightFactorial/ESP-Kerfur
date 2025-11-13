use embedded_graphics::{
    prelude::*,
    primitives::{Circle, Line},
};

use crate::{KerfurElements, element::KerfurEyeType};

/// A set of default Kerfur expressions.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum KerfurEmote {
    /// A neutral face
    #[default]
    Neutral,
    /// A meowing face
    Meow,
    /// A dazed face
    Dazed,
}

impl KerfurEmote {
    /// The [`KerfurElements`] for the [`KerfurEmote::Dazed`] emote.
    pub const DAZED: KerfurElements = KerfurElements::new().with_eyes(
        KerfurEyeType::Swirl(Circle::with_center(Point::zero(), 8)),
        KerfurEyeType::Swirl(Circle::with_center(Point::zero(), 8)),
    );
    /// The [`KerfurElements`] for the [`KerfurEmote::Meow`] emote.
    pub const MEOW: KerfurElements = KerfurElements::new()
        .with_eyes(
            KerfurEyeType::Arrow(
                Line::new(Point::zero(), Point::zero()),
                Line::new(Point::zero(), Point::zero()),
            ),
            KerfurEyeType::Arrow(
                Line::new(Point::zero(), Point::zero()),
                Line::new(Point::zero(), Point::zero()),
            ),
        )
        .with_eyebrows(
            Line::new(Point::zero(), Point::zero()),
            Line::new(Point::zero(), Point::zero()),
        );
    /// The [`KerfurElements`] for the [`KerfurEmote::Neutral`] emote.
    pub const NEUTRAL: KerfurElements = KerfurElements::new();
}

impl KerfurExpression for KerfurEmote {
    fn into_elements(self) -> KerfurElements {
        match self {
            KerfurEmote::Neutral => Self::NEUTRAL,
            KerfurEmote::Meow => Self::MEOW,
            KerfurEmote::Dazed => Self::DAZED,
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// A trait for types that can become a set of facial elements.
pub trait KerfurExpression {
    /// Create a set of [`KerfurElements`].
    fn into_elements(self) -> KerfurElements;
}

impl KerfurExpression for KerfurElements {
    #[inline]
    fn into_elements(self) -> KerfurElements { self }
}
