//! [`KerfTouch`] and it's touch sensor.

use esp_hal::gpio::{Input, InputConfig, InputPin, Pull};

/// Kerfur's touch sensor interface.
pub struct KerfTouch {
    /// The input pin connected to the touch sensor.
    input: Input<'static>,
}

impl KerfTouch {
    /// Create a new [`KerfTouch`].
    #[must_use]
    pub fn new<T: InputPin + 'static>(input: T) -> Self {
        Self { input: Input::new(input, InputConfig::default().with_pull(Pull::Down)) }
    }

    /// Returns `true` if the sensor is currently being touched (high).
    #[must_use]
    pub fn is_touched(&self) -> bool { self.input.is_high() }

    /// Returns `true` if the sensor is currently released (low).
    #[must_use]
    pub fn is_released(&self) -> bool { self.input.is_low() }

    /// Wait until the sensor is touched (goes high).
    ///
    /// See [`Input::wait_for`] for more information.
    pub async fn wait_for_touch(&mut self) { self.input.wait_for_high().await }

    /// Wait until the sensor is released (goes low).
    ///
    /// See [`Input::wait_for`] for more information.
    pub async fn wait_for_release(&mut self) { self.input.wait_for_low().await }
}
