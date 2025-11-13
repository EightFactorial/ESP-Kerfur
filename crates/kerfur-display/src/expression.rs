use embedded_graphics::{prelude::*, primitives::Ellipse};

use crate::{KerfurElements, element::KerfurEyeType};

/// A set of default Kerfur expressions.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum KerfurEmote {
    /// A dazed face
    #[default]
    Dazed,
    /// A neutral face
    Neutral,
}

impl KerfurEmote {
    /// The [`KerfurElements`] for the [`KerfurEmote::Dazed`] emote.
    pub const DAZED: KerfurElements = KerfurElements::new();
    /// The [`KerfurElements`] for the [`KerfurEmote::Neutral`] emote.
    pub const NEUTRAL: KerfurElements = KerfurElements::new().with_eyes(
        KerfurEyeType::Ellipse(Ellipse::with_center(Point::zero(), Size::zero())),
        KerfurEyeType::Ellipse(Ellipse::with_center(Point::zero(), Size::zero())),
    );
}

impl KerfurExpression for KerfurEmote {
    fn into_elements(self) -> KerfurElements {
        match self {
            KerfurEmote::Dazed => Self::DAZED,
            KerfurEmote::Neutral => Self::NEUTRAL,
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
