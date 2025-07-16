//! The clock module.

use core::net::SocketAddr;

use embassy_executor::Spawner;
use embassy_net::{
    dns::DnsQueryType,
    udp::{PacketMetadata, UdpSocket},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};
use embassy_time::{Instant, Timer};
use esp_hal::{
    peripherals::{RADIO_CLK, WIFI},
    rng::Rng,
    timer::timg::Timer as TimgTimer,
};
use log::{error, info};
use sntpc::NtpContext;
use static_cell::make_static;
use time::{Date, OffsetDateTime, UtcOffset, macros::format_description};

use crate::wifi::{STOP_WIFI_SIGNAL, TimestampGenerator, WiFiStack};

/// Spawn the clock synchronization task.
pub(super) fn spawn(
    spawner: Spawner,
    timer: TimgTimer<'static>,
    wifi: WIFI<'static>,
    radio_clock: RADIO_CLK<'static>,
    clock: Clock,
    rng: Rng,
) {
    // Spawn the clock synchronization task.
    spawner.must_spawn(clock_synchronization(spawner, timer, wifi, radio_clock, clock, rng));
}

#[embassy_executor::task]
async fn clock_synchronization(
    spawner: Spawner,
    timer: TimgTimer<'static>,
    wifi: WIFI<'static>,
    radio_clock: RADIO_CLK<'static>,
    clock: Clock,
    rng: Rng,
) {
    info!("Starting clock synchronization");
    if let Ok(wifi) = WiFiStack::new(spawner, timer, wifi, radio_clock, rng).await {
        clock.sync(&wifi).await;
    }
    STOP_WIFI_SIGNAL.signal(());
    Timer::after_millis(500).await;
    STOP_WIFI_SIGNAL.signal(());
    info!("Clock synchronization completed");
}

// -------------------------------------------------------------------------------------------------

/// A clock that provides the current time and timezone.
#[derive(Clone, Copy)]
pub(super) struct Clock(&'static RwLock<CriticalSectionRawMutex, ClockInner>);

/// The inner data of a [`Clock`].
#[derive(Clone, Copy)]
struct ClockInner {
    /// The boot time in seconds since the epoch.
    boot_timestamp: u64,
    /// The offset of the current timezone.
    timezone_offset: UtcOffset,
}

impl Clock {
    /// Whether to enable daylight saving time.
    ///
    /// # Note
    /// The date range is defined in `.cargo/config.toml` as `DST_RANGE`.
    const DAYLIGHT_SAVING_TIME: bool = option_env!("DST_ENABLE").is_some();
    /// The NTP server to use for time synchronization.
    ///
    /// If none is provided `pool.ntp.org` is used by default.
    const NTP_SERVER: &str = match option_env!("NTP_SERVER") {
        Some(server) => server,
        None => "pool.ntp.org",
    };
    /// The provided timezone offset.
    ///
    /// If none is provided it is assumed to be UTC.
    const TIMEZONE_OFFSET: Option<&str> = option_env!("TIMEZONE_OFFSET");

    /// Create a new [`Clock`].
    ///
    /// # Panics
    /// Only one clock can ever exist.
    /// Calling this function twice will immediately panic.
    pub(super) fn new() -> Self {
        Self(make_static!(
            RwLock::<CriticalSectionRawMutex, ClockInner>::new(ClockInner::default())
        ))
    }

    /// Get the current time.
    #[expect(clippy::cast_possible_wrap)]
    pub(super) async fn now(&self) -> Option<OffsetDateTime> {
        let inner = *self.0.read().await;
        let epoch = Instant::now().as_secs() + inner.boot_timestamp;
        let utc = OffsetDateTime::from_unix_timestamp(epoch as i64).ok()?;
        utc.checked_to_offset(inner.timezone_offset)
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
        self.0.write().await.boot_timestamp =
            u64::from(ntp_result.sec()).saturating_sub(Instant::now().as_secs());

        // TODO: Clean this up
        if let Some(mut now) = self.now().await {
            // Adjust the timezone for daylight saving time if enabled.
            if Self::DAYLIGHT_SAVING_TIME {
                let description = format_description!("[year]-[month]-[day]");
                let mut inner = self.0.write().await;

                if let Some(start) = Date::parse(env!("DST_RANGE_START"), description)
                    .ok()
                    .and_then(|d| d.replace_year(now.year()).ok())
                    && let Some(end) = Date::parse(env!("DST_RANGE_END"), description)
                        .ok()
                        .and_then(|d| d.replace_year(now.year()).ok())
                {
                    if now.date() > start && now.date() < end {
                        let (h, m, s) = inner.timezone_offset.as_hms();
                        inner.timezone_offset = UtcOffset::from_hms(h.saturating_add(1) % 25, m, s)
                            .expect("Always within the valid range");

                        drop(inner);
                        now = self.now().await.expect("Was already proved valid");
                    }
                } else {
                    error!("Failed to parse DST dates, pretending DST is disabled");
                }
            }

            info!("Current time: {now}");
        }
    }
}

impl Default for ClockInner {
    fn default() -> Self {
        if let Some(offset) = Clock::TIMEZONE_OFFSET {
            match UtcOffset::parse(offset, format_description!("[offset_hour][offset_minute]")) {
                Ok(timezone_offset) => Self { boot_timestamp: 0, timezone_offset },
                Err(err) => {
                    error!("Failed to parse timezone offset: {err:?}");
                    Self { boot_timestamp: 0, timezone_offset: UtcOffset::UTC }
                }
            }
        } else {
            Self { boot_timestamp: 0, timezone_offset: UtcOffset::UTC }
        }
    }
}
