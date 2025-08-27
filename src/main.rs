use epd_waveshare::color::Color;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::{gpio::Level, peripherals::Peripherals};
use esp_idf_svc::nvs::EspDefaultNvsPartition;

use log::info;
use std::thread;
use std::time::Duration;

mod display;
mod internal_led;
mod layouts;
mod widgets;
mod wifi;

use crate::display::Display;
use crate::internal_led::{InternalLed, LedProgram};
use crate::layouts::{JokeLayout, Layout};

fn main() -> anyhow::Result<()> {
    initialise_esp32();

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut internal_led = InternalLed::new(peripherals.pins.gpio2);

    // =============== Wifi connection ===============

    internal_led.set_program(LedProgram::Blink(Duration::from_millis(500)));

    let mut wifi_connection =
        wifi::WifiConnection::new(peripherals.modem, sys_loop.clone(), nvs.clone());

    internal_led.set_program(LedProgram::Stable(Level::High));

    // ================= EPD ===================

    let mut display = Display::new(
        peripherals.spi2,
        peripherals.pins.gpio14,
        peripherals.pins.gpio13,
        peripherals.pins.gpio12,
        peripherals.pins.gpio15,
        peripherals.pins.gpio27,
        peripherals.pins.gpio26,
        peripherals.pins.gpio25,
        peripherals.pins.gpio33,
    )?;

    let mut joke_layout = JokeLayout::new();

    info!("Clearing display");
    display.clear_display(Color::Black)?;
    info!("Updating display");
    display.update_and_display()?;

    loop {
        thread::sleep(Duration::from_millis(5000));

        info!("Getting joke...");
        joke_layout.refresh_data(&mut wifi_connection);

        info!("About to clear display...");
        display.clear_display(Color::Black)?;
        info!("About to draw layout");
        display.draw_layout(&joke_layout);
        info!("About to update display ...");
        display.update_and_display()?;
    }
}

fn initialise_esp32() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
}
