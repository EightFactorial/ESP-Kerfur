//! The clock module.

use core::{net::SocketAddr, num::IntErrorKind};

use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use embassy_executor::Spawner;
use embassy_net::{
    dns::DnsQueryType,
    udp::{PacketMetadata, UdpSocket},
};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::Instant;
use esp_hal::{peripherals::WIFI, rng::Rng, timer::timg::Timer as TimgTimer};
use log::{error, info};
use sntpc::NtpContext;
use static_cell::make_static;

use crate::wifi::{STOP_WIFI_SIGNAL, TimestampGenerator, WiFiStack};

/// Spawn the clock synchronization task.
pub(super) fn spawn(
    spawner: Spawner,
    timer: TimgTimer<'static>,
    wifi: WIFI<'static>,
    clock: Clock,
    rng: Rng,
) {
    // Spawn the clock synchronization task.
    spawner.must_spawn(clock_synchronization(spawner, timer, wifi, clock, rng));
}

#[embassy_executor::task]
async fn clock_synchronization(
    spawner: Spawner,
    timer: TimgTimer<'static>,
    wifi: WIFI<'static>,
    clock: Clock,
    rng: Rng,
) {
    info!("Starting clock synchronization");
    if let Ok(wifi) = WiFiStack::new(spawner, timer, wifi, rng).await {
        clock.sync(&wifi).await;
    }
    info!("Clock synchronization completed");
    STOP_WIFI_SIGNAL.signal(());
}

// -------------------------------------------------------------------------------------------------

/// A clock that provides the current time and timezone.
#[derive(Clone, Copy)]
pub(super) struct Clock(&'static Mutex<NoopRawMutex, ClockInner>);

/// The inner data of a [`Clock`].
#[derive(Clone, Copy)]
struct ClockInner {
    /// The boot time in seconds since the epoch.
    boot_timestamp: u64,
    /// The current timezone.
    timezone: Tz,
}

impl Clock {
    /// The boot time in seconds since the epoch.
    ///
    /// Useful for testing to skip the NTP synchronization step.
    const BOOT_TIME: u64 = match u64::from_str_radix(env!("BOOT_TIME"), 10) {
        Ok(time) => time,
        Err(err) if matches!(err.kind(), IntErrorKind::Empty | IntErrorKind::Zero) => 0,
        Err(..) => panic!("Invalid `BOOT_TIME` environment variable, must be empty or a valid u64"),
    };
    /// The NTP server to use for time synchronization.
    ///
    /// If none is provided `pool.ntp.org` is used by default.
    const NTP_SERVER: &str = env!("NTP_SERVER");
    /// The provided timezone.
    ///
    /// If none is provided it is assumed to be UTC.
    const TIMEZONE: Option<&str> = option_env!("TIMEZONE");

    /// Create a new [`Clock`].
    ///
    /// # Panics
    /// Only one clock can ever exist.
    /// Calling this function twice will immediately panic.
    pub(super) fn new() -> Self { Self(make_static!(Mutex::new(ClockInner::default()))) }

    /// Get the current time.
    #[expect(clippy::cast_possible_wrap)]
    pub(super) async fn now(&self) -> DateTime<Tz> {
        let inner = *self.0.lock().await;
        let epoch = Instant::now().as_secs().wrapping_add(inner.boot_timestamp) as i64;
        let utc = DateTime::<Utc>::from_timestamp(epoch.max(0), 0)
            .expect("It is impossible for epoch to be invalid");
        utc.with_timezone(&inner.timezone)
    }

    // ---------------------------------------------------------------------------------------------
    //

    /// Create a new [`Clock`].
    ///
    /// Attempts to retrieve the current time from an NTP server.
    pub(super) async fn sync(&self, wifi: &WiFiStack) {
        // Create UDP socket for NTP requests.
        let mut rx_meta = [PacketMetadata::EMPTY; 24];
        let mut rx_buffer = [0; 6144];
        let mut tx_meta = [PacketMetadata::EMPTY; 24];
        let mut tx_buffer = [0; 6144];

        let mut socket = UdpSocket::new(
            wifi.stack(),
            &mut rx_meta,
            &mut rx_buffer,
            &mut tx_meta,
            &mut tx_buffer,
        );
        if let Err(err) = socket.bind(123) {
            error!("Failed to bind UDP socket for NTP: {err:?}");
            return;
        }

        info!("Resolving NTP server address: \"{}\"", Self::NTP_SERVER);

        let context = NtpContext::new(TimestampGenerator);
        let addrs = match wifi.stack().dns_query(Self::NTP_SERVER, DnsQueryType::A).await {
            Err(err) => {
                error!("Failed to resolve NTP server: {err:?}");
                return;
            }
            Ok(addrs) if addrs.is_empty() => {
                error!("No addresses found for NTP server");
                return;
            }
            Ok(addrs) => addrs,
        };

        info!("Fetching time from NTP server");

        // Set the boot timestamp
        let ntp_addr = SocketAddr::new(addrs[0].into(), 123);
        let ntp_result = match sntpc::get_time(ntp_addr, &socket, context).await {
            Ok(result) => result,
            Err(err) => {
                error!("Failed to get time from NTP server: {err:?}");
                return;
            }
        };
        self.0.lock().await.boot_timestamp =
            u64::from(ntp_result.sec()).saturating_sub(Instant::now().as_secs());

        info!("Current time: {}", self.now().await);
    }
}

impl Default for ClockInner {
    fn default() -> Self {
        if let Some(offset) = Clock::TIMEZONE {
            match offset.parse::<Tz>() {
                Ok(timezone) => Self { boot_timestamp: Clock::BOOT_TIME, timezone },
                Err(err) => {
                    error!("Failed to parse timezone: {err:?}");
                    Self { boot_timestamp: Clock::BOOT_TIME, timezone: Tz::UTC }
                }
            }
        } else {
            Self { boot_timestamp: Clock::BOOT_TIME, timezone: Tz::UTC }
        }
    }
}
