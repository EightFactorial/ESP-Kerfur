//! TODO

use core::{
    fmt::Write,
    num::IntErrorKind,
    ops::Deref,
    str::FromStr,
    sync::atomic::{AtomicU32, Ordering},
};

use chrono::{DateTime, Datelike, Timelike, Utc, Weekday, WeekdaySet};
use chrono_tz::Tz;
use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, lazy_lock::LazyLock, mutex::Mutex};
use embassy_time::Instant;
use embedded_graphics::{
    mono_font::{MonoTextStyle, MonoTextStyleBuilder, ascii::FONT_10X20},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use esp_hal::i2c::master::Error as I2cError;
use log::{debug, error, info, warn};
use sh1106::{Error as DisplayError, interface::DisplayInterface, mode::GraphicsMode};

use crate::network::{KerfWifi, wifi::STOP_CONNECTION_SIGNAL};

/// A clock implementation.
///
/// Requires synchronization using NTP on each boot.
#[derive(Clone, Copy)]
pub struct KerfClock(&'static Mutex<NoopRawMutex, KerfClockInner>);

impl KerfClock {
    /// Create a new [`KerfClock`].
    #[must_use]
    pub fn new() -> Self { Self(KerfClockInner::get()) }

    /// Get the current time.
    #[must_use]
    #[expect(clippy::cast_possible_wrap, reason = "Wrapping will happen *far* into the future")]
    pub async fn now(&self) -> DateTime<Tz> {
        let inner = self.lock().await;
        // Calculate the time by adding the boot timestamp to the time since boot.
        let epoch = Instant::now().as_secs().wrapping_add(inner.boot_timestamp) as i64;
        // Create a UTC datetime from the epoch seconds.
        let utc = DateTime::<Utc>::from_timestamp(epoch.max(0), 0)
            .unwrap_or_else(|| unreachable!("It is impossible for the timestamp to be invalid"));
        // Add the timezone.
        utc.with_timezone(&inner.timezone)
    }

    /// Returns `true` if Kerfur is in "silent mode".
    #[must_use]
    pub async fn in_silent_mode(&self) -> bool {
        const SILENT_MODE_START: u32 = match u32::from_str_radix(env!("SILENT_MODE_START"), 10) {
            Ok(hour) if hour < 24 => hour,
            Ok(..) => panic!("SILENT_MODE_START must be between 0 and 23"),
            Err(..) => 0,
        };
        const SILENT_MODE_END: u32 = match u32::from_str_radix(env!("SILENT_MODE_END"), 10) {
            Ok(hour) if hour < 24 => hour,
            Ok(..) => panic!("SILENT_MODE_END must be between 0 and 23"),
            Err(..) => 0,
        };

        let now = self.now().await;

        now.hour() >= SILENT_MODE_START || now.hour() < SILENT_MODE_END
    }

    /// Returns `true` if the alarm is currently tripped.
    #[must_use]
    pub async fn is_alarm_tripped(&self) -> bool {
        /// Whether the alarm is enabled.
        const ALARM_ENABLE: bool = env!("ALARM_ENABLE").eq_ignore_ascii_case("true");

        /// The hour (0-23) the alarm should trip.
        const ALARM_HOUR: u32 = match u32::from_str_radix(env!("ALARM_HOUR"), 10) {
            Ok(hour) if hour < 24 => hour,
            Ok(..) => panic!("ALARM_HOUR must be between 0 and 23"),
            Err(..) => 0,
        };
        /// The minute (0-59) the alarm should trip.
        const ALARM_MINUTE: u32 = match u32::from_str_radix(env!("ALARM_MINUTE"), 10) {
            Ok(minute) if minute < 60 => minute,
            Ok(..) => panic!("ALARM_MINUTE must be between 0 and 59"),
            Err(..) => 0,
        };

        /// The weekdays the alarm should trip on.
        static ALARM_WEEKDAYS: LazyLock<WeekdaySet> = LazyLock::new(|| {
            let mut set = WeekdaySet::EMPTY;
            for day in env!("ALARM_WEEKDAYS").split(',') {
                match Weekday::from_str(day) {
                    Ok(weekday) => {
                        set.insert(weekday);
                    }
                    Err(..) => warn!("Invalid weekday: {day}"),
                }
            }

            debug!("Alarm enabled during weekdays: {set}");
            if ALARM_ENABLE && set.is_empty() {
                warn!("Alarm is enabled but no weekdays are set and will never trip!");
            }

            set
        });

        if !ALARM_ENABLE {
            return false;
        }

        let datetime = self.now().await;
        let weekdays = ALARM_WEEKDAYS.get();

        if (datetime.hour() == ALARM_HOUR && datetime.minute() == ALARM_MINUTE)
            && weekdays.contains(datetime.weekday())
        {
            /// Store the last `Instant` the alarm was tripped.
            static LAST_TRIPPED: AtomicU32 = AtomicU32::new(u32::MAX);

            #[expect(clippy::cast_possible_truncation, reason = "Only the difference matters here")]
            let instant = Instant::now().as_secs() as u32;

            // If the alarm was tripped in the last 60 seconds, do not trip it again.
            let tripped = LAST_TRIPPED.load(Ordering::Relaxed).abs_diff(instant) > 60;
            // Update the last tripped time.
            LAST_TRIPPED.store(instant, Ordering::Relaxed);

            tripped
        } else {
            false
        }
    }

    /// Draw the current time to the provided display.
    pub async fn draw<T: DisplayInterface<Error = DisplayError<I2cError, ()>>>(
        self,
        display: &mut GraphicsMode<T>,
    ) -> Result<(), I2cError> {
        static FONT: MonoTextStyle<'static, BinaryColor> =
            MonoTextStyleBuilder::new().font(&FONT_10X20).text_color(BinaryColor::On).build();

        let now = self.now().await;
        let mut buffer = NoAllocBuf(0, [0u8; 32]);

        // Clear the display
        display.clear();

        // Draw the time
        if let Ok(()) = write!(buffer, "{}", now.format("%-I:%M %p")) {
            Text::with_baseline(buffer.as_str(), Point::new(40, 20), FONT, Baseline::Top)
                .draw(display)
                .unwrap();
        } else {
            error!("Failed to write time to display!");
        }

        // Draw the date
        if let Ok(()) = write!(buffer, "{}", now.format("%m/%d/%Y")) {
            Text::with_baseline(buffer.as_str(), Point::new(10, 35), FONT, Baseline::Top)
                .draw(display)
                .unwrap();
        } else {
            error!("Failed to write date to display!");
        }

        // Flush the display
        match display.flush() {
            Ok(()) => Ok(()),
            Err(DisplayError::Comm(err)) => Err(err),
            Err(DisplayError::Pin(())) => unreachable!(),
        }
    }

    /// Synchronize the clock using NTP.
    pub fn synchronize(self, wifi: KerfWifi, spawner: Spawner) {
        spawner.must_spawn(synchronization_task(self, wifi));
    }
}

impl Default for KerfClock {
    fn default() -> Self { Self::new() }
}

impl Deref for KerfClock {
    type Target = Mutex<NoopRawMutex, KerfClockInner>;

    fn deref(&self) -> &Self::Target { self.0 }
}

/// A `no_alloc` buffer for writing temporary strings.
struct NoAllocBuf<const N: usize>(usize, [u8; N]);

impl<const N: usize> NoAllocBuf<N> {
    /// Get the string slice written to the buffer.
    fn as_str(&self) -> &str {
        str::from_utf8(&self.1[..self.0]).expect("NoAllocBuf contains invalid UTF-8")
    }
}

impl<const N: usize> Write for NoAllocBuf<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.0 = s.len().min(N);
        self.1[..self.0].copy_from_slice(&s.as_bytes()[..self.0]);
        Ok(())
    }
}

// -------------------------------------------------------------------------------------------------

/// The shared inner data of all [`KerfClock`] instances.
pub struct KerfClockInner {
    /// The boot time in seconds since the Unix epoch.
    boot_timestamp: u64,
    /// The timezone to use for local time.
    timezone: Tz,
}

impl KerfClockInner {
    /// The boot time in seconds since the epoch.
    ///
    /// Useful for testing to skip the NTP synchronization step.
    const BOOT_TIME: u64 = match u64::from_str_radix(env!("BOOT_TIME"), 10) {
        Ok(time) => time,
        Err(err) if matches!(err.kind(), IntErrorKind::Empty | IntErrorKind::Zero) => 0,
        Err(..) => panic!("Invalid `BOOT_TIME` environment variable, must be empty or a valid u64"),
    };
    /// The provided timezone.
    ///
    /// If none is provided it is assumed to be UTC.
    const TIMEZONE: Option<&str> = option_env!("TIMEZONE");

    /// Get a reference to the single instance of a [`KerfClockInner`].
    #[must_use]
    fn get() -> &'static Mutex<NoopRawMutex, KerfClockInner> {
        static INSTANCE: LazyLock<Mutex<NoopRawMutex, KerfClockInner>> = LazyLock::new(|| {
            Mutex::new(if let Some(offset) = KerfClockInner::TIMEZONE {
                // If provided a timezone, attempt to parse it.
                match offset.parse::<Tz>() {
                    Ok(timezone) => {
                        info!("Using timezone: {timezone}");
                        KerfClockInner { boot_timestamp: KerfClockInner::BOOT_TIME, timezone }
                    }
                    Err(err) => {
                        error!("Failed to parse timezone: {err}, using UTC instead");
                        KerfClockInner {
                            boot_timestamp: KerfClockInner::BOOT_TIME,
                            timezone: Tz::UTC,
                        }
                    }
                }
            } else {
                // Otherwise default to UTC.
                KerfClockInner { boot_timestamp: KerfClockInner::BOOT_TIME, timezone: Tz::UTC }
            })
        });

        INSTANCE.get()
    }
}

// -------------------------------------------------------------------------------------------------

#[embassy_executor::task]
async fn synchronization_task(clock: KerfClock, wifi: KerfWifi) {
    wifi.stack().wait_config_up().await;
    info!("Starting clock synchronization");

    match wifi.ntp().await {
        None => error!("Failed to get NTP time, clock synchronization failed!"),
        Some(ntp) => {
            let now = Instant::now();

            // Set the boot timestamp: boot_timestamp = current_ntp_time - time_since_boot
            clock.lock().await.boot_timestamp = u64::from(ntp.sec()).saturating_sub(now.as_secs());
            info!("Clock synchronization completed");
        }
    }

    info!("Current time: {}", clock.now().await);
    STOP_CONNECTION_SIGNAL.signal(());
}
