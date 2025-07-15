//! TODO

use core::net::SocketAddr;

use embassy_executor::Spawner;
use embassy_net::{
    Config, DhcpConfig, Runner, StackResources,
    dns::DnsQueryType,
    udp::{PacketMetadata, UdpSocket},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Instant, Timer};
use esp_hal::{
    peripherals::{RADIO_CLK, WIFI},
    rng::Rng,
    timer::timg::Timer as TimgTimer,
};
use esp_wifi::wifi::{
    AuthMethod, ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent,
    WifiState, wifi_state,
};
use log::{error, info, warn};
use sntpc::{NtpContext, NtpTimestampGenerator};
use static_cell::make_static;
use time::{Error as TimeError, OffsetDateTime, UtcOffset};

use crate::{NTP_SERVER, TIMEZONE_OFFSET};

/// A clock that provides the current time and timezone.
pub(super) struct Clock {
    /// The boot time in seconds since the epoch.
    boot_timestamp: u64,
    /// The offset of the current timezone.
    timezone_offset: UtcOffset,
}

impl Clock {
    /// Get the current time.
    #[expect(clippy::cast_possible_wrap, clippy::unwrap_used)]
    pub(super) fn now(&self) -> Result<OffsetDateTime, TimeError> {
        let epoch = Instant::now().as_secs() + self.boot_timestamp;
        let utc = OffsetDateTime::from_unix_timestamp(epoch as i64)?;
        Ok(utc.checked_to_offset(self.timezone_offset).unwrap())
    }

    // ---------------------------------------------------------------------------------------------

    /// Create a new [`Clock`].
    ///
    /// Attempts to read the boot time from the RTC Fast memory,
    /// or uses NTP if the boot time was not set.
    #[must_use]
    pub(super) async fn new(
        spawner: Spawner,
        timer: TimgTimer<'static>,
        wifi: WIFI<'static>,
        radio_clock: RADIO_CLK<'static>,
        mut rng: Rng,
    ) -> Self {
        // Initialize the WiFi radio.
        let controller = match esp_wifi::init(timer, rng, radio_clock) {
            Ok(controller) => make_static!(controller),
            Err(err) => {
                error!("Failed to initialize WiFi radio: {err:?}");
                return Self { boot_timestamp: 0, timezone_offset: TIMEZONE_OFFSET };
            }
        };

        // Initialize the WiFi controller and interfaces.
        let (controller, interfaces) = match esp_wifi::wifi::new(controller, wifi) {
            Ok((controller, interfaces)) => (controller, interfaces),
            Err(err) => {
                error!("Failed to initialize WiFi controller: {err:?}");
                return Self { boot_timestamp: 0, timezone_offset: TIMEZONE_OFFSET };
            }
        };

        let resources: &'static mut StackResources<4> = make_static!(StackResources::<4>::new());
        let config = Config::dhcpv4(DhcpConfig::default());

        // Concatenate two random u32 values to create a u64 seed.
        let (seed_a, seed_b) = (rng.random().to_ne_bytes(), rng.random().to_ne_bytes());
        let seed = u64::from_ne_bytes([
            seed_a[0], seed_a[1], seed_a[2], seed_a[3], seed_b[0], seed_b[1], seed_b[2], seed_b[3],
        ]);

        // Spawn the network and connection tasks.
        let (stack, runner) = embassy_net::new::<_, 4>(interfaces.sta, config, resources, seed);
        spawner.must_spawn(network_task(runner));
        spawner.must_spawn(connection_task(controller));
        stack.wait_config_up().await;

        if let Some(config) = stack.config_v4() {
            info!(
                "Connected as `{}` with gateway `{:?}`",
                config.address.address(),
                config.gateway
            );
        } else {
            warn!("Assumed to have a valid address, but none found?");
        }

        // Create UDP socket for NTP requests.
        let mut rx_meta = [PacketMetadata::EMPTY; 16];
        let mut rx_buffer = [0; 4096];
        let mut tx_meta = [PacketMetadata::EMPTY; 16];
        let mut tx_buffer = [0; 4096];

        let mut socket =
            UdpSocket::new(stack, &mut rx_meta, &mut rx_buffer, &mut tx_meta, &mut tx_buffer);
        if let Err(err) = socket.bind(123) {
            error!("Failed to bind UDP socket for NTP: {err:?}");
            return Self { boot_timestamp: 0, timezone_offset: TIMEZONE_OFFSET };
        }

        info!("Resolving NTP server address");
        let context = NtpContext::new(TimestampGenerator);
        let addrs = match stack.dns_query(NTP_SERVER, DnsQueryType::A).await {
            Err(err) => {
                error!("Failed to resolve NTP server: {err:?}");
                return Self { boot_timestamp: 0, timezone_offset: TIMEZONE_OFFSET };
            }
            Ok(addrs) if addrs.is_empty() => {
                error!("No addresses found for NTP server");
                return Self { boot_timestamp: 0, timezone_offset: TIMEZONE_OFFSET };
            }
            Ok(addrs) => addrs,
        };

        info!("Fetching time from NTP server");

        let mut counter = 0u8;
        loop {
            match sntpc::get_time(SocketAddr::new(addrs[0].into(), 123), &socket, context).await {
                Ok(time) => {
                    STOP_WIFI_SIGNAL.signal(());
                    let clock = Self {
                        boot_timestamp: u64::from(time.sec()),
                        timezone_offset: TIMEZONE_OFFSET,
                    };

                    if let Ok(now) = clock.now() {
                        info!("Current time: {now}");
                    }

                    return clock;
                }
                Err(err) => {
                    error!("Failed to request current time: {err:?}");
                    counter += 1;

                    if counter >= 5 {
                        error!("Giving up on NTP after 5 attempts");
                        return Self { boot_timestamp: 0, timezone_offset: TIMEZONE_OFFSET };
                    }
                }
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

#[embassy_executor::task]
async fn network_task(mut runner: Runner<'static, WifiDevice<'static>>) -> ! {
    info!("Starting network background task");
    runner.run().await
}

/// The WiFi SSID to connect to.
const WIFI_SSID: &str = env!("WIFI_SSID");
/// The WiFi password, if any.
const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");

/// A [`Signal`] to stop the WiFi connection task.
static STOP_WIFI_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

#[embassy_executor::task]
async fn connection_task(mut controller: WifiController<'static>) {
    info!("Starting WiFi background task");
    let _ = controller.disconnect_async().await;

    loop {
        // If we're already connected, wait until we disconnect.
        if matches!(wifi_state(), WifiState::StaConnected) {
            controller.wait_for_event(WifiEvent::StaDisconnected).await;
            Timer::after_secs(3).await;
        }

        // If the controller hasn't been started, configure and start it.
        if !matches!(controller.is_started(), Ok(true)) {
            let config = ClientConfiguration {
                ssid: WIFI_SSID.into(),
                password: WIFI_PASSWORD.into(),
                auth_method: AuthMethod::WPA2WPA3Personal,
                ..Default::default()
            };

            if let Err(err) = controller.set_configuration(&Configuration::Client(config)) {
                error!("Failed to configure WiFi controller: {err:?}");
                continue;
            }
            if let Err(err) = controller.start_async().await {
                error!("Failed to start WiFi controller: {err:?}");
                continue;
            }
        }

        // Attempt to connect to the WiFi network.
        match controller.connect_async().await {
            Ok(()) => {
                info!("Connected to WiFi network");
                STOP_WIFI_SIGNAL.wait().await;
                info!("Disconnecting from WiFi network");
                return;
            }
            Err(err) => {
                error!("Failed to connect to WiFi network: {err:?}");
                Timer::after_secs(3).await;
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// A simple [`NtpTimestampGenerator`] implementation
/// that uses [`Instant`] to generate timestamps.
#[derive(Clone, Copy)]
struct TimestampGenerator;

impl NtpTimestampGenerator for TimestampGenerator {
    fn init(&mut self) {}

    fn timestamp_sec(&self) -> u64 { Instant::now().as_secs() }

    fn timestamp_subsec_micros(&self) -> u32 {
        let instant = Instant::now();
        let fraction = instant.as_micros().saturating_sub(instant.as_secs() * 1_000_000);
        u32::try_from(fraction).unwrap_or_default()
    }
}
