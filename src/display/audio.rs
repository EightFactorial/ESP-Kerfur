//! [`KerfAudio`] and audio trigger functionality.

use embassy_time::Timer;
use esp_hal::gpio::{DriveMode, Level, Output, OutputConfig, OutputPin};

/// An audio output trigger.
pub struct KerfAudio {
    /// The audio output pin.
    ///
    /// Used to signal when audio should be played.
    trigger: Output<'static>,
}

impl KerfAudio {
    /// Create a new [`KerfAudio`].
    #[must_use]
    pub fn new<T: OutputPin + 'static>(output: T) -> Self {
        Self {
            trigger: Output::new(
                output,
                Level::High,
                OutputConfig::default().with_drive_mode(DriveMode::OpenDrain),
            ),
        }
    }

    /// Trigger a meow sound effect to start playing.
    pub async fn meow(&mut self) {
        self.trigger.set_low();
        Timer::after_millis(50).await;
        self.trigger.set_high();
    }
}
