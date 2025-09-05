//! TODO
#![expect(dead_code, reason = "Work in progress rewrite")]
#![feature(never_type, type_alias_impl_trait)]
#![no_main]
#![no_std]

use embassy_executor::Spawner;
use embassy_time::Timer;
use esp_hal::{i2c::master::Error as I2cError, rng::Rng, timer::timg::TimerGroup};
use futures_lite::future;
use log::{error, info};

mod init;

mod display;
use display::{KerfAudio, KerfDisplay, KerfEmote, KerfTouch};

mod network;
use network::{KerfClock, KerfWifi};

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) -> ! {
    // Initialize peripherals.
    let per = init::init();

    // Initialize the embassy runtime.
    let timer = TimerGroup::new(per.TIMG0);
    esp_hal_embassy::init(timer.timer0);

    // Create a display and draw the confused screen.
    let mut display = KerfDisplay::new(per.I2C0, per.GPIO3, per.GPIO4);
    if let Err(err) = KerfEmote::Confused.draw(&mut display) {
        error!("Failed to draw startup emote: {err}");
    }

    // Create an RNG source and clock.
    let rand = Rng::new(per.RNG);
    let clock = KerfClock::new();

    // Connect to WiFi and synchronize the clock.
    let timer = TimerGroup::new(per.TIMG1);
    if let Ok(wifi) = KerfWifi::new(per.WIFI, timer.timer0, rand, spawner).await {
        clock.synchronize(wifi, spawner);
    }

    // Create audio output and touch sensor.
    let audio = KerfAudio::new(per.GPIO21);
    let touch = KerfTouch::new(per.GPIO5);

    // And finally start the display manager.
    Kerfur::new(audio, clock, display, touch, rand).execute().await
}

// -------------------------------------------------------------------------------------------------

/// Kerfur's brain.
///
/// Manages all display, audio, time, and touch logic.
pub struct Kerfur {
    /// The audio output to use.
    audio: KerfAudio,
    /// The clock to use for timekeeping.
    clock: KerfClock,
    /// The display to render to.
    display: KerfDisplay,
    /// The touch sensor to detect touches.
    touch: KerfTouch,

    /// A source of randomness.
    rand: Rng,
}

impl Kerfur {
    /// Create a new [`KerfDisplay`].
    #[must_use]
    pub fn new(
        audio: KerfAudio,
        clock: KerfClock,
        display: KerfDisplay,
        touch: KerfTouch,
        rand: Rng,
    ) -> Self {
        Self { audio, clock, display, touch, rand }
    }

    /// Wrap the main execution loop, restarting on error.
    pub async fn execute(mut self) -> ! {
        loop {
            let Err(err) = self.execute_inner().await;
            error!("Display Manager encountered an error: {err}");
            Timer::after_secs(10).await;
            info!("Restarting Display Manager...");
        }
    }

    // ----------------------------------------------------------------------------------------------

    /// The main execution loop, handling all display updates and interactions.
    async fn execute_inner(&mut self) -> Result<!, I2cError> {
        loop {
            // Reset to the neutral emote.
            KerfEmote::Neutral.draw(&mut self.display)?;

            // Wait for either a blink or a touch.
            match future::or(
                Kerfur::wait_for_blink(&mut self.rand),
                Kerfur::wait_for_touch(&mut self.touch),
            )
            .await
            {
                // Draw the `Blink` emote for a very short time.
                KerfAction::Blink => {
                    KerfEmote::Blink.draw(&mut self.display)?;
                    Timer::after_millis(100).await;
                }
                // Wait for the touch to be released, then draw the `Meow` emote.
                KerfAction::Touch => {
                    self.touch.wait_for_release().await;
                    KerfEmote::Meow.draw(&mut self.display)?;
                    Timer::after_millis(650).await;
                }
            }
        }
    }

    /// Wait for a random amount of time before blinking.
    async fn wait_for_blink(rand: &mut Rng) -> KerfAction {
        // Have a ~1/32 chance to quickly blink, otherwise wait longer.
        let delay = rand.random();
        if delay.is_multiple_of(32) {
            // Wait between 1.0 (0+1000) and 1.499 (499+1000) seconds
            Timer::after_millis(u64::from(delay % 500 + 1000)).await;
        } else {
            // Wait between 3.0 (0+3000) and 6.999 (3999+3000) seconds
            Timer::after_millis(u64::from(delay % 4000 + 3000)).await;
        }

        KerfAction::Blink
    }

    /// Wait for a touch event.
    async fn wait_for_touch(touch: &mut KerfTouch) -> KerfAction {
        touch.wait_for_touch().await;

        KerfAction::Touch
    }
}

/// An action that occurred, either a blink or a touch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KerfAction {
    /// Blink
    Blink,
    /// Touch
    Touch,
}
