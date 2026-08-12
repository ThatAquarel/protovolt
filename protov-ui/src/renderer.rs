use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Drawable, Point, Primitive, Size},
    primitives::{PrimitiveStyle, Rectangle},
};

use protov_core::model::{
    Channel, ChannelFocus, ChannelHardwareState, ConfirmState, DecimalPrecision, DfuStatus,
    FunctionButton, Limits, PowerType, Readout, SetSelect, SetState,
};

use crate::{
    appearance::{ChannelAppearance, channel_focus_color},
    boot::BootScreen,
    controls::ControlsScreen,
    fonts::Fonts,
    geometry::DisplayGeometry,
    labels,
    layout::Layout,
    navbar::{Navbar, PowerInfoDisplay},
    platform::UiPlatform,
    settings::SettingsScreen,
    theme::{self, settings_layout},
};

pub struct UiRenderer<'a, D> {
    pub target: &'a mut D,
    pub fonts: Fonts,
    pub layout: Layout,

    boot: BootScreen<'a>,
    controls: ControlsScreen,
    settings: SettingsScreen,
    navbar: Navbar,

    settings_visible: bool,
    dfu_screen_active: bool,
}

impl<'a, D> UiRenderer<'a, D>
where
    D: DrawTarget<Color = Rgb565>,
{
    pub fn new(target: &'a mut D, geometry: DisplayGeometry) -> Self {
        Self {
            target,
            fonts: Fonts::default(),
            layout: Layout::new(geometry),
            boot: BootScreen::new(geometry),
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
        .into_styled(PrimitiveStyle::with_fill(theme::BACKGROUND))
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
            .draw_background(&mut target, theme::SELECTED)
            .map_err(|_| ())?;
        self.controls
            .draw_header_text(&mut target, labels::SETTINGS)?;
        self.settings.draw(&mut target, &self.fonts, width)
    }

    pub fn clear<P: UiPlatform>(&mut self, platform: &mut P) -> Result<(), ()> {
        platform.with_backlight_suppressed(|| {
            self.target.clear(theme::BACKGROUND).map_err(|_| ())
        })
    }

    #[cfg(feature = "demo")]
    pub fn boot_demo_mode(&mut self) -> Result<(), ()> {
        self.boot
            .draw_demo_screen(&mut *self.target, &self.layout, &self.fonts)
    }

    pub fn boot_splash_screen(&mut self) -> Result<(), ()> {
        self.boot
            .draw_splash_screen(&mut *self.target, &self.layout)
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
            &self.layout,
            &self.fonts,
            index,
            title,
            subtitle,
            valid,
        )
    }

    pub fn draw_dfu_status<P: UiPlatform>(
        &mut self,
        status: DfuStatus,
        platform: &mut P,
    ) -> Result<(), ()> {
        match status {
            DfuStatus::Idle => {
                self.dfu_screen_active = false;
                Ok(())
            }
            DfuStatus::Preparing { .. } => {
                self.dfu_screen_active = true;
                self.clear(platform)?;
                self.boot
                    .draw_splash_screen(&mut *self.target, &self.layout)?;
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
                platform.set_backlight_off();
                self.boot
                    .draw_dfu_text(&mut *self.target, &self.fonts, status)
            }
        }
    }

    pub fn controls_channel_box<A: ChannelAppearance, P: UiPlatform>(
        &mut self,
        appearance: &A,
        platform: &mut P,
        channel: Channel,
        focus: ChannelFocus,
    ) -> Result<(), ()> {
        let color = channel_focus_color(appearance, channel, focus);

        let text = match channel {
            Channel::A => labels::CHANNEL_A,
            Channel::B => labels::CHANNEL_B,
        };

        let mut target = self.layout.channel_section(&mut *self.target, channel);
        self.controls
            .draw_channel_background(&mut target, color)
            .map_err(|_| ())?;
        self.controls.draw_header_text(&mut target, text)?;

        platform.set_channel_leds(channel, focus, appearance);

        Ok(())
    }

    pub fn controls_header_chip<A: ChannelAppearance>(
        &mut self,
        appearance: &A,
        channel: Channel,
        focus: ChannelFocus,
        channel_hardware_state: ChannelHardwareState,
    ) -> Result<(), ()> {
        let mut target = self.layout.channel_section(&mut *self.target, channel);
        let color = channel_focus_color(appearance, channel, focus);
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

    pub fn controls_submeasurement<A: ChannelAppearance>(
        &mut self,
        appearance: &A,
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
            appearance.selected_color(channel),
            set_select,
            limits,
            confirm_state,
            select_precision,
        )
    }

    pub fn controls_submeasurement_tag<A: ChannelAppearance>(
        &mut self,
        appearance: &A,
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
            appearance.selected_color(channel),
            set_select,
            top_tag,
            bottom_tag,
            confirm_state,
        )
    }

    pub fn nav_power_info<P: UiPlatform>(
        &mut self,
        power_type: PowerType,
        platform: &P,
    ) -> Result<(), ()> {
        let info = PowerInfoDisplay::new(power_type, platform.serial_connected());
        self.navbar
            .draw_power_info(&mut *self.target, &self.fonts, info)
    }

    pub fn nav_buttons<A: ChannelAppearance, P: UiPlatform>(
        &mut self,
        appearance: &A,
        platform: &mut P,
        confirm_state: ConfirmState,
        button_state: Option<FunctionButton>,
    ) -> Result<(), ()> {
        self.navbar.draw_button(
            &mut *self.target,
            &self.fonts,
            confirm_state,
            button_state,
        )?;

        platform.set_nav_button_leds(appearance, confirm_state, button_state);

        Ok(())
    }
}
