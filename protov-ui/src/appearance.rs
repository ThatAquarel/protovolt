use embedded_graphics::pixelcolor::Rgb565;
use protov_core::model::Channel;
use protov_core::scpi::state::ScpiState;
use smart_leds::RGB8;

/// Channel colors and LED tints derived from SCPI appearance state.
pub trait ChannelAppearance {
    fn selected_color(&self, channel: Channel) -> Rgb565;
    fn unselected_color(&self, channel: Channel) -> Rgb565;
    fn channel_led(&self, channel: Channel) -> RGB8;
    fn white_led(&self) -> RGB8;
}

impl ChannelAppearance for ScpiState {
    fn selected_color(&self, channel: Channel) -> Rgb565 {
        self.selected_rgb565(channel)
    }

    fn unselected_color(&self, channel: Channel) -> Rgb565 {
        self.unselected_rgb565(channel)
    }

    fn channel_led(&self, channel: Channel) -> RGB8 {
        self.channel_led(channel)
    }

    fn white_led(&self) -> RGB8 {
        self.white_led()
    }
}

pub fn channel_focus_color<A: ChannelAppearance>(
    appearance: &A,
    channel: Channel,
    focus: protov_core::model::ChannelFocus,
) -> Rgb565 {
    use protov_core::model::ChannelFocus;
    use crate::theme;

    match focus {
        ChannelFocus::SelectedInactive => theme::SELECTED,
        ChannelFocus::UnselectedInactive => theme::UNSELECTED,
        ChannelFocus::SelectedActive => appearance.selected_color(channel),
        ChannelFocus::UnselectedActive => appearance.unselected_color(channel),
    }
}
