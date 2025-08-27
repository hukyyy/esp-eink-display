use embedded_graphics::{
    mono_font::MonoTextStyleBuilder,
    prelude::*,
    text::{Baseline, Text, TextStyleBuilder},
};
use epd_waveshare::color::Color;
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::Error;

use crate::{widgets::Widget, wifi::WifiConnection};

const PROGRAMMING_JOKE_URL: &str = "https://v2.jokeapi.dev/joke/Programming?type=single";

#[derive(Deserialize, Serialize, Default)]
struct JokeData {
    error: bool,
    joke: String,
}

fn fetch_data(wifi_connection: &mut WifiConnection, joke_data: &mut JokeData) {
    if let Ok(joke_json) = wifi_connection.get_request(PROGRAMMING_JOKE_URL) {
        let joke: Result<JokeData, Error> = serde_json::from_str(&joke_json);
        if let Ok(joke) = joke {
            match joke_data.error {
                false => {
                    *joke_data = joke;
                }
                true => {
                    info!("Failed to get a joke!");
                }
            }
        } else {
            info!("Failed to deserialize joke!");
        }
    }
}

//////////////////////////

pub struct JokeFullWidget {
    data: JokeData,
}

impl JokeFullWidget {
    pub fn new() -> Self {
        JokeFullWidget {
            data: JokeData::default(),
        }
    }
}

impl Widget for JokeFullWidget {
    fn refresh_data(&mut self, wifi_connection: &mut WifiConnection) {
        fetch_data(wifi_connection, &mut self.data);
    }

    fn draw(&self, display: &mut Box<epd_waveshare::epd7in5_v2::Display7in5>) {
        let character_style = MonoTextStyleBuilder::new()
            .font(&embedded_graphics::mono_font::iso_8859_1::FONT_10X20)
            .text_color(Color::White)
            .background_color(Color::Black)
            .build();

        let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();

        Text::with_text_style(
            &self.data.joke,
            Point::new(10, 50),
            character_style,
            text_style,
        )
        .draw(&mut **display)
        .expect("Failed to draw to display!");
    }
}
