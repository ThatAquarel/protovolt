use embedded_graphics::pixelcolor::Rgb565;
use smart_leds::RGB8;

pub use crate::config::{
    DARKEN_DEN, DARKEN_NUM, DEFAULT_CH1, DEFAULT_CH2, DEFAULT_LCD_BRIGHTNESS,
    DEFAULT_LED_BRIGHTNESS, Rgb,
};
pub use protov_scpi::{format_rgb, parse_brightness, parse_rgb_triplet};

pub fn darken(rgb: Rgb) -> Rgb {
    Rgb {
        r: scale_channel(rgb.r),
        g: scale_channel(rgb.g),
        b: scale_channel(rgb.b),
    }
}

fn scale_channel(c: u8) -> u8 {
    ((c as u32 * DARKEN_NUM) / DARKEN_DEN) as u8
}

pub fn to_rgb565(rgb: Rgb) -> Rgb565 {
    Rgb565::new(rgb.r >> 3, rgb.g >> 2, rgb.b >> 3)
}

pub fn scale_led(rgb: Rgb, brightness: u8) -> RGB8 {
    RGB8::new(
        scale_channel_brightness(rgb.r, brightness),
        scale_channel_brightness(rgb.g, brightness),
        scale_channel_brightness(rgb.b, brightness),
    )
}

pub fn scale_white(brightness: u8) -> RGB8 {
    let v = scale_channel_brightness(255, brightness);
    RGB8::new(v, v, v)
}

fn scale_channel_brightness(c: u8, brightness: u8) -> u8 {
    ((c as u32 * brightness as u32) / 255) as u8
}

#[cfg(test)]
mod tests;
