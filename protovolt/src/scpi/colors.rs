use core::fmt::Write;

use embedded_graphics::pixelcolor::Rgb565;
use smart_leds::RGB8;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub const DEFAULT_CH1: Rgb = Rgb {
    r: 234,
    g: 67,
    b: 53,
};

pub const DEFAULT_CH2: Rgb = Rgb {
    r: 66,
    g: 133,
    b: 244,
};

pub const DEFAULT_LED_BRIGHTNESS: u8 = 10;
pub const DEFAULT_LCD_BRIGHTNESS: u8 = 255;

/// CSS #FF0000 → #8B0000 per-channel ratio.
const DARKEN_NUM: u32 = 139;
const DARKEN_DEN: u32 = 255;

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
    Rgb565::new(
        rgb.r >> 3,
        rgb.g >> 2,
        rgb.b >> 3,
    )
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

pub fn format_rgb(rgb: Rgb) -> heapless::String<16> {
    let mut buf = heapless::String::new();
    let _ = write!(buf, "{},{},{}", rgb.r, rgb.g, rgb.b);
    buf
}

pub fn parse_rgb_triplet(raw: &str) -> Result<Rgb, ()> {
    let parts: heapless::Vec<&str, 3> = raw.split(',').collect();
    if parts.len() != 3 {
        return Err(());
    }
    let r = parts[0].trim().parse::<u16>().map_err(|_| ())?;
    let g = parts[1].trim().parse::<u16>().map_err(|_| ())?;
    let b = parts[2].trim().parse::<u16>().map_err(|_| ())?;
    if r > 255 || g > 255 || b > 255 {
        return Err(());
    }
    Ok(Rgb {
        r: r as u8,
        g: g as u8,
        b: b as u8,
    })
}

pub fn parse_brightness(raw: &str) -> Result<u8, ()> {
    let value = raw.trim().parse::<u16>().map_err(|_| ())?;
    if value > 255 {
        return Err(());
    }
    Ok(value as u8)
}
