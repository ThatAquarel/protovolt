use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{
        CornerRadii, PrimitiveStyleBuilder, Rectangle, RoundedRectangle, StrokeAlignment,
    },
};

use protov_core::config::{FIRMWARE_REVISION, HARDWARE_REVISION, SERIAL_NUMBER};
use u8g2_fonts::types::{FontColor, HorizontalAlignment, VerticalPosition};

use crate::ui::{Display, Fonts, color_scheme, labels, settings_layout};

pub struct SettingsScreen;

impl SettingsScreen {
    pub fn new() -> Self {
        Self
    }

    pub fn draw_background<D>(&self, target: &mut D, color: Rgb565) -> Result<(), D::Error>
    where
        D: Display,
    {
        let channel_style = PrimitiveStyleBuilder::new()
            .stroke_color(color)
            .stroke_width(2)
            .stroke_alignment(StrokeAlignment::Inside);

        let outline_style = channel_style.build();
        let box_style = channel_style.fill_color(color).build();

        RoundedRectangle::new(
            Rectangle::new(Point::new(0, 0), Size::new(157 + 163, 200)),
            CornerRadii::new(Size::new(10, 10)),
        )
        .into_styled(outline_style)
        .draw(target)?;

        let rounded_rect_end_x: u32 = 64;

        RoundedRectangle::new(
            Rectangle::new(Point::new(0, 0), Size::new(rounded_rect_end_x, 20)),
            CornerRadii::new(Size::new(10, 10)),
        )
        .into_styled(box_style)
        .draw(target)?;

        Rectangle::new(Point::new(0, 0), Size::new(10, 10))
            .into_styled(box_style)
            .translate(Point::new(0, 20 - 10))
            .draw(target)?;

        Rectangle::new(Point::new(0, 0), Size::new(10, 10))
            .into_styled(box_style)
            .translate(Point::new(rounded_rect_end_x as i32 - 10, 0))
            .draw(target)
    }

    pub fn draw<D>(&self, target: &mut D, fonts: &Fonts, screen_width: u32) -> Result<(), ()>
    where
        D: Display,
    {
        let y_offset = settings_layout::Y_OFFSET;
        let x_offset_text = settings_layout::X_OFFSET_TEXT;
        let y_skip = settings_layout::Y_SKIP;

        let label_color = color_scheme::SELECTED;
        let value_color = color_scheme::FONT_MAIN;

        fonts
            .info_small
            .render_aligned(
                labels::MANAGE_AT,
                Point::new(x_offset_text, y_offset - 5 * y_skip / 2),
                VerticalPosition::Center,
                HorizontalAlignment::Left,
                FontColor::Transparent(label_color),
                target,
            )
            .map_err(|_| ())?;

        fonts
            .readout_small
            .render_aligned(
                labels::WEBSITE,
                Point::new(
                    screen_width as i32 - x_offset_text,
                    y_offset - 5 * y_skip / 2,
                ),
                VerticalPosition::Center,
                HorizontalAlignment::Right,
                FontColor::Transparent(value_color),
                target,
            )
            .map_err(|_| ())?;

        let keys = [
            labels::FW_VERSION,
            labels::HW_VERSION,
            labels::SERIAL_NUMBER,
        ];

        let values = [FIRMWARE_REVISION, HARDWARE_REVISION, SERIAL_NUMBER];

        for i in 0..values.len() {
            let current_y = i as i32 * y_skip + y_offset;

            fonts
                .info_small
                .render_aligned(
                    keys[i],
                    Point::new(x_offset_text, current_y),
                    VerticalPosition::Center,
                    HorizontalAlignment::Left,
                    FontColor::Transparent(label_color),
                    target,
                )
                .map_err(|_| ())?;

            fonts
                .readout_small
                .render_aligned(
                    values[i],
                    Point::new(screen_width as i32 - x_offset_text, current_y),
                    VerticalPosition::Center,
                    HorizontalAlignment::Right,
                    FontColor::Transparent(value_color),
                    target,
                )
                .map_err(|_| ())?;
        }

        Ok(())
    }

    pub fn draw_overlay<D>(
        &self,
        target: &mut D,
        fonts: &Fonts,
        screen_width: u32,
    ) -> Result<(), ()>
    where
        D: Display,
    {
        self.draw_background(target, color_scheme::SELECTED)
            .map_err(|_| ())?;
        self.draw(target, fonts, screen_width)
    }
}
