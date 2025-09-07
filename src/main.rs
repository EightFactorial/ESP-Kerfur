//! TODO
#![feature(never_type, type_alias_impl_trait)]
#![no_main]
#![no_std]

use embassy_executor::Spawner;
use embassy_time::Timer;
use esp_hal::{i2c::master::Error as I2cError, rng::Rng, timer::timg::TimerGroup};
use futures_lite::future;
use log::{error, info, warn};

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
    if let Err(err) = display.init().and_then(|()| KerfEmote::Confused.draw(&mut display)) {
        error!("Failed to initialize display and draw: {err}");
    }

    // Create an RNG source and clock.
    let rand = Rng::new(per.RNG);
    let clock = KerfClock::new();

    // Connect to WiFi and synchronize the clock,
    // if the time was not provided at compile time.
    if env!("BOOT_TIME").is_empty() {
        let timer = TimerGroup::new(per.TIMG1);
        if let Ok(wifi) = KerfWifi::new(per.WIFI, timer.timer0, rand, spawner).await {
            clock.synchronize(wifi, spawner);
        }
    } else {
        info!("Skipping clock synchronization, using compile-time timestamp");
    }

    // Create audio output and touch sensor.
    let audio = KerfAudio::new(per.GPIO21);
    let touch = KerfTouch::new(per.GPIO5);

    // And finally start the display manager.
    Kerfur::new(audio, clock, display, touch, rand).execute(spawner).await
}

// -------------------------------------------------------------------------------------------------

/// Kerfur's brain.
///
/// Manages all display, audio, time, and touch logic.
pub struct Kerfur {
    /// The audio output to use.
    audio: KerfAudio,
    /// The clock used for timekeeping.
    clock: KerfClock,
    /// The display to draw emotes to.
    display: KerfDisplay,
    /// The touch sensor used to detect touches.
    touch: KerfTouch,

    /// A source of randomness.
    rand: Rng,
}

/// An action that should occur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KerfAction {
    /// Blink
    Blink,
    /// Touch
    Touch,
    /// Clock
    Clock,
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
    pub async fn execute(mut self, spawner: Spawner) -> ! {
        info!("Starting Display Manager");

        loop {
            let Err(err) = self.execute_inner(&spawner).await;
            error!("Display Manager encountered an error: {err:?}");

            Timer::after_secs(10).await;
            warn!("Restarting Display Manager...");
        }
    }

    // ----------------------------------------------------------------------------------------------

    /// The main execution loop, handling all display updates and interactions.
    async fn execute_inner(&mut self, _spawner: &Spawner) -> Result<!, KerfError> {
        loop {
            // Reset to the neutral emote.
            KerfEmote::Neutral.draw(&mut self.display)?;

            // If the alarm is tripped, meow until touched.
            if self.clock.is_alarm_tripped().await {
                info!("Alarm tripped at {}!", self.clock.now().await);

                let mut reset = false;
                while !reset {
                    self.audio.meow().await;
                    Timer::after_millis(175).await;
                    reset |= self.touch.is_touched();

                    KerfEmote::Meow.draw(&mut self.display)?;
                    Timer::after_millis(650).await;
                    reset |= self.touch.is_touched();

                    KerfEmote::Neutral.draw(&mut self.display)?;
                    Timer::after_millis(175).await;
                    reset |= self.touch.is_touched();
                }
            }

            // Wait for either a blink or a touch action.
            match future::or::<KerfAction, _, _>(
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
                // Wait for the touch to be released, then meow and draw the `Meow` emote.
                KerfAction::Touch => {
                    self.touch.wait_for_release().await;

                    // Skip meowing if in silent mode.
                    if !self.clock.in_silent_mode().await {
                        self.audio.meow().await;
                        Timer::after_millis(175).await;
                    }

                    KerfEmote::Meow.draw(&mut self.display)?;
                    Timer::after_millis(725).await;
                }
                // Draw the current time on the display.
                KerfAction::Clock => {
                    self.clock.draw(&mut self.display).await?;
                    self.touch.wait_for_release().await;
                    Timer::after_secs(5).await;
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

        future::or::<KerfAction, _, _>(
            // If the touch is released, return `Touch`.
            async {
                touch.wait_for_release().await;
                KerfAction::Touch
            },
            // If the touch is held for 3 seconds, return `Clock`.
            async {
                Timer::after_secs(3).await;
                KerfAction::Clock
            },
        )
        .await
    }
}

// -------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub enum KerfError {
    I2c(I2cError),
}

impl From<I2cError> for KerfError {
    fn from(err: I2cError) -> Self { Self::I2c(err) }
}
