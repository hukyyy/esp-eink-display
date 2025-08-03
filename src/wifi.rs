use embedded_svc::http::{client::Client as HttpClient, Method};

use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::modem::Modem,
    http::client::EspHttpConnection,
    nvs::EspDefaultNvsPartition,
    wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};

use log::info;

pub struct WifiConnection {
    pub client: HttpClient<EspHttpConnection>,
}

impl WifiConnection {
    pub fn new(
        modem: Modem,
        sys_loop: EspSystemEventLoop,
        nvs: EspDefaultNvsPartition,
    ) -> WifiConnection {
        let mut wifi = BlockingWifi::wrap(
            EspWifi::new(modem, sys_loop.clone(), Some(nvs)).unwrap(),
            sys_loop,
        )
        .unwrap();

        connect_wifi(&mut wifi).expect("Failed to connect to WiFi");

        let client = HttpClient::wrap(
            EspHttpConnection::new(&Default::default()).expect("Failed to create HTTP Connection."),
        );

        WifiConnection { client }
    }
}

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
