use core::f32::consts::{FRAC_PI_6, FRAC_PI_8, PI};

use embedded_graphics::{prelude::*, primitives::Line};

use crate::{
    KerfurElements,
    element::KerfurEyeType,
    primitive::{ConstArc, ConstSector},
};

/// A set of default Kerfur expressions.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum KerfurEmote {
    /// A neutral face
    #[default]
    Neutral,
    /// A neutral face looking up
    NeutralUp,
    /// A neutral face looking down
    NeutralDown,
    /// A neutral face looking left
    NeutralLeft,
    /// A neutral face looking right
    NeutralRight,

    /// A meowing face
    Meow,
    /// A dazed face
    Dazed,
}

impl KerfurExpression for KerfurEmote {
    fn into_elements(self) -> KerfurElements {
        match self {
            KerfurEmote::Neutral => Self::NEUTRAL,
            KerfurEmote::NeutralUp => Self::NEUTRAL_UP,
            KerfurEmote::NeutralDown => Self::NEUTRAL_DOWN,
            KerfurEmote::NeutralLeft => Self::NEUTRAL_LEFT,
            KerfurEmote::NeutralRight => Self::NEUTRAL_RIGHT,
            KerfurEmote::Meow => Self::MEOW,
            KerfurEmote::Dazed => Self::DAZED,
        }
    }
}

impl KerfurEmote {
    /// The [`KerfurElements`] for the [`KerfurEmote::Neutral`] emote.
    pub const NEUTRAL: KerfurElements = KerfurElements::new();
    /// The [`KerfurElements`] for the [`KerfurEmote::NeutralDown`] emote.
    pub const NEUTRAL_DOWN: KerfurElements = KerfurElements::new().with_eyes(
        KerfurEyeType::NEUTRAL_LEFT.with_pupil_translated(Point::new(0, 12)),
        KerfurEyeType::NEUTRAL_RIGHT.with_pupil_translated(Point::new(0, 12)),
    );
    /// The [`KerfurElements`] for the [`KerfurEmote::NeutralLeft`] emote.
    pub const NEUTRAL_LEFT: KerfurElements = KerfurElements::new().with_eyes(
        KerfurEyeType::NEUTRAL_LEFT.with_pupil_translated(Point::new(-12, 0)),
        KerfurEyeType::NEUTRAL_RIGHT.with_pupil_translated(Point::new(-12, 0)),
    );
    /// The [`KerfurElements`] for the [`KerfurEmote::NeutralRight`] emote.
    pub const NEUTRAL_RIGHT: KerfurElements = KerfurElements::new().with_eyes(
        KerfurEyeType::NEUTRAL_LEFT.with_pupil_translated(Point::new(12, 0)),
        KerfurEyeType::NEUTRAL_RIGHT.with_pupil_translated(Point::new(12, 0)),
    );
    /// The [`KerfurElements`] for the [`KerfurEmote::NeutralUp`] emote.
    pub const NEUTRAL_UP: KerfurElements = KerfurElements::new().with_eyes(
        KerfurEyeType::NEUTRAL_LEFT.with_pupil_translated(Point::new(0, -12)),
        KerfurEyeType::NEUTRAL_RIGHT.with_pupil_translated(Point::new(0, -12)),
    );
}

impl KerfurEmote {
    /// The [`KerfurElements`] for the [`KerfurEmote::Dazed`] emote.
    pub const DAZED: KerfurElements = KerfurElements::new().with_eyebrows(
        Line::new(
            Point::new(480 * 42 / 100, 480 * 22 / 100),
            Point::new(480 * 35 / 100, 480 * 25 / 100),
        ),
        Line::new(
            Point::new(480 * 58 / 100, 480 * 22 / 100),
            Point::new(480 * 65 / 100, 480 * 25 / 100),
        ),
    );
    /// The [`KerfurElements`] for the [`KerfurEmote::Meow`] emote.
    pub const MEOW: KerfurElements = KerfurElements::new()
        .with_eyes(
            KerfurEyeType::Arrow(
                ConstSector::with_center(
                    Point::new(480 * 46 / 100, 235),
                    480 * 75 / 100,
                    PI - FRAC_PI_8,
                    2. * FRAC_PI_8,
                ),
                ConstSector::with_center(
                    Point::new(480 * 34 / 100, 235),
                    480 * 55 / 100,
                    PI - (PI / 10.0),
                    2. * (PI / 10.0),
                ),
            ),
            KerfurEyeType::Arrow(
                ConstSector::with_center(
                    Point::new(480 * 54 / 100, 235),
                    480 * 75 / 100,
                    (2. * PI) - FRAC_PI_8,
                    2. * FRAC_PI_8,
                ),
                ConstSector::with_center(
                    Point::new(480 * 66 / 100, 235),
                    480 * 55 / 100,
                    (2. * PI) - (PI / 10.0),
                    2. * (PI / 10.0),
                ),
            ),
        )
        .with_eyebrows(
            Line::new(
                Point::new(480 * 42 / 100, 480 * 34 / 100),
                Point::new(480 * 35 / 100, 480 * 33 / 100),
            ),
            Line::new(
                Point::new(480 * 58 / 100, 480 * 34 / 100),
                Point::new(480 * 65 / 100, 480 * 33 / 100),
            ),
        )
        .with_mouth_bottom(ConstArc::with_center(
            Point::new(240, 480 * 64 / 100),
            50,
            -FRAC_PI_6,
            PI + 2. * FRAC_PI_6,
        ));
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
