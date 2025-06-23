use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{
        CornerRadii, PrimitiveStyleBuilder, Rectangle, RoundedRectangle, StrokeAlignment,
    },
    text::{Alignment, Baseline, Text, TextStyleBuilder, renderer::CharacterStyle},
};

use u8g2_fonts::{
    FontRenderer, U8g2TextStyle, fonts,
    types::{FontColor, HorizontalAlignment, VerticalPosition},
};

use crate::{
    lib::event::{Limits, PowerType, Readout},
    ui::{
        Display, Fonts, Layout,
        color_scheme::{self, FONT_SMALL},
        fmt::format_f32,
        icons_4x, labels,
    },
};

use core::fmt::Write;
use heapless::String;

use embedded_graphics_framebuf::{FrameBuf, backends::FrameBufferBackend};

pub struct Navbar;

impl Navbar {
    pub fn new() -> Self {
        Self {}
    }

    pub fn draw_power_info<D>(
        &mut self,
        target: &mut D,
        fonts: &Fonts,
        power_type: PowerType,
    ) -> Result<(), ()>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let icons = &fonts.icons_4x;

        icons
            .render_aligned(
                icons_4x::LIGHTNING,
                Point::new(0, 16),
                VerticalPosition::Center,
                HorizontalAlignment::Left,
                FontColor::Transparent(Rgb565::CSS_DARK_GRAY),
                target,
            )
            .map_err(|_| ())?;

        let font = &fonts.info_small;

        let usb_type = match power_type {
            PowerType::PowerDelivery(_) => "USB-C PD",
            PowerType::Standard(_) => "USB 2.0",
        };
        let (voltage_fmt, current_fmt) = match power_type {
            PowerType::PowerDelivery(limits) | PowerType::Standard(limits) => {
                let (mut voltage_fmt, mut current_fmt) = (
                    format_f32::<8>(limits.voltage, 2),
                    format_f32::<8>(limits.current, 2),
                );
                voltage_fmt.write_str(" V");
                current_fmt.write_str(" A");

                (voltage_fmt, current_fmt)
            }
        };
        let (voltage, current) = (voltage_fmt.as_str(), current_fmt.as_str());

        let lines = [usb_type, voltage, current];

        for (i, line) in lines.iter().enumerate() {
            font.render_aligned(
                *line,
                Point::new(36, 6 + 10 * i as i32),
                VerticalPosition::Center,
                HorizontalAlignment::Left,
                FontColor::Transparent(FONT_SMALL),
                target,
            )
            .map_err(|_| ())?;
        }

        Ok(())
    }
}
