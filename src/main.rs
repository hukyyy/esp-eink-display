use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::{
    gpio::{Gpio2, Output, PinDriver},
    peripherals::Peripherals,
};
use esp_idf_svc::nvs::EspDefaultNvsPartition;

use std::thread;
use std::time::Duration;

use std::sync::mpsc;

mod wifi;

fn main() {
    initialise_esp32();

    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    // let internal_led = InternalLed::new(peripherals.pins.gpio2);
    let mut internal_led = PinDriver::output(peripherals.pins.gpio2).unwrap();

    internal_led.set_high().unwrap();

    let wifi_connection =
        wifi::WifiConnection::new(peripherals.modem, sys_loop.clone(), nvs.clone());

    loop {
        internal_led.set_low().unwrap();
        thread::sleep(Duration::from_millis(500));
        internal_led.set_high().unwrap();
        thread::sleep(Duration::from_millis(500));
    }
}

fn initialise_esp32() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
}
