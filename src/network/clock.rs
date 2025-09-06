//! TODO

use core::{num::IntErrorKind, ops::Deref};

use chrono::{DateTime, Timelike, Utc};
use chrono_tz::Tz;
use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, lazy_lock::LazyLock, mutex::Mutex};
use embassy_time::Instant;
use log::{error, info};

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

    /// Returns `true` if Kerfus is in "silent mode".
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
                        KerfClockInner { boot_timestamp: KerfClockInner::BOOT_TIME, timezone }
                    }
                    Err(err) => {
                        error!("Failed to parse timezone: {err}");
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

    /// Set the application's boot timestamp.
    pub fn set_timestamp(&mut self, timestamp: u64) { self.boot_timestamp = timestamp; }

    /// Set the timezone to use for local time.
    pub fn set_timezone(&mut self, timezone: Tz) { self.timezone = timezone; }
}

// -------------------------------------------------------------------------------------------------

#[embassy_executor::task]
async fn synchronization_task(clock: KerfClock, wifi: KerfWifi) {
    wifi.stack().wait_config_up().await;
    info!("Starting clock synchronization");

    info!("Clock synchronization completed");
    info!("Current time: {}", clock.now().await);
    STOP_CONNECTION_SIGNAL.signal(());
}
