use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{
        CornerRadii, PrimitiveStyleBuilder, Rectangle, RoundedRectangle, StrokeAlignment,
    },
};

use u8g2_fonts::types::{FontColor, HorizontalAlignment, VerticalPosition};

use crate::{
    hal::event::{ConfirmState, FunctionButton, PowerType},
    ui::{
        Fonts,
        color_scheme::{self},
        fmt::format_f32,
        icons_2x, icons_4x, labels,
    },
};

use core::fmt::Write;


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
                FontColor::Transparent(color_scheme::UNSELECTED),
                target,
            )
            .map_err(|_| ())?;

        let font = &fonts.info_small;

        let usb_type = match power_type {
            PowerType::PowerDelivery(_) => labels::PD,
            PowerType::Standard(_) => labels::STD,
        };
        let (voltage_fmt, current_fmt) = match power_type {
            PowerType::PowerDelivery(limits) | PowerType::Standard(limits) => {
                let (mut voltage_fmt, mut current_fmt) = (
                    format_f32::<8>(limits.voltage, 2),
                    format_f32::<8>(limits.current, 2),
                );
                voltage_fmt.write_char(' ').unwrap();
                voltage_fmt.write_str(labels::VOLT).unwrap();
                current_fmt.write_char(' ').unwrap();
                current_fmt.write_str(labels::AMPERE).unwrap();

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
                FontColor::Transparent(color_scheme::FONT_SMALL),
                target,
            )
            .map_err(|_| ())?;
        }

        Ok(())
    }

    pub fn draw_button<D>(
        &mut self,
        target: &mut D,
        fonts: &Fonts,
        confirm_state: ConfirmState,
        button_state: Option<FunctionButton>,
    ) -> Result<(), ()>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let icons = &fonts.icons_2x;

        let box_style = PrimitiveStyleBuilder::new()
            .stroke_width(2)
            .stroke_alignment(StrokeAlignment::Inside)
            .fill_color(color_scheme::BACKGROUND);

        let gap = 64;
        let w = 60;

        let enter_icon = match confirm_state {
            ConfirmState::AwaitConfirmModify(_) => icons_2x::CHECKMARK,
            ConfirmState::AwaitModify=> icons_2x::PENCIL,
        };
        let buttons = [enter_icon, icons_2x::SWITCH, icons_2x::SETTINGS];

        let selected_index = match button_state {
            Some(FunctionButton::Enter) => Some(0),
            Some(FunctionButton::Switch) => Some(1),
            Some(FunctionButton::Settings) => Some(2),
            _ => None,
        };

        for (i, &icon) in buttons.iter().enumerate() {
            let center = 354 - (3 - i as i32) * gap;
            let left = center - w / 2;

            let color = if selected_index == Some(i) {
                color_scheme::SELECTED
            } else {
                color_scheme::UNSELECTED
            };

            let current_style = box_style
                .stroke_color(color)
                .build();

            RoundedRectangle::new(
                Rectangle::new(Point::new(left, 0), Size::new(w as u32, 30)),
                CornerRadii::new(Size::new(10, 10)),
            )
            .into_styled(current_style)
            .draw(target)
            .map_err(|_| ())?;

            icons
                .render_aligned(
                    icon,
                    Point::new(center, 15),
                    VerticalPosition::Center,
                    HorizontalAlignment::Center,
                    FontColor::Transparent(color),
                    target,
                )
                .map_err(|_| ())?;
        }

        Ok(())
    }
}
