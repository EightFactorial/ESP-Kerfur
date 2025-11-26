//! A module containing signals used across the application.
//!
//! These are placed here, as each processor cannot access another's module.
#![expect(dead_code, reason = "Work in progress")]

use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, signal::Signal,
};
use embassy_time::Duration;
use esp_hal::i2s::master::Config as I2sConfig;
use kerfur_display::{KerfurElements, KerfurExpression};

/// A [`Signal`] that enables the audio subsystem.
///
/// Signaling this assumes that the peripherals have been configured.
pub(crate) static AUDIO_CFG: Signal<CriticalSectionRawMutex, I2sConfig> = Signal::new();

// -------------------------------------------------------------------------------------------------

/// A [`Channel`] for sending commands to the display task.
///
/// Commands sent through this channel will be executed by the display task.
pub(crate) static DISPLAY_CMD: Channel<CriticalSectionRawMutex, DisplayCommand, 8> = Channel::new();

/// A command sent to the display task.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum DisplayCommand {
    /// Push an emote to the display.
    Push(KerfurElements),
    /// Push an emote to the display for a duration.
    PushHold(KerfurElements, Duration),
}

impl DisplayCommand {
    /// Create a new [`DisplayCommand::Push`].
    #[inline]
    #[must_use]
    pub(crate) fn new<T: KerfurExpression>(expression: T) -> Self {
        Self::Push(expression.into_elements())
    }

    /// Create a new [`DisplayCommand::PushHold`].
    #[inline]
    #[must_use]
    pub(crate) fn new_hold<T: KerfurExpression>(expression: T, duration: Duration) -> Self {
        Self::PushHold(expression.into_elements(), duration)
    }
}
