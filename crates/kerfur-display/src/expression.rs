use embedded_graphics::{prelude::*, primitives::Line};

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
    pub const DAZED: KerfurElements = KerfurElements::new()
        .with_eyebrows(
            Line::new(
                Point::new(480 * 42 / 100, 480 * 22 / 100),
                Point::new(480 * 35 / 100, 480 * 25 / 100),
            ),
            Line::new(
                Point::new(480 * 58 / 100, 480 * 22 / 100),
                Point::new(480 * 65 / 100, 480 * 25 / 100),
            ),
        )
        .with_whiskers(
            Line::new(
                Point::new(480 * 7 / 100, 480 * 65 / 100),
                Point::new(480 * 0 / 100, 480 * 65 / 100),
            ),
            Line::new(
                Point::new(480 * 93 / 100, 480 * 65 / 100),
                Point::new(480 * 100 / 100, 480 * 65 / 100),
            ),
        );
    /// The [`KerfurElements`] for the [`KerfurEmote::Meow`] emote.
    pub const MEOW: KerfurElements = KerfurElements::new()
        .with_eyes(
            KerfurEyeType::Arrow(
                Line::new(Point::new(480 * 40 / 100, 248), Point::new(480 * 20 / 100, 170)),
                Line::new(Point::new(480 * 40 / 100, 232), Point::new(480 * 20 / 100, 310)),
            ),
            KerfurEyeType::Arrow(
                Line::new(Point::new(480 * 60 / 100, 248), Point::new(480 * 80 / 100, 170)),
                Line::new(Point::new(480 * 60 / 100, 232), Point::new(480 * 80 / 100, 310)),
            ),
        )
        .with_eyebrows(
            Line::new(
                Point::new(480 * 42 / 100, 480 * 40 / 100),
                Point::new(480 * 35 / 100, 480 * 35 / 100),
            ),
            Line::new(
                Point::new(480 * 58 / 100, 480 * 40 / 100),
                Point::new(480 * 65 / 100, 480 * 35 / 100),
            ),
        )
        .with_whiskers(
            Line::new(
                Point::new(480 * 7 / 100, 480 * 60 / 100),
                Point::new(480 * 0 / 100, 480 * 60 / 100),
            ),
            Line::new(
                Point::new(480 * 93 / 100, 480 * 60 / 100),
                Point::new(480 * 100 / 100, 480 * 60 / 100),
            ),
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
