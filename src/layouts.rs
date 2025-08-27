use epd_waveshare::epd7in5_v2::Display7in5;

use crate::widgets::joke::JokeFullWidget;
use crate::widgets::Widget;
use crate::wifi::WifiConnection;

pub struct JokeLayout {
    joke_widget: JokeFullWidget,
}

impl JokeLayout {
    pub fn new() -> Self {
        JokeLayout {
            joke_widget: JokeFullWidget::new(),
        }
    }
}

impl Widget for JokeLayout {
    fn draw(&self, display: &mut Box<Display7in5>) {
        self.joke_widget.draw(display);
    }

    fn refresh_data(&mut self, wifi_connection: &mut WifiConnection) {
        self.joke_widget.refresh_data(wifi_connection);
    }
}
