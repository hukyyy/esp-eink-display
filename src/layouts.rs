use epd_waveshare::epd7in5_v2::Display7in5;

pub trait Layout {
    fn draw(&self, display: &mut Box<Display7in5>);
}
