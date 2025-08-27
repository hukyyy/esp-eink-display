use epd_waveshare::epd7in5_v2::Display7in5;

pub mod joke;

use crate::wifi::WifiConnection;

pub trait Widget {
    fn refresh_data(&mut self, wifi_connection: &mut WifiConnection);
    fn draw(&self, display: &mut Box<Display7in5>);
}
