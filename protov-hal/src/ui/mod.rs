use embassy_rp::pio::Instance;
use embedded_graphics::{
    draw_target::Translated,
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Drawable, Point, Primitive, Size},
    primitives::{PrimitiveStyle, Rectangle},
};

pub mod boot;
pub mod controls;
pub mod navbar;
pub mod settings;

use boot::BootScreen;
use controls::ControlsScreen;
use navbar::{Navbar, PowerInfoDisplay};
use settings::SettingsScreen;

use embedded_graphics::draw_target::DrawTargetExt;
use u8g2_fonts::{FontRenderer, fonts};

use crate::{
    app::{DecimalPrecision, SetSelect},
    hal::{
        backlight::Backlight,
        display::st7789,
        event::{
            Channel, ChannelFocus, ChannelHardwareState, ConfirmState, DfuStatus, FunctionButton,
            Limits, PowerType, Readout, SetState,
        },
        led::{LedsColor, LedsInterface},
    },
    scpi::{self, state::ScpiState},
};

pub trait Display: DrawTarget<Color = Rgb565> {}
impl<T: DrawTarget<Color = Rgb565>> Display for T {}

pub struct Ui<'disp, 'bl, D, PIO>
where
    D: DrawTarget<Color = Rgb565>,
    PIO: Instance,
{
    pub target: &'disp mut D,
    pub led_interface: LedsInterface<'disp, PIO>,
    backlight: Backlight<'bl>,

    pub fonts: Fonts,
    pub layout: Layout,

    boot: BootScreen<'disp>,
    controls: ControlsScreen,
    settings: SettingsScreen,

    navbar: Navbar,
    settings_visible: bool,
    dfu_screen_active: bool,
}

impl<'disp, 'bl, D, PIO> Ui<'disp, 'bl, D, PIO>
where
    D: DrawTarget<Color = Rgb565>,
    PIO: Instance,
{
    pub fn new(
        target: &'disp mut D,
        led_interface: LedsInterface<'disp, PIO>,
        backlight: Backlight<'bl>,
    ) -> Self {
        Self {
            target,
            led_interface,
            backlight,

            fonts: Fonts::default(),
            layout: Layout {},

            boot: BootScreen::new(),
            controls: ControlsScreen::new(),
            settings: SettingsScreen::new(),

            navbar: Navbar::new(),
            settings_visible: false,
            dfu_screen_active: false,
        }
    }

    pub fn dfu_screen_active(&self) -> bool {
        self.dfu_screen_active
    }

    pub fn lcd_brightness(&self) -> u8 {
        self.backlight.level()
    }

    pub fn set_lcd_brightness(&mut self, level: u8) {
        self.backlight.set_brightness(level);
    }

    pub fn set_settings_visible(&mut self, visible: bool) {
        self.settings_visible = visible;
    }

    pub fn settings_visible(&self) -> bool {
        self.settings_visible
    }

    pub fn clear_settings_section(&mut self) -> Result<(), ()> {
        let mut target = self.layout.settings_section(&mut *self.target);
        Rectangle::new(
            Point::new(0, 0),
            Size::new(
                settings_layout::SECTION_WIDTH,
                settings_layout::SECTION_HEIGHT,
            ),
        )
        .into_styled(PrimitiveStyle::with_fill(color_scheme::BACKGROUND))
        .draw(&mut target)
        .map_err(|_| ())
    }

    pub fn draw_settings_overlay(&mut self) -> Result<(), ()> {
        self.clear_settings_section()?;
        self.paint_settings_overlay()
    }

    pub fn redraw_settings_overlay_if_visible(&mut self) -> Result<(), ()> {
        if self.settings_visible {
            self.paint_settings_overlay()?;
        }
        Ok(())
    }

    fn paint_settings_overlay(&mut self) -> Result<(), ()> {
        let width = self.layout.width() as u32;
        let mut target = self.layout.settings_section(&mut *self.target);
        self.settings
            .draw_background(&mut target, color_scheme::SELECTED)
            .map_err(|_| ())?;
        self.controls
            .draw_header_text(&mut target, labels::SETTINGS)?;
        self.settings.draw(&mut target, &self.fonts, width)
    }

    pub fn clear(&mut self) -> Result<(), ()> {
        self.backlight
            .while_suppressed(|| self.target.clear(color_scheme::BACKGROUND).map_err(|_| ()))
    }

    #[cfg(feature = "demo")]
    pub fn boot_demo_mode(&mut self) -> Result<(), ()> {
        self.boot
            .draw_demo_screen(&mut *self.target, &mut self.layout, &self.fonts)
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

    pub fn draw_dfu_status(&mut self, status: DfuStatus) -> Result<(), ()> {
        match status {
            DfuStatus::Idle => {
                self.dfu_screen_active = false;
                Ok(())
            }
            DfuStatus::Preparing { .. } => {
                self.dfu_screen_active = true;
                self.clear()?;
                self.boot
                    .draw_splash_screen(&mut *self.target, &mut self.layout)?;
                self.boot
                    .draw_dfu_text(&mut *self.target, &self.fonts, status)
            }
            DfuStatus::Receiving { .. }
            | DfuStatus::Ready { .. }
            | DfuStatus::Verified
            | DfuStatus::Error => {
                self.dfu_screen_active = true;
                self.boot
                    .draw_dfu_text(&mut *self.target, &self.fonts, status)
            }
            DfuStatus::Flashing => {
                self.dfu_screen_active = true;
                self.backlight.turn_off();
                self.boot
                    .draw_dfu_text(&mut *self.target, &self.fonts, status)
            }
        }
    }

    fn channel_color(scpi: &ScpiState, channel: Channel, focus: ChannelFocus) -> Rgb565 {
        match focus {
            ChannelFocus::SelectedInactive => color_scheme::SELECTED,
            ChannelFocus::UnselectedInactive => color_scheme::UNSELECTED,
            ChannelFocus::SelectedActive => scpi.selected_rgb565(channel),
            ChannelFocus::UnselectedActive => scpi.unselected_rgb565(channel),
        }
    }

    pub async fn controls_channel_box(
        &mut self,
        scpi: &ScpiState,
        channel: Channel,
        focus: ChannelFocus,
    ) -> Result<(), ()> {
        let color = Self::channel_color(scpi, channel, focus);

        let text = match channel {
            Channel::A => labels::CHANNEL_A,
            Channel::B => labels::CHANNEL_B,
        };

        let mut target = self.layout.channel_section(&mut *self.target, channel);
        self.controls
            .draw_channel_background(&mut target, color)
            .map_err(|_| ())?;
        self.controls.draw_header_text(&mut target, text)?;

        let led_color = match focus {
            ChannelFocus::SelectedInactive | ChannelFocus::UnselectedInactive => match channel {
                Channel::A => LedsColor::ChannelA(color_scheme::LED_OFF, color_scheme::LED_OFF),
                Channel::B => LedsColor::ChannelB(color_scheme::LED_OFF, color_scheme::LED_OFF),
            },
            ChannelFocus::SelectedActive | ChannelFocus::UnselectedActive => {
                let led = scpi.channel_led(channel);
                match channel {
                    Channel::A => LedsColor::ChannelA(led, led),
                    Channel::B => LedsColor::ChannelB(led, led),
                }
            }
        };

        self.led_interface.update_refresh(led_color).await;

        Ok(())
    }

    pub fn controls_header_chip(
        &mut self,
        scpi: &ScpiState,
        channel: Channel,
        focus: ChannelFocus,
        channel_hardware_state: ChannelHardwareState,
    ) -> Result<(), ()> {
        let mut target = self.layout.channel_section(&mut *self.target, channel);
        let color = Self::channel_color(scpi, channel, focus);
        let (invert, text) = match channel_hardware_state {
            ChannelHardwareState::Off => (false, ""),

            ChannelHardwareState::ConstantVoltage => (false, labels::CONSTANT_VOLTAGE),
            ChannelHardwareState::ConstantCurrent => (false, labels::CONSTANT_CURRENT),

            ChannelHardwareState::ShortCircuit => (true, labels::SHORT_CIRCUIT),
            ChannelHardwareState::OverTemperature => (true, labels::OVER_TEMPERATURE),
            ChannelHardwareState::OverCurrent => (true, labels::OVER_CURRENT),
            ChannelHardwareState::OverVoltage => (true, labels::OVER_VOLTAGE),
        };

        self.controls
            .draw_header_chip(&mut target, &self.fonts, invert, color, text)?;

        Ok(())
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

    pub fn controls_submeasurement(
        &mut self,
        scpi: &ScpiState,
        channel: Channel,
        set_select: Option<SetSelect>,
        limits: Limits,
        confirm_state: ConfirmState,
        select_precision: Option<DecimalPrecision>,
    ) -> Result<(), ()> {
        let mut target = self.layout.channel_section(&mut *self.target, channel);
        self.controls.draw_submeasurements(
            &mut target,
            &self.fonts,
            scpi.selected_rgb565(channel),
            set_select,
            limits,
            confirm_state,
            select_precision,
        )
    }

    pub fn controls_submeasurement_tag(
        &mut self,
        scpi: &ScpiState,
        channel: Channel,
        set_state: SetState,
        set_select: Option<SetSelect>,
        confirm_state: ConfirmState,
    ) -> Result<(), ()> {
        let mut target = self.layout.channel_section(&mut *self.target, channel);

        let (top_tag, bottom_tag) = match set_state {
            SetState::Set => (labels::SET, labels::SET),
            SetState::Limits => (labels::OVP, labels::OCP),
        };

        self.controls.draw_submeasurements_tag(
            &mut target,
            &self.fonts,
            scpi.selected_rgb565(channel),
            set_select,
            top_tag,
            bottom_tag,
            confirm_state,
        )
    }

    pub fn nav_power_info(&mut self, power_type: PowerType) -> Result<(), ()> {
        let info = PowerInfoDisplay::new(power_type, scpi::serial_connected());
        self.navbar
            .draw_power_info(&mut *self.target, &self.fonts, info)
    }

    pub async fn nav_buttons(
        &mut self,
        scpi: &ScpiState,
        confirm_state: ConfirmState,
        button_state: Option<FunctionButton>,
    ) -> Result<(), ()> {
        let white = scpi.white_led();
        let (switch_color, settings_color) = match &button_state {
            Some(button) => match button {
                FunctionButton::Switch => (white, color_scheme::LED_OFF),
                FunctionButton::Settings => (color_scheme::LED_OFF, white),
                _ => (color_scheme::LED_OFF, color_scheme::LED_OFF),
            },
            None => (color_scheme::LED_OFF, color_scheme::LED_OFF),
        };
        let enter_led_color = match confirm_state {
            ConfirmState::AwaitConfirmModify(channel) => match channel {
                Some(channel) => scpi.channel_led(channel),
                None => color_scheme::LED_OFF,
            },
            ConfirmState::AwaitModify => color_scheme::LED_OFF,
        };

        self.navbar
            .draw_button(&mut *self.target, &self.fonts, confirm_state, button_state)?;

        self.led_interface
            .update_color(LedsColor::Enter(enter_led_color));
        self.led_interface
            .update_color(LedsColor::Switch(switch_color));
        self.led_interface
            .update_color(LedsColor::Settings(settings_color));
        self.led_interface.refresh().await;
        Ok(())
    }
}

pub struct Fonts {
    pub icons_1x: FontRenderer,
    pub icons_2x: FontRenderer,
    pub info_small: FontRenderer,
    pub info_navbar: FontRenderer,
    pub info_large: FontRenderer,
    pub readout_small: FontRenderer,
    pub readout_large: FontRenderer,
}

impl Default for Fonts {
    fn default() -> Self {
        Self {
            icons_1x: FontRenderer::new::<fonts::u8g2_font_open_iconic_arrow_1x_t>(),
            icons_2x: FontRenderer::new::<fonts::u8g2_font_open_iconic_all_2x_t>(),
            info_small: FontRenderer::new::<fonts::u8g2_font_helvB08_tf>(),
            info_navbar: FontRenderer::new::<fonts::u8g2_font_profont11_tr>(),
            info_large: FontRenderer::new::<fonts::u8g2_font_helvR14_tr>(),
            readout_small: FontRenderer::new::<fonts::u8g2_font_logisoso16_tr>(),
            readout_large: FontRenderer::new::<fonts::u8g2_font_logisoso32_tn>(),
        }
    }
}

//https://github.com/olikraus/u8g2/wiki/fntgrpiconic
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

pub struct Layout;

impl Layout {
    pub fn width(&mut self) -> u16 {
        st7789::HEIGHT
    }

    // pub fn height(&mut self) -> u16 {
    //     st7789::WIDTH
    // }

    pub fn center_x(&mut self) -> i32 {
        self.width() as i32 / 2
    }

    // pub fn center_y(&mut self) -> i32 {
    //     self.height() as i32 / 2
    // }

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

    fn settings_section<'a, D>(&'a mut self, target: &'a mut D) -> Translated<'a, D>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        self.ch_a_section(target)
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
    use smart_leds::RGB8;

    pub const FONT_MAIN: Rgb565 = Rgb565::CSS_WHITE;
    pub const FONT_SMALL: Rgb565 = Rgb565::CSS_DIM_GRAY;

    pub const BACKGROUND: Rgb565 = Rgb565::BLACK;
    // pub const ACCENT: Rgb565 = Rgb565::CSS_WHITE;
    pub const SELECTED: Rgb565 = Rgb565::CSS_SILVER;
    pub const UNSELECTED: Rgb565 = Rgb565::CSS_DIM_GRAY;
    pub const NAVBAR_TEXT: Rgb565 = Rgb565::CSS_DIM_GRAY;
    pub const WARNING: Rgb565 = Rgb565::CSS_RED;

    pub const LED_OFF: RGB8 = RGB8::new(0, 0, 0);
}

pub mod labels {
    // Boot
    pub const INPUT: &'static str = "INPUT";
    pub const PD: &'static str = "USB-C PD";
    pub const STD: &'static str = "USB 2.0";

    pub const SENSE: &'static str = "SENSE";
    pub const CONVERTER: &'static str = "CONVERTER";

    pub const PASS: &'static str = "PASS";
    pub const FAIL: &'static str = "FAIL";

    #[cfg(feature = "demo")]
    pub const DEMO: &'static str = concat!("DEMO BUILD v", env!("CARGO_PKG_VERSION"));

    // Controls
    pub const CHANNEL_A: &'static str = "CHANNEL A";
    pub const CHANNEL_B: &'static str = "CHANNEL B";

    pub const VOLT: &'static str = "V";
    pub const AMPERE: &'static str = "A";
    pub const WATT: &'static str = "W";

    pub const SET: &'static str = "SET";
    pub const OVP: &'static str = "OVP";
    pub const OCP: &'static str = "OCP";

    // Channel Hardware State
    pub const CONSTANT_VOLTAGE: &'static str = "CV";
    pub const CONSTANT_CURRENT: &'static str = "CC";

    pub const SHORT_CIRCUIT: &'static str = "SHORT";
    pub const OVER_TEMPERATURE: &'static str = "TEMP";
    pub const OVER_CURRENT: &'static str = "OCP";
    pub const OVER_VOLTAGE: &'static str = "OVP";

    // Settings
    pub const SETTINGS: &'static str = "SETTINGS";
    pub const MANAGE_AT: &'static str = "MANAGE AT";
    pub const WEBSITE: &'static str = "www.protov.app";
    pub const FW_VERSION: &'static str = "FW VERSION";
    pub const HW_VERSION: &'static str = "HW VERSION";
    pub const SERIAL_NUMBER: &'static str = "SERIAL NUMBER";

    // Firmware update
    pub const DFU_PREPARING: &'static str = "PREPARING.";
    pub const DFU_TRANSFERRING: &'static str = "TRANSFERRING.";
    pub const DFU_VERIFIED: &'static str = "VERIFIED.";
    pub const DFU_FLASHING: &'static str = "BOOTLOADER FLASHING.";
    pub const DFU_DO_NOT_DISCONNECT: &'static str = "DO NOT DISCONNECT.";
    pub const DFU_FAILED: &'static str = "UPDATE FAILED.";
}

#[cfg(feature = "demo")]
pub const SCREEN_HOLD_TIME: u64 = 2000;
#[cfg(not(feature = "demo"))]
pub const SCREEN_HOLD_TIME: u64 = 10;
