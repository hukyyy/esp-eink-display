use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::{gpio::Level, peripherals::Peripherals};
use esp_idf_svc::nvs::EspDefaultNvsPartition;

use log::info;
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use internal_led::{InternalLed, LedProgram};

mod display;
mod internal_led;
mod wifi;

fn main() -> ! {
    initialise_esp32();

    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let mut internal_led = InternalLed::new(peripherals.pins.gpio2);

    internal_led.set_program(LedProgram::Blink(Duration::from_millis(500)));

    let mut wifi_connection =
        wifi::WifiConnection::new(peripherals.modem, sys_loop.clone(), nvs.clone());

    internal_led.set_program(LedProgram::Stable(Level::High));

    loop {
        thread::sleep(Duration::from_millis(5000));
        info!("Getting joke!!!");

        if let Ok(joke_json) =
            wifi_connection.get_request("https://v2.jokeapi.dev/joke/Programming?type=single")
        {
            let joke: JokeResponse = serde_json::from_str(&joke_json).unwrap();

            match joke.error {
                false => {
                    info!("Got a joke!");
                    info!("{}", joke.joke);
                }
                true => {
                    info!("Failed to get a joke");
                }
            }
        }
    }
}

fn initialise_esp32() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
}

#[derive(Deserialize, Serialize)]
struct JokeResponse {
    error: bool,
    joke: String,
}
