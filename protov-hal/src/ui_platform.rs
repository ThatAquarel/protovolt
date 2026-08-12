use embassy_rp::pio::Instance;
use protov_core::model::{Channel, ChannelFocus, ConfirmState, DisplayTask, FunctionButton};
use protov_ui::appearance::ChannelAppearance;
use protov_ui::platform::UiPlatform;
use protov_ui::theme;

use crate::hal::backlight::Backlight;
use crate::hal::led::{LedsColor, LedsInterface};
use crate::scpi;

pub struct HalPlatform<'bl, 'led, PIO>
where
    PIO: Instance,
{
    backlight: Backlight<'bl>,
    leds: LedsInterface<'led, PIO>,
}

impl<'bl, 'led, PIO> HalPlatform<'bl, 'led, PIO>
where
    PIO: Instance,
{
    pub fn new(backlight: Backlight<'bl>, leds: LedsInterface<'led, PIO>) -> Self {
        Self { backlight, leds }
    }

    pub async fn refresh_leds(&mut self) {
        self.leds.refresh().await;
    }
}

impl<'bl, 'led, PIO> UiPlatform for HalPlatform<'bl, 'led, PIO>
where
    PIO: Instance,
{
    fn lcd_brightness(&self) -> u8 {
        self.backlight.level()
    }

    fn set_lcd_brightness(&mut self, level: u8) {
        self.backlight.set_brightness(level);
    }

    fn with_backlight_suppressed<R>(&mut self, f: impl FnOnce() -> R) -> R {
        self.backlight.while_suppressed(f)
    }

    fn set_backlight_off(&mut self) {
        self.backlight.turn_off();
    }

    fn serial_connected(&self) -> bool {
        scpi::serial_connected()
    }

    fn set_channel_leds<A: ChannelAppearance>(
        &mut self,
        channel: Channel,
        focus: ChannelFocus,
        appearance: &A,
    ) {
        let led_color = match focus {
            ChannelFocus::SelectedInactive | ChannelFocus::UnselectedInactive => match channel {
                Channel::A => LedsColor::ChannelA(theme::LED_OFF, theme::LED_OFF),
                Channel::B => LedsColor::ChannelB(theme::LED_OFF, theme::LED_OFF),
            },
            ChannelFocus::SelectedActive | ChannelFocus::UnselectedActive => {
                let led = appearance.channel_led(channel);
                match channel {
                    Channel::A => LedsColor::ChannelA(led, led),
                    Channel::B => LedsColor::ChannelB(led, led),
                }
            }
        };

        self.leds.update_color(led_color);
    }

    fn set_nav_button_leds<A: ChannelAppearance>(
        &mut self,
        appearance: &A,
        confirm_state: ConfirmState,
        button_state: Option<FunctionButton>,
    ) {
        let white = appearance.white_led();
        let (switch_color, settings_color) = match &button_state {
            Some(button) => match button {
                FunctionButton::Switch => (white, theme::LED_OFF),
                FunctionButton::Settings => (theme::LED_OFF, white),
                _ => (theme::LED_OFF, theme::LED_OFF),
            },
            None => (theme::LED_OFF, theme::LED_OFF),
        };
        let enter_led_color = match confirm_state {
            ConfirmState::AwaitConfirmModify(channel) => match channel {
                Some(channel) => appearance.channel_led(channel),
                None => theme::LED_OFF,
            },
            ConfirmState::AwaitModify => theme::LED_OFF,
        };

        self.leds.update_color(LedsColor::Enter(enter_led_color));
        self.leds.update_color(LedsColor::Switch(switch_color));
        self.leds.update_color(LedsColor::Settings(settings_color));
    }
}

pub fn channel_leds_need_refresh(display_task: DisplayTask) -> bool {
    matches!(
        display_task,
        DisplayTask::SetupMain(_, _, _)
            | DisplayTask::UpdateChannelFocus(_, _, _, _)
            | DisplayTask::UpdateButton(_, _)
    )
}
