use embedded_svc::{http::client::Client as HttpClient, utils::io};

use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::modem::Modem,
    http::client::{Configuration as HttpConfiguration, EspHttpConnection},
    nvs::EspDefaultNvsPartition,
    wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};

use log::info;

pub struct WifiConnection<'w> {
    pub client: HttpClient<EspHttpConnection>,
    _wifi: BlockingWifi<EspWifi<'w>>,
}

impl<'w> WifiConnection<'w> {
    /// Returns an active wifi connection
    ///
    /// # Panics
    /// If connection to wifi using credentials stored in the environment fails.
    pub fn new(
        modem: Modem,
        sys_loop: EspSystemEventLoop,
        nvs: EspDefaultNvsPartition,
    ) -> WifiConnection<'w> {
        let mut wifi = BlockingWifi::wrap(
            EspWifi::new(modem, sys_loop.clone(), Some(nvs)).unwrap(),
            sys_loop,
        )
        .unwrap();

        connect_wifi(&mut wifi).expect("Failed to connect to WiFi");

        let http_config = HttpConfiguration {
            crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
            ..Default::default()
        };

        let client = HttpClient::wrap(
            EspHttpConnection::new(&http_config).expect("Failed to create HTTP Connection."),
        );

        WifiConnection {
            client,
            _wifi: wifi,
        }
    }

    /// Returns a String of the result if the get request is successful, anyhow::Error otherwise.
    pub fn get_request(&mut self, url: &str) -> anyhow::Result<String> {
        let request = self.client.get(url).unwrap();
        info!("GET {url}");
        let mut response = request.submit()?;

        let status = response.status();
        let status_message = response.status_message().unwrap_or("");
        info!("-> {status} {status_message}");

        let mut buf = [0u8; 2048];
        let bytes_read = io::try_read_full(&mut response, &mut buf).map_err(|e| e.0)?;
        info!("Read {bytes_read} bytes!");

        match std::str::from_utf8(&buf[0..bytes_read]) {
            Ok(body_string) => Ok(String::from(body_string)),
            Err(e) => Err(anyhow::Error::from(e)),
        }
    }
}

/// Connects to wifi using credentials stored in the environment.
fn connect_wifi(wifi: &mut BlockingWifi<EspWifi>) -> anyhow::Result<()> {
    const SSID: &str = env!("SSID");
    const PASSWORD: &str = env!("PASSWORD");

    let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
        ssid: SSID.try_into().expect("Failed to get SSID."),
        bssid: None,
        auth_method: AuthMethod::WPA2Personal,
        password: PASSWORD.try_into().expect("Failed to get PASSWORD."),
        channel: None,
        ..Default::default()
    });

    wifi.set_configuration(&wifi_configuration)?;

    wifi.start()?;
    info!("Wifi started");

    wifi.connect()?;
    info!("Wifi connected");

    wifi.wait_netif_up()?;
    info!("Wifi netif up");

    Ok(())
}
