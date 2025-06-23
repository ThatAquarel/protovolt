use embedded_graphics::{
    draw_target::Translated,
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point, RgbColor},
};

pub mod fmt;

pub mod boot;
pub mod controls;
pub mod navbar;

use boot::BootScreen;
use controls::ControlsScreen;
use navbar::Navbar;

use embedded_graphics::draw_target::DrawTargetExt;
use u8g2_fonts::{FontRenderer, fonts};

use crate::lib::{
    display::st7789,
    event::{Channel, FunctionButton, Limits, PowerType, Readout, SetState},
};

pub trait Display: DrawTarget<Color = Rgb565> {}
impl<T: DrawTarget<Color = Rgb565>> Display for T {}

pub struct Ui<'a, D>
where
    D: DrawTarget<Color = Rgb565>,
{
    pub target: &'a mut D,

    pub fonts: Fonts,
    pub layout: Layout,

    boot: BootScreen<'a>,
    controls: ControlsScreen,

    navbar: Navbar,
}

impl<'a, D> Ui<'a, D>
where
    D: DrawTarget<Color = Rgb565>,
{
    pub fn new(target: &'a mut D) -> Self {
        Self {
            target: target,
            fonts: Fonts::default(),
            layout: Layout {},

            boot: BootScreen::new(),
            controls: ControlsScreen::new(),

            navbar: Navbar::new(),
        }
    }

    pub fn clear(&mut self) -> Result<(), ()> {
        self.target.clear(color_scheme::BACKGROUND).map_err(|_| ())
    }

    pub fn boot_splash_screen(&mut self) -> Result<(), ()> {
        self.boot
            .draw_splash_screen(&mut *self.target, &mut self.layout)
    }

    pub fn boot_splash_text(
        &mut self,
        index: u8,
        title: &'static str,
        subtitle: &'static str,
        valid: bool,
    ) -> Result<(), ()> {
        self.boot.draw_splash_text(
            &mut *self.target,
            &mut self.layout,
            &self.fonts,
            index,
            title,
            subtitle,
            valid,
        )
    }

    pub fn controls_channel_box(&mut self, color: Rgb565, channel: Channel) -> Result<(), ()> {
        let text = match channel {
            Channel::A => labels::CHANNEL_A,
            Channel::B => labels::CHANNEL_B,
        };

        let mut target = self.layout.channel_section(&mut *self.target, channel);
        self.controls
            .draw_channel_background(&mut target, color)
            .map_err(|_| ())?;
        self.controls.draw_header_text(&mut target, text)
    }

    pub fn controls_channel_units(&mut self, channel: Channel) -> Result<(), ()> {
        let mut target = self.layout.channel_section(&mut *self.target, channel);
        self.controls.draw_units(&mut target, &self.fonts)
    }

    pub fn controls_measurement(&mut self, channel: Channel, readout: Readout) -> Result<(), ()> {
        let mut target = self.layout.channel_section(&mut *self.target, channel);
        self.controls
            .draw_measurements(&mut target, &self.fonts, readout)
    }

    pub fn controls_submeasurement(&mut self, channel: Channel, limits: Limits) -> Result<(), ()> {
        let mut target = self.layout.channel_section(&mut *self.target, channel);
        self.controls
            .draw_submeasurements(&mut target, &self.fonts, limits)
    }

    pub fn controls_submeasurement_tag(
        &mut self,
        channel: Channel,
        set_state: SetState,
    ) -> Result<(), ()> {
        let mut target = self.layout.channel_section(&mut *self.target, channel);

        let (top_tag, bottom_tag) = match set_state {
            SetState::SetLimits => (labels::SET, labels::SET),
            SetState::SetProtection => (labels::OVP, labels::OCP),
        };

        self.controls
            .draw_submeasurements_tag(&mut target, &self.fonts, top_tag, bottom_tag)
    }

    pub fn nav_power_info(&mut self, power_type: PowerType) -> Result<(), ()> {
        self.navbar.draw_power_info(&mut *self.target, &self.fonts, power_type)
    }

    pub fn nav_buttons(&mut self, button_state: Option<FunctionButton>) -> Result <(), ()> {
        self.navbar.draw_button(&mut *self.target, &self.fonts, button_state)
    }
}

pub struct Fonts {
    pub icons_2x: FontRenderer,
    pub icons_4x: FontRenderer,
    pub info_small: FontRenderer,
    pub info_large: FontRenderer,
    pub readout_small: FontRenderer,
    pub readout_large: FontRenderer,
}

impl Default for Fonts {
    fn default() -> Self {
        Self {
            icons_2x: FontRenderer::new::<fonts::u8g2_font_open_iconic_all_2x_t>(),
            // icons_4x: FontRenderer::new::<fonts::u8g2_font_open_iconic_all_4x_t>(),
            icons_4x: FontRenderer::new::<fonts::u8g2_font_open_iconic_other_4x_t>(),
            info_small: FontRenderer::new::<fonts::u8g2_font_helvB08_tf>(),
            info_large: FontRenderer::new::<fonts::u8g2_font_helvR14_tr>(),
            readout_small: FontRenderer::new::<fonts::u8g2_font_logisoso16_tn>(),
            readout_large: FontRenderer::new::<fonts::u8g2_font_logisoso32_tn>(),
        }
    }
}

//https://github.com/olikraus/u8g2/wiki/fntgrpiconic
pub mod icons_2x {
    pub const CHECKMARK: &str = "\u{0073}";
    pub const CROSS: &str = "\u{011B}";

    pub const SETTINGS: &str = "\u{0081}";
    pub const SWITCH: &str = "\u{00CC}";
}

pub mod icons_4x {
    pub const LIGHTNING: &str = "\u{0040}";
}


pub struct Layout;

impl Layout {
    pub fn width(&mut self) -> u16 {
        st7789::HEIGHT
    }

    pub fn height(&mut self) -> u16 {
        st7789::WIDTH
    }

    pub fn center_x(&mut self) -> i32 {
        self.width() as i32 / 2
    }

    pub fn center_y(&mut self) -> i32 {
        self.height() as i32 / 2
    }

    pub fn channel_section<'a, D>(
        &'a mut self,
        target: &'a mut D,
        channel: Channel,
    ) -> Translated<'a, D>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        match channel {
            Channel::A => self.ch_a_section(&mut *target),
            Channel::B => self.ch_b_section(&mut *target),
        }
    }

    fn ch_a_section<'a, D>(&'a mut self, target: &'a mut D) -> Translated<'a, D>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        target.translated(Point::new(0, 40))
    }

    fn ch_b_section<'a, D>(&'a mut self, target: &'a mut D) -> Translated<'a, D>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        target.translated(Point::new(163, 40))
    }
}

pub mod color_scheme {
    use embedded_graphics::{
        pixelcolor::Rgb565,
        prelude::{RgbColor, WebColors},
    };

    pub const FONT_MAIN: Rgb565 = Rgb565::CSS_WHITE;
    pub const FONT_SMALL: Rgb565 = Rgb565::CSS_DIM_GRAY;

    pub const BACKGROUND: Rgb565 = Rgb565::BLACK;
    pub const ACCENT: Rgb565 = Rgb565::CSS_WHITE;
    pub const SELECTED: Rgb565 = Rgb565::CSS_SILVER;
    pub const UNSELECTED: Rgb565 = Rgb565::CSS_DIM_GRAY;

    pub const CH_A_SELECTED: Rgb565 = Rgb565::CSS_RED;
    pub const CH_A_UNSELECTED: Rgb565 = Rgb565::CSS_DARK_RED;
    pub const CH_B_SELECTED: Rgb565 = Rgb565::CSS_BLUE;
    pub const CH_B_UNSELECTED: Rgb565 = Rgb565::CSS_DARK_BLUE;
}

pub mod labels {
    pub const CHANNEL_A: &'static str = "CHANNEL A";
    pub const CHANNEL_B: &'static str = "CHANNEL B";

    pub const VOLT: &'static str = "V";
    pub const AMPERE: &'static str = "A";
    pub const WATT: &'static str = "W";

    pub const SET: &'static str = "SET";
    pub const OVP: &'static str = "OVP";
    pub const OCP: &'static str = "OCP";
}
