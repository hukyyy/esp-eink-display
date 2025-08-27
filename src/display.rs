use embedded_graphics::prelude::*;
use std::thread;
use std::time::Duration;

use epd_waveshare::{
    color::Color,
    epd7in5_v2::{Display7in5, Epd7in5},
    prelude::WaveshareDisplay,
};
use esp_idf_svc::hal::gpio::{AnyIOPin, Input, Output};
use esp_idf_svc::hal::{
    delay::Delay,
    gpio::PinDriver,
    spi::{config::MODE_0, SpiConfig, SpiDeviceDriver, SpiDriver, SpiDriverConfig, SPI2},
};
use log::info;

use crate::widgets::Widget;

type SpiDevice<'a> = SpiDeviceDriver<'a, SpiDriver<'a>>;
type InputPin<'a> = PinDriver<'a, AnyIOPin, Input>;
type OutputPin<'a> = PinDriver<'a, AnyIOPin, Output>;
type EpdDriver<'a> = Epd7in5<SpiDevice<'a>, InputPin<'a>, OutputPin<'a>, OutputPin<'a>, Delay>;

pub struct SpiPins {
    pub sclk_pin: AnyIOPin,
    pub sdo_pin: AnyIOPin,
    pub sdi_pin: AnyIOPin,
    pub cs_pin: AnyIOPin,
}

pub struct ControlPins {
    pub busy_pin: AnyIOPin,
    pub dc_pin: AnyIOPin,
    pub rst_pin: AnyIOPin,
    pub pwr_pin: AnyIOPin,
}

pub struct Display<'a> {
    _pwr: OutputPin<'a>,
    delay: Delay,
    epd7in5: EpdDriver<'a>,
    display: Box<Display7in5>,
    spi_device: SpiDevice<'a>,
}

impl<'a> Display<'a> {
    pub fn new(
        spi: SPI2,
        spi_pins: SpiPins,
        control_pins: ControlPins,
    ) -> anyhow::Result<Display<'a>> {
        let busy = PinDriver::input(control_pins.busy_pin)?;
        let dc = PinDriver::output(control_pins.dc_pin)?;
        let mut rst = PinDriver::output(control_pins.rst_pin)?;
        let mut pwr = PinDriver::output(control_pins.pwr_pin)?;

        let mut delay = Delay::new_default();

        info!("Powering display");
        pwr.set_high()?;
        delay.delay_ms(100);

        rst.set_low()?;
        delay.delay_ms(200);
        rst.set_high()?;
        delay.delay_ms(200);

        info!("Power and Reset sequence completed");

        let spi_device_config = SpiConfig::new()
            .baudrate(4_000_000.into())
            .data_mode(MODE_0);

        let spi_driver_config = SpiDriverConfig::new();

        let spi_driver = SpiDriver::new(
            spi,
            spi_pins.sclk_pin,
            spi_pins.sdo_pin,
            Some(spi_pins.sdi_pin),
            &spi_driver_config,
        )?;

        let mut spi_device =
            SpiDeviceDriver::new(spi_driver, Some(spi_pins.cs_pin), &spi_device_config)?;

        info!("SPI configured");

        let mut epd7in5 = Epd7in5::new(&mut spi_device, busy, dc, rst, &mut delay, None)?;
        let mut display = Box::new(Display7in5::default());

        info!("Clearing display to WHITE...");
        display.clear(Color::Black)?;
        epd7in5.update_and_display_frame(&mut spi_device, display.buffer(), &mut delay)?;

        thread::sleep(Duration::from_millis(100));

        Ok(Display {
            _pwr: pwr,
            delay,
            epd7in5,
            display,
            spi_device,
        })
    }

    pub fn clear_display(&mut self, color: Color) -> anyhow::Result<()> {
        self.display.clear(color)?;
        Ok(())
    }

    pub fn draw_layout(&mut self, layout: &impl Widget) {
        layout.draw(&mut self.display);
    }

    pub fn update_and_display(&mut self) -> anyhow::Result<()> {
        self.epd7in5.update_and_display_frame(
            &mut self.spi_device,
            self.display.buffer(),
            &mut self.delay,
        )?;
        Ok(())
    }
}

impl<'a> Drop for Display<'a> {
    fn drop(&mut self) {
        self.epd7in5
            .sleep(&mut self.spi_device, &mut self.delay)
            .unwrap();
    }
}
