use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{
        CornerRadii, PrimitiveStyleBuilder, Rectangle, RoundedRectangle, StrokeAlignment,
    },
};

use u8g2_fonts::types::{FontColor, HorizontalAlignment, VerticalPosition};

use crate::{
    hal::event::{ConfirmState, FunctionButton, Limits, PowerType},
    ui::{Fonts, color_scheme, icons_2x, navbar_layout},
};
use protov_core::fmt::{format_navbar_status_line, format_navbar_va_line};

/// Input power summary for the navbar (power contract + host serial link).
#[derive(Clone, Copy, Debug)]
pub struct PowerInfoDisplay {
    pub power_type: PowerType,
    pub serial_connected: bool,
}

impl PowerInfoDisplay {
    pub fn new(power_type: PowerType, serial_connected: bool) -> Self {
        Self {
            power_type,
            serial_connected,
        }
    }

    fn limits(self) -> Limits {
        match self.power_type {
            PowerType::PowerDelivery(limits) | PowerType::Standard(limits) => limits,
        }
    }

    fn icon(self) -> &'static str {
        if self.serial_connected {
            icons_2x::LINK
        } else {
            icons_2x::LIGHTNING
        }
    }

    fn status_line(self) -> heapless::String<16> {
        format_navbar_status_line(self.power_type, self.serial_connected)
    }
}

pub struct Navbar;

impl Navbar {
    pub fn new() -> Self {
        Self {}
    }

    pub fn clear_power_info<D>(&self, target: &mut D) -> Result<(), ()>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        Rectangle::new(
            Point::new(0, 0),
            Size::new(navbar_layout::BOX_WIDTH, navbar_layout::BOX_HEIGHT),
        )
        .into_styled(
            PrimitiveStyleBuilder::new()
                .fill_color(color_scheme::BACKGROUND)
                .build(),
        )
        .draw(target)
        .map_err(|_| ())
    }

    pub fn draw_power_info<D>(
        &mut self,
        target: &mut D,
        fonts: &Fonts,
        info: PowerInfoDisplay,
    ) -> Result<(), ()>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        self.clear_power_info(target)?;

        let box_style = PrimitiveStyleBuilder::new()
            .stroke_color(color_scheme::UNSELECTED)
            .stroke_width(navbar_layout::BOX_STROKE_WIDTH)
            .stroke_alignment(StrokeAlignment::Inside)
            .build();

        RoundedRectangle::new(
            Rectangle::new(
                Point::new(0, 0),
                Size::new(navbar_layout::BOX_WIDTH, navbar_layout::BOX_HEIGHT),
            ),
            CornerRadii::new(Size::new(10, 10)),
        )
        .into_styled(box_style)
        .draw(target)
        .map_err(|_| ())?;

        fonts
            .icons_2x
            .render_aligned(
                info.icon(),
                navbar_layout::ICON_CENTER,
                VerticalPosition::Center,
                HorizontalAlignment::Center,
                FontColor::Transparent(color_scheme::NAVBAR_TEXT),
                target,
            )
            .map_err(|_| ())?;

        let status_line = info.status_line();
        let va_line = format_navbar_va_line(info.limits());
        let lines = [status_line.as_str(), va_line.as_str()];

        for (i, line) in lines.iter().enumerate() {
            fonts
                .info_navbar
                .render_aligned(
                    *line,
                    Point::new(
                        navbar_layout::TEXT_X,
                        navbar_layout::TEXT_Y + navbar_layout::TEXT_LINE_GAP * i as i32,
                    ),
                    VerticalPosition::Center,
                    HorizontalAlignment::Left,
                    FontColor::Transparent(color_scheme::NAVBAR_TEXT),
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
            ConfirmState::AwaitModify => icons_2x::PENCIL,
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

            let current_style = box_style.stroke_color(color).build();

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
