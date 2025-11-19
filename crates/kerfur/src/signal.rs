//! A module containing signals used across the application.
//!
//! These are placed here, as each processor cannot access another's module.

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

/// A [`Signal`] that enables the audio subsystem.
///
/// Signaling this assumes that the peripherals have been configured.
pub(crate) static AUDIO_ENABLE: Signal<CriticalSectionRawMutex, ()> = Signal::new();
