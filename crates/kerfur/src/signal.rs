//! A module containing signals used across the application.
//!
//! These are placed here, as each processor cannot access another's module.

use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, signal::Signal,
};

use crate::app::DisplayCommand;

/// A [`Signal`] that enables the audio subsystem.
///
/// Signaling this assumes that the peripherals have been configured.
pub(crate) static AUDIO_ENABLE: Signal<CriticalSectionRawMutex, ()> = Signal::new();

/// A [`Channel`] for sending commands to the display task.
///
/// Commands sent through this channel will be executed by the display task.
#[expect(unused, reason = "Display driver not yet written")]
pub(crate) static EMOTION_DISPLAY: Channel<CriticalSectionRawMutex, DisplayCommand, 8> =
    Channel::new();
