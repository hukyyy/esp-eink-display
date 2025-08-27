use embedded_graphics::prelude::*;
use std::thread;
use std::time::Duration;

use epd_waveshare::{
    color::Color,
    epd7in5_v2::{Display7in5, Epd7in5},
    prelude::WaveshareDisplay,
};
use esp_idf_svc::hal::gpio::{Input, Output};
use esp_idf_svc::hal::{
    delay::Delay,
    gpio::{Gpio12, Gpio13, Gpio14, Gpio15, Gpio25, Gpio26, Gpio27, Gpio33, PinDriver},
    spi::{config::MODE_0, SpiConfig, SpiDeviceDriver, SpiDriver, SpiDriverConfig, SPI2},
};
use log::info;

use crate::layouts::Layout;

pub struct Display<'a> {
    _pwr: PinDriver<'a, Gpio33, Output>,
    delay: Delay,
    epd7in5: Epd7in5<
        SpiDeviceDriver<'a, SpiDriver<'a>>,
        PinDriver<'a, Gpio27, Input>,
        PinDriver<'a, Gpio26, Output>,
        PinDriver<'a, Gpio25, Output>,
        Delay,
    >,
    display: Box<Display7in5>,
    spi_device: SpiDeviceDriver<'a, SpiDriver<'a>>,
}

impl<'a> Display<'a> {
    pub fn new(
        spi: SPI2,
        sclk_pin: Gpio14,
        sdo_pin: Gpio13,
        sdi_pin: Gpio12,
        cs_pin: Gpio15,
        busy_pin: Gpio27,
        dc_pin: Gpio26,
        rst_pin: Gpio25,
        pwr_pin: Gpio33,
    ) -> anyhow::Result<Display<'a>> {
        let busy = PinDriver::input(busy_pin)?;
        let dc = PinDriver::output(dc_pin)?;
        let mut rst = PinDriver::output(rst_pin)?;
        let mut pwr = PinDriver::output(pwr_pin)?;

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

        let spi_driver = SpiDriver::new(spi, sclk_pin, sdo_pin, Some(sdi_pin), &spi_driver_config)?;

        let mut spi_device = SpiDeviceDriver::new(spi_driver, Some(cs_pin), &spi_device_config)?;

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

    pub fn draw_layout(&mut self, layout: &impl Layout) {
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
