use protov_core::model::{Channel, ChannelFocus, ConfirmState, FunctionButton};

use crate::appearance::ChannelAppearance;

/// Hardware-specific UI side effects (backlight, RGB LEDs, host link indicator).
pub trait UiPlatform {
    fn lcd_brightness(&self) -> u8;
    fn set_lcd_brightness(&mut self, level: u8);
    fn with_backlight_suppressed<R>(&mut self, f: impl FnOnce() -> R) -> R;
    fn set_backlight_off(&mut self);
    fn serial_connected(&self) -> bool;
    fn set_channel_leds<A: ChannelAppearance>(
        &mut self,
        channel: Channel,
        focus: ChannelFocus,
        appearance: &A,
    );
    fn set_nav_button_leds<A: ChannelAppearance>(
        &mut self,
        appearance: &A,
        confirm_state: ConfirmState,
        button_state: Option<FunctionButton>,
    );
}

/// No-op platform for host simulation and tests.
pub struct NullPlatform {
    brightness: u8,
    serial_connected: bool,
}

impl NullPlatform {
    pub const fn new() -> Self {
        Self {
            brightness: 0,
            serial_connected: false,
        }
    }

    pub fn with_serial_connected(serial_connected: bool) -> Self {
        Self {
            brightness: 0,
            serial_connected,
        }
    }
}

impl Default for NullPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl UiPlatform for NullPlatform {
    fn lcd_brightness(&self) -> u8 {
        self.brightness
    }

    fn set_lcd_brightness(&mut self, level: u8) {
        self.brightness = level;
    }

    fn with_backlight_suppressed<R>(&mut self, f: impl FnOnce() -> R) -> R {
        f()
    }

    fn set_backlight_off(&mut self) {}

    fn serial_connected(&self) -> bool {
        self.serial_connected
    }

    fn set_channel_leds<A: ChannelAppearance>(
        &mut self,
        _channel: Channel,
        _focus: ChannelFocus,
        _appearance: &A,
    ) {
    }

    fn set_nav_button_leds<A: ChannelAppearance>(
        &mut self,
        _appearance: &A,
        _confirm_state: ConfirmState,
        _button_state: Option<FunctionButton>,
    ) {
    }
}
