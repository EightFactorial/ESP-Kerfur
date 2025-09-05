//! The WiFi module.

use embassy_executor::Spawner;
use embassy_net::{Config, DhcpConfig, Runner, Stack, StackResources};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Instant, Timer};
use esp_hal::{peripherals::WIFI, rng::Rng, timer::timg::Timer as TimgTimer};
use esp_wifi::{
    config::PowerSaveMode,
    wifi::{
        AuthMethod, ClientConfiguration, Configuration, WifiController, WifiDevice, WifiError,
        WifiEvent, WifiState, wifi_state,
    },
};
use futures_lite::future;
use log::{debug, error, info, warn};
use sntpc::NtpTimestampGenerator;
use static_cell::make_static;

/// A handle to the WiFi stack.
pub(super) struct WiFiStack(Stack<'static>);

impl WiFiStack {
    /// The WiFi authentication method to use.
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
        _ => core::panic!("Unknown WiFi authentication method"),
    };
    /// The WiFi password, if any.
    const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");
    /// The WiFi SSID to connect to.
    const WIFI_SSID: &str = env!("WIFI_SSID");

    /// Access the inner [`Stack`].
    #[must_use]
    pub(super) const fn stack(&self) -> Stack<'static> { self.0 }

    // ---------------------------------------------------------------------------------------------

    /// Create a new [`WiFiStack`].
    ///
    /// # Panics
    /// Only one stack can ever exist.
    /// Calling this function twice will immediately panic.
    pub(super) async fn new(
        spawner: Spawner,
        timer: TimgTimer<'static>,
        wifi: WIFI<'static>,
        mut rng: Rng,
    ) -> Result<Self, WifiError> {
        // Initialize the WiFi radio.
        let controller =
            make_static!(esp_wifi::init(timer, rng).expect("Failed to initialize `esp_wifi`?"));

        // Initialize the WiFi controller and interfaces.
        let (mut controller, interfaces) = match esp_wifi::wifi::new(controller, wifi) {
            Ok((controller, interfaces)) => (controller, interfaces),
            Err(err) => {
                error!("Failed to create WiFi controller: {err:?}");
                return Err(err);
            }
        };
        let _ = controller.set_power_saving(PowerSaveMode::Minimum);

        // Concatenate two random u32 values to create a u64 seed.
        let (seed_a, seed_b) = (rng.random().to_ne_bytes(), rng.random().to_ne_bytes());
        let seed = u64::from_ne_bytes([
            seed_a[0], seed_a[1], seed_a[2], seed_a[3], seed_b[0], seed_b[1], seed_b[2], seed_b[3],
        ]);

        // Create a new stack and network configuration.
        let resources: &mut StackResources<4> = make_static!(StackResources::new());
        let config = Config::dhcpv4(DhcpConfig::default());

        // Spawn the network and connection tasks.
        let (stack, runner) = embassy_net::new(interfaces.sta, config, resources, seed);
        spawner.must_spawn(network_task(make_static!(runner)));
        spawner.must_spawn(connection_task(make_static!(controller)));

        stack.wait_link_up().await;
        info!("Waiting for DHCP configuration...");
        stack.wait_config_up().await;

        if let Some(config) = stack.config_v4() {
            info!("Assigned network address: {}", config.address);
        } else {
            warn!("Assumed to have an address, but none found?");
        }

        Ok(Self(stack))
    }
}

// -------------------------------------------------------------------------------------------------

/// A [`Signal`] to stop the network task.
static STOP_NETWORK_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

#[embassy_executor::task]
async fn network_task(runner: &'static mut Runner<'static, WifiDevice<'static>>) {
    info!("Starting network background task");
    future::or(async { runner.run().await }, STOP_NETWORK_SIGNAL.wait()).await;
    info!("Stopping network background task");
}

/// A [`Signal`] to stop the connection task.
pub(super) static STOP_WIFI_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

#[embassy_executor::task]
async fn connection_task(controller: &'static mut WifiController<'static>) {
    #[cfg(feature = "logging")]
    if const { env!("ESP_LOG").eq_ignore_ascii_case("trace") } {
        esp_wifi::wifi_set_log_verbose();
    }

    info!("Starting WiFi background task");
    let _ = controller.disconnect_async().await;

    let mut found_network = true;
    let mut warn_auth_method = false;

    let mut attempts = 0u8;
    let max_attempts = 8u8;

    loop {
        // If we're already connected, wait until we disconnect.
        if matches!(wifi_state(), WifiState::StaConnected) {
            controller.wait_for_event(WifiEvent::StaDisconnected).await;
            Timer::after_secs(10).await;
        }

        // If the controller hasn't been started, configure and start it.
        if !matches!(controller.is_started(), Ok(true)) {
            let config = ClientConfiguration {
                ssid: WiFiStack::WIFI_SSID.into(),
                password: WiFiStack::WIFI_PASSWORD.into(),
                auth_method: WiFiStack::WIFI_AUTH_METHOD,
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
                        debug!(
                            "    SSID: \"{}\", BSSID: {:?}, RSSI: {}, AUTH: {:?}",
                            net.ssid, net.bssid, net.signal_strength, net.auth_method
                        );
                        if net.auth_method == Some(WiFiStack::WIFI_AUTH_METHOD) {
                            return net.ssid == WiFiStack::WIFI_SSID;
                        } else if warn_auth_method {
                            warn!("Found target WiFi network, but authentication method does not match!");
                            warn_auth_method = true;
                        }

                        false
                    }) {
                        info!("Found target WiFi network: \"{}\"", WiFiStack::WIFI_SSID);
                        found_network = true;
                        break;
                    }
                }
                Err(err) => {
                    error!("Failed to scan WiFi networks: {err:?}");
                }
            }

            warn!("Waiting for WiFi network to be available...");

            // Wait before rescanning.
            Timer::after_secs(30).await;
        }

        // Attempt to connect to the WiFi network.
        match controller.connect_async().await {
            Ok(()) => {
                info!("Connected to WiFi network");
                STOP_WIFI_SIGNAL.wait().await;

                info!("Disconnecting from WiFi network");
                if let Err(err) = controller.disconnect_async().await {
                    error!("Failed to disconnect from WiFi network: {err:?}");
                }

                info!("Stopping WiFi background task");
                STOP_NETWORK_SIGNAL.signal(());

                return;
            }
            Err(err) => {
                error!("Failed to connect to WiFi network: {err:?}");

                attempts += 1;
                if attempts >= max_attempts {
                    attempts = 0;
                    found_network = false;
                    warn!("Failed to connect to WiFi network, rescanning");
                }

                // Wait before trying again.
                Timer::after_secs(10).await;
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// A simple [`NtpTimestampGenerator`] implementation
/// that uses [`Instant`] to generate timestamps.
#[derive(Clone, Copy)]
pub(super) struct TimestampGenerator;

impl NtpTimestampGenerator for TimestampGenerator {
    fn init(&mut self) {}

    fn timestamp_sec(&self) -> u64 { Instant::now().as_secs() }

    fn timestamp_subsec_micros(&self) -> u32 {
        let instant = Instant::now();
        let fraction = instant.as_micros().saturating_sub(instant.as_secs() * 1_000_000);
        u32::try_from(fraction).unwrap_or_default()
    }
}
