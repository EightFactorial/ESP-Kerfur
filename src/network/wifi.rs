//! TODO

use core::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

use embassy_executor::Spawner;
use embassy_net::{
    Config as WifiConfig, DhcpConfig, IpAddress, Runner, Stack, StackResources,
    dns::DnsQueryType,
    udp::{PacketMetadata, UdpSocket},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Instant, Timer, with_timeout};
use esp_hal::{peripherals::WIFI, rng::Rng};
use esp_wifi::{
    EspWifiTimerSource,
    config::PowerSaveMode,
    wifi::{
        AuthMethod, ClientConfiguration, Configuration, WifiController, WifiDevice, WifiError,
        WifiEvent, WifiState, wifi_state,
    },
};
use heapless::Vec;
use log::{debug, error, info, warn};
use sntpc::NtpContext;
use static_cell::make_static;

/// TODO
pub struct KerfWifi {
    /// A handle to the network stack.
    stack: Stack<'static>,
}

impl KerfWifi {
    /// Access the inner [`embassy_net::Stack`].
    #[must_use]
    pub const fn stack(&self) -> Stack<'static> { self.stack }

    /// Attempt to create a new [`KerfWifi`].
    pub async fn new<T: EspWifiTimerSource + 'static>(
        wifi: WIFI<'static>,
        timer: T,
        mut rand: Rng,
        spawner: Spawner,
    ) -> Result<Self, WifiError> {
        // Initialize the WiFi controller and interfaces.
        let esp_controller =
            make_static!(esp_wifi::init(timer, rand).expect("Failed to initialize `esp_wifi`!"));
        let (mut controller, interfaces) = match esp_wifi::wifi::new(esp_controller, wifi) {
            Ok((controller, interfaces)) => (controller, interfaces),
            Err(err) => {
                error!("Failed to initialize WiFi interface: {err:?}");
                return Err(err);
            }
        };

        // Attempt to set the `PowerSaveMode` to `Minimum`.
        if let Err(err) = controller.set_power_saving(PowerSaveMode::Minimum) {
            warn!("Failed to set WiFi `PowerSaveMode`: {err:?}");
        }

        // Concatenate two random u32 values to create a u64 seed.
        let (seed_a, seed_b) = (rand.random().to_ne_bytes(), rand.random().to_ne_bytes());
        let random_seed = u64::from_ne_bytes([
            seed_a[0], seed_a[1], seed_a[2], seed_a[3], seed_b[0], seed_b[1], seed_b[2], seed_b[3],
        ]);

        // Create a DHCP config and memory for a network stack.
        let config = WifiConfig::dhcpv4(DhcpConfig::default());
        let resources: &'static mut StackResources<4> = make_static!(StackResources::new());

        // Create the stack and spawn the network and connection tasks.
        let (stack, runner) = embassy_net::new(interfaces.sta, config, resources, random_seed);
        spawner.must_spawn(network_task(runner));
        spawner.must_spawn(connection_task(controller));

        // Wait for the network to come up.
        stack.wait_link_up().await;
        info!("Waiting for DHCP configuration...");
        stack.wait_config_up().await;

        // Log the assigned network address.
        if let Some(config) = stack.config_v4() {
            info!("Assigned network address: {}", config.address);
        } else {
            warn!("Assumed to have an address, but none found?");
        }

        // Return the initialized WiFi stack.
        Ok(Self { stack })
    }

    /// Resolve a hostname to an IPv4 address using DNS.
    ///
    /// Returns `None` if the resolution fails.
    pub async fn resolve(&self, addr: &str) -> Option<IpAddress> {
        let mut attempt = 0u32;

        while attempt < 3 {
            info!("Resolving hostname: \"{addr}\"");

            match with_timeout(Duration::from_secs(10), self.stack.dns_query(addr, DnsQueryType::A))
                .await
            {
                Ok(Ok(addrs)) => return Some(addrs[0]),
                Ok(Err(err)) => {
                    error!("DNS query failed: {err:?}, retrying...");
                    attempt += 1;
                }
                Err(..) => warn!("DNS query timed out, retrying..."),
            }

            Timer::after_secs(15).await;
        }

        None
    }

    /// Perform an NTP query and return the result.
    ///
    /// Returns `None` if the query fails.
    pub async fn ntp(&self) -> Option<sntpc::NtpResult> {
        /// The NTP server to use for time synchronization.
        ///
        /// If none is provided `pool.ntp.org` is used by default.
        const NTP_SERVER: &str = env!("NTP_SERVER");

        // Create UDP socket for NTP requests.
        let mut rx_meta = [PacketMetadata::EMPTY; 24];
        let mut rx_buffer = [0; 6144];
        let mut tx_meta = [PacketMetadata::EMPTY; 24];
        let mut tx_buffer = [0; 6144];

        let mut socket = UdpSocket::new(
            self.stack(),
            &mut rx_meta,
            &mut rx_buffer,
            &mut tx_meta,
            &mut tx_buffer,
        );

        // Bind to the NTP port.
        socket.bind(123).expect("Failed to bind to NTP port!");

        let mut addrs = Vec::<IpAddress, 4>::new();
        let mut port = 123;

        if let Ok(SocketAddr::V4(addr)) = SocketAddr::from_str(NTP_SERVER) {
            // If `NTP_SERVER` is an IPv4 socket address, use it directly.
            let _ = addrs.push(IpAddress::Ipv4(*addr.ip()));
            port = addr.port();
        } else if let Ok(IpAddr::V4(addr)) = IpAddr::from_str(NTP_SERVER) {
            // If `NTP_SERVER` is an IPv4 address, use it directly.
            let _ = addrs.push(IpAddress::from(addr));
        } else {
            // Otherwise, use DNS to attempt to resolve the hostname.
            let mut server = NTP_SERVER;

            loop {
                if let Some(addr) = self.resolve(server).await {
                    let _ = addrs.push(addr);
                    break;
                }

                error!("Failed to resolve \"{server}\"");

                if server == "pool.ntp.org" {
                    return None;
                }

                warn!("Attempting again using \"pool.ntp.org\" instead");
                server = "pool.ntp.org";
            }
        }

        let context = NtpContext::new(NtpTimestamp);
        let addr = SocketAddr::new(addrs[0].into(), port);

        loop {
            info!("Requesting time from NTP server");

            match with_timeout(Duration::from_secs(10), sntpc::get_time(addr, &socket, context))
                .await
            {
                Ok(Ok(time)) => return Some(time),
                Ok(Err(err)) => error!("NTP query failed: {err:?}, retrying..."),
                Err(..) => warn!("NTP query timed out, retrying..."),
            }

            Timer::after_secs(15).await;
        }
    }
}

/// An implementation of [`sntpc::NtpTimestampGenerator`]
/// using [`embassy_time::Instant`].
#[derive(Debug, Clone, Copy)]
struct NtpTimestamp;

impl sntpc::NtpTimestampGenerator for NtpTimestamp {
    fn init(&mut self) {}

    fn timestamp_sec(&self) -> u64 { Instant::now().as_secs() }

    fn timestamp_subsec_micros(&self) -> u32 { (Instant::now().as_micros() % 1_000_000) as u32 }
}

// -------------------------------------------------------------------------------------------------

/// A [`Signal`] to stop the [`network_task`].
static STOP_NETWORK_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

#[embassy_executor::task]
async fn network_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    info!("Starting network background task");
    futures_lite::future::or(async { runner.run().await }, STOP_NETWORK_SIGNAL.wait()).await;
    info!("Stopping network background task");
}

// -------------------------------------------------------------------------------------------------

/// A [`Signal`] to stop the [`connection_task`].
pub static STOP_CONNECTION_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

#[embassy_executor::task]
async fn connection_task(mut controller: WifiController<'static>) {
    /// The WiFi authentication method to use.
    ///
    /// # WARNING
    ///
    /// THIS MUST MATCH THE AUTHENTICATION METHOD *EXACTLY*!
    ///
    /// DO NOT USE `WPA2Personal` IF YOUR NETWORK USES `WPAWPA2Personal`!
    const WIFI_AUTH_METHOD: AuthMethod = match env!("WIFI_AUTH_METHOD").as_bytes() {
        b"None" => AuthMethod::None,
        b"WEP" => AuthMethod::WEP,
        b"WPA" => AuthMethod::WPA,
        b"WPA2Personal" => AuthMethod::WPA2Personal,
        b"WPAWPA2Personal" => AuthMethod::WPAWPA2Personal,
        b"WPA2Enterprise" => AuthMethod::WPA2Enterprise,
        b"WPA3Personal" => AuthMethod::WPA3Personal,
        b"WPA2WPA3Personal" => AuthMethod::WPA2WPA3Personal,
        b"WAPIPersonal" => AuthMethod::WAPIPersonal,
        _ => panic!("Unknown WiFi authentication method"),
    };
    /// The WiFi password, if any.
    const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");
    /// The WiFi SSID to connect to.
    const WIFI_SSID: &str = env!("WIFI_SSID");

    // Enable verbose logging if `ESP_LOG` is set to `trace`.
    #[cfg(feature = "logging")]
    if const { env!("ESP_LOG").eq_ignore_ascii_case("trace") } {
        esp_wifi::wifi_set_log_verbose();
    }

    let mut found_network = true;
    let mut warn_auth_method = false;

    let mut attempts = 0u8;
    let max_attempts = 8u8;

    info!("Starting WiFi background task");
    let _ = controller.disconnect_async().await;

    loop {
        // If we're already connected, wait until we disconnect.
        if matches!(wifi_state(), WifiState::StaConnected) {
            controller.wait_for_event(WifiEvent::StaDisconnected).await;
            Timer::after_secs(10).await;
        }

        // If the controller hasn't been started, configure and start it.
        if !matches!(controller.is_started(), Ok(true)) {
            let config = ClientConfiguration {
                ssid: WIFI_SSID.into(),
                password: WIFI_PASSWORD.into(),
                auth_method: WIFI_AUTH_METHOD,
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

        // Scan until we find the target WiFi network.
        while !found_network {
            match controller.scan_n_async(8).await {
                Ok(res) => {
                    debug!("Network Scan:");
                    if res.iter().any(|net| {
                        // Debug log all found networks.
                        debug!(
                            "    SSID: \"{}\", BSSID: {:?}, RSSI: {}, AUTH: {:?}",
                            net.ssid, net.bssid, net.signal_strength, net.auth_method
                        );
                        // If the SSID and AuthMethod matches, return `true`.
                        if net.auth_method == Some(WIFI_AUTH_METHOD) {
                            return net.ssid == WIFI_SSID;
                        } else if warn_auth_method {
                            // Warn once if the SSID matches but the authentication method does not.
                            warn!("Found target WiFi network, but authentication method does not match!");
                            warn_auth_method = true;
                        }

                        false
                    }) {
                        info!("Found target WiFi network: \"{WIFI_SSID}\"");
                        found_network = true;
                        break;
                    }
                }
                Err(err) => {
                    error!("Failed to scan WiFi networks: {err:?}");
                }
            }

            // Wait before rescanning.
            warn!("Waiting for WiFi network to be available...");
            Timer::after_secs(30).await;
        }

        // Attempt to connect to the WiFi network.
        match controller.connect_async().await {
            Ok(()) => {
                info!("Connected to WiFi network");
                STOP_CONNECTION_SIGNAL.wait().await;

                // If signaled to stop, disconnect from the network
                info!("Disconnecting from WiFi network");
                if let Err(err) = controller.disconnect_async().await {
                    error!("Failed to disconnect from WiFi: {err:?}");
                }

                // Send a stop signal to the network task
                info!("Stopping WiFi background task");
                STOP_NETWORK_SIGNAL.signal(());

                return;
            }
            Err(err) => {
                error!("Failed to connect to WiFi: {err:?}");

                // After failed to connect too many times, rescan for the network.
                attempts += 1;
                if attempts >= max_attempts {
                    attempts = 0;
                    found_network = false;
                    warn!("Failed to connect to WiFi network, rescanning");
                }

                // Wait before trying again.
                Timer::after_secs(15).await;
            }
        }
    }
}
