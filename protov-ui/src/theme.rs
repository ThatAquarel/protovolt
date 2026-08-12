use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{RgbColor, WebColors},
};
use smart_leds::RGB8;

pub const FONT_MAIN: Rgb565 = Rgb565::CSS_WHITE;
pub const FONT_SMALL: Rgb565 = Rgb565::CSS_DIM_GRAY;

pub const BACKGROUND: Rgb565 = Rgb565::BLACK;
pub const SELECTED: Rgb565 = Rgb565::CSS_SILVER;
pub const UNSELECTED: Rgb565 = Rgb565::CSS_DIM_GRAY;
pub const NAVBAR_TEXT: Rgb565 = Rgb565::CSS_DIM_GRAY;
pub const WARNING: Rgb565 = Rgb565::CSS_RED;

pub const LED_OFF: RGB8 = RGB8::new(0, 0, 0);

/// https://github.com/olikraus/u8g2/wiki/fntgrpiconic
pub mod icons_1x {
    pub const UP_ARROW_THICK: &str = "\u{0053}";
}

pub mod icons_2x {
    pub const CHECKMARK: &str = "\u{0073}";
    pub const CROSS: &str = "\u{011B}";

    pub const PENCIL: &str = "\u{00E3}";
    pub const SETTINGS: &str = "\u{0081}";
    pub const SWITCH: &str = "\u{00CC}";

    pub const LIGHTNING: &str = "\u{0060}";
    pub const LINK: &str = "\u{00c6}";
}

pub mod navbar_layout {
    use embedded_graphics::prelude::Point;

    pub const BOX_WIDTH: u32 = 125;
    pub const BOX_HEIGHT: u32 = 30;
    pub const ICON_CENTER: Point = Point::new(17, 15);
    pub const TEXT_X: i32 = 32;
    pub const TEXT_Y: i32 = 9;
    pub const TEXT_LINE_GAP: i32 = 12;
    pub const BOX_STROKE_WIDTH: u32 = 2;
}

pub mod settings_layout {
    pub const SECTION_WIDTH: u32 = 157 + 163;
    pub const SECTION_HEIGHT: u32 = 200;
    pub const Y_OFFSET: i32 = 110;
    pub const X_OFFSET_TEXT: i32 = 32;
    pub const Y_SKIP: i32 = 24;
}
