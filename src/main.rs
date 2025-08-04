use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::gpio::Level;
use esp_idf_svc::hal::{
    gpio::{Gpio2, PinDriver},
    peripherals::Peripherals,
};
use esp_idf_svc::nvs::EspDefaultNvsPartition;

use log::info;

use std::thread;
use std::time::Duration;

use std::sync::mpsc::{self, Sender};

use serde::{Deserialize, Serialize};

mod wifi;

enum LedProgram {
    Stable(Level),
    Blink(Duration),
    Shutdown,
}

struct InternalLed {
    thread_handle: Option<thread::JoinHandle<()>>,
    tx: Sender<LedProgram>,
}

impl<'a> InternalLed {
    fn new(gpio2: Gpio2) -> InternalLed {
        let (tx, rx) = mpsc::channel();
        let mut pin = PinDriver::output(gpio2).unwrap();
        let mut program = LedProgram::Stable(Level::High);

        let thread_handle = thread::spawn(move || loop {
            if let Ok(p) = rx.recv_timeout(Duration::from_millis(10)) {
                program = p;
            };

            match program {
                LedProgram::Stable(level) => {
                    pin.set_level(level).unwrap();
                    thread::sleep(Duration::from_millis(100));
                }
                LedProgram::Blink(duration) => {
                    pin.set_low().unwrap();
                    thread::sleep(duration);
                    pin.set_high().unwrap();
                    thread::sleep(duration - Duration::from_millis(10));
                }
                LedProgram::Shutdown => break,
            }
        });

        InternalLed {
            thread_handle: Some(thread_handle),
            tx,
        }
    }

    fn set_program(&mut self, program: LedProgram) {
        self.tx.send(program);
    }
}

impl Drop for InternalLed {
    fn drop(&mut self) {
        self.tx.send(LedProgram::Shutdown);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

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
