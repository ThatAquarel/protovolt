//! Default channel colors and brightness (factory / *RST appearance).

pub use protov_scpi::Rgb;

#[derive(Clone, Copy, Debug)]
pub struct AppearanceDefaults {
    pub ch1: Rgb,
    pub ch2: Rgb,
    pub led_brightness: u8,
    pub lcd_brightness: u8,
    pub darken_num: u32,
    pub darken_den: u32,
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

/// CSS #FF0000 → #8B0000 per-channel ratio for unselected LCD colors.
pub const DARKEN_NUM: u32 = 139;
pub const DARKEN_DEN: u32 = 255;

pub const DEFAULT: AppearanceDefaults = AppearanceDefaults {
    ch1: DEFAULT_CH1,
    ch2: DEFAULT_CH2,
    led_brightness: DEFAULT_LED_BRIGHTNESS,
    lcd_brightness: DEFAULT_LCD_BRIGHTNESS,
    darken_num: DARKEN_NUM,
    darken_den: DARKEN_DEN,
};
