use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{
        CornerRadii, PrimitiveStyleBuilder, Rectangle, RoundedRectangle, StrokeAlignment,
        StyledDrawable,
    },
};
use embedded_graphics_framebuf::FrameBuf;
use protov_core::fmt::format_f32;
use protov_core::model::{ConfirmState, DecimalPrecision, Limits, Readout, SetSelect};
use u8g2_fonts::{
    FontRenderer, fonts,
    types::{FontColor, HorizontalAlignment, VerticalPosition},
};

use crate::{
    display::Display,
    fonts::Fonts,
    labels,
    theme::{self, icons_1x},
};

pub struct ControlsScreen;

impl ControlsScreen {
    pub fn new() -> Self {
        Self {}
    }

    pub fn draw_channel_background<D>(
        &mut self,
        target: &mut D,
        color: Rgb565,
    ) -> Result<(), D::Error>
    where
        D: Display,
    {
        let r: u32 = 10;
        let (text_corner_width, text_corner_height): (u32, u32) = (75, 20);
        let channel_box_size = Size::new(157, 200);

        let channel_style = PrimitiveStyleBuilder::new()
            .stroke_color(color)
            .stroke_width(2)
            .stroke_alignment(StrokeAlignment::Inside);

        let outline_style = channel_style.build();
        let box_style = channel_style.fill_color(color).build();

        RoundedRectangle::new(
            Rectangle::new(Point::zero(), channel_box_size),
            CornerRadii::new(Size::new(r, r)),
        )
        .into_styled(outline_style)
        .draw(target)?;

        RoundedRectangle::new(
            Rectangle::new(
                Point::zero(),
                Size::new(text_corner_width, text_corner_height),
            ),
            CornerRadii::new(Size::new(r, r)),
        )
        .into_styled(box_style)
        .draw(target)?;

        Rectangle::new(
            Point::new(0, (text_corner_height - r) as i32),
            Size::new(r, r),
        )
        .into_styled(box_style)
        .draw(target)?;

        Rectangle::new(
            Point::new((text_corner_width - r) as i32, 0),
            Size::new(r, r),
        )
        .into_styled(box_style)
        .draw(target)
    }

    pub fn draw_header_text<D>(&mut self, target: &mut D, text: &'static str) -> Result<(), ()>
    where
        D: Display,
    {
        let font = FontRenderer::new::<fonts::u8g2_font_helvB08_tf>();

        font.render_aligned(
            text,
            Point::new(6, 11),
            VerticalPosition::Center,
            HorizontalAlignment::Left,
            FontColor::Transparent(theme::BACKGROUND),
            target,
        )
        .map_err(|_| ())?;

        Ok(())
    }

    const HEADER_WIDTH: usize = 64;
    const HEADER_HEIGHT: usize = 8;
    const HEADER_BOX_WIDTH: usize = (ControlsScreen::HEADER_WIDTH + 2);
    const HEADER_BOX_HEIGHT: usize = (ControlsScreen::HEADER_HEIGHT + 2);
    const HEADER_FB_SIZE: usize =
        ControlsScreen::HEADER_BOX_WIDTH * ControlsScreen::HEADER_BOX_HEIGHT;

    pub fn draw_header_chip<D>(
        &mut self,
        target: &mut D,
        fonts: &Fonts,
        invert: bool,
        color: Rgb565,
        text: &'static str,
    ) -> Result<(), ()>
    where
        D: Display,
    {
        let (fg_color, bg_color) = match invert {
            false => (color, Rgb565::BLACK),
            true => (Rgb565::BLACK, color),
        };

        let mut fbuf_data = [theme::BACKGROUND; ControlsScreen::HEADER_FB_SIZE];
        let mut fbuf = FrameBuf::new(
            &mut fbuf_data,
            ControlsScreen::HEADER_BOX_WIDTH,
            ControlsScreen::HEADER_BOX_HEIGHT,
        );

        let mode_font = &fonts.info_small;

        let (pos, vpos, halign) = (
            Point::new(ControlsScreen::HEADER_WIDTH as i32, 0),
            VerticalPosition::Top,
            HorizontalAlignment::Right,
        );

        let bbox = mode_font.get_rendered_dimensions_aligned(text, pos, vpos, halign);

        if let Ok(Some(rect)) = bbox {
            let style = PrimitiveStyleBuilder::new()
                .stroke_alignment(StrokeAlignment::Inside)
                .stroke_width(1)
                .stroke_color(bg_color)
                .fill_color(bg_color)
                .build();

            rect.offset(1).draw_styled(&style, &mut fbuf);
        }

        mode_font
            .render_aligned(
                text,
                pos,
                vpos,
                halign,
                FontColor::Transparent(fg_color),
                &mut fbuf,
            )
            .map_err(|_| ())?;

        let rect = Rectangle::new(
            Point::new(83, 9),
            Size::new(
                ControlsScreen::HEADER_BOX_WIDTH as u32,
                ControlsScreen::HEADER_BOX_HEIGHT as u32,
            ),
        );

        target.fill_contiguous(&rect, fbuf_data).map_err(|_| ())?;

        Ok(())
    }

    pub fn draw_units<D>(&mut self, target: &mut D, fonts: &Fonts) -> Result<(), ()>
    where
        D: Display,
    {
        let font = &fonts.info_large;
        let units = [labels::VOLT, labels::AMPERE, labels::WATT];

        for (i, unit) in units.iter().enumerate() {
            font.render_aligned(
                *unit,
                Point::new(140, 30 + 62 * i as i32),
                VerticalPosition::Top,
                HorizontalAlignment::Center,
                FontColor::Transparent(Rgb565::CSS_WHITE),
                target,
            )
            .map_err(|_| ())?;
        }

        Ok(())
    }

    const MEAS_WIDTH: usize = 108;
    const MEAS_HEIGHT: usize = 32;
    const MEAS_FB_SIZE: usize = ControlsScreen::MEAS_WIDTH * ControlsScreen::MEAS_HEIGHT;

    pub fn draw_measurements<D>(
        &mut self,
        target: &mut D,
        fonts: &Fonts,
        readout: Readout,
    ) -> Result<(), ()>
    where
        D: Display,
    {
        let font = &fonts.readout_large;
        let readouts = [readout.voltage, readout.current, readout.power];

        for (i, value) in readouts.iter().enumerate() {
            let mut fbuf_data = [theme::BACKGROUND; ControlsScreen::MEAS_FB_SIZE];
            let mut fbuf = FrameBuf::new(
                &mut fbuf_data,
                ControlsScreen::MEAS_WIDTH,
                ControlsScreen::MEAS_HEIGHT,
            );

            font.render_aligned(
                format_f32::<6>(*value, 3).as_str(),
                Point::new(ControlsScreen::MEAS_WIDTH as i32 + 1, -1),
                VerticalPosition::Top,
                HorizontalAlignment::Right,
                FontColor::Transparent(theme::FONT_MAIN),
                &mut fbuf,
            )
            .map_err(|_| ())?;

            let top_left = Point::new(122 - ControlsScreen::MEAS_WIDTH as i32, 30 + 62 * i as i32);
            let area = Rectangle::new(top_left, fbuf.size());

            target.fill_contiguous(&area, fbuf_data).map_err(|_| ())?;
        }

        Ok(())
    }

    const SUBMEAS_WIDTH: usize = ControlsScreen::MEAS_WIDTH / 2;
    const SUBMEAS_HEIGHT: usize = ControlsScreen::MEAS_HEIGHT / 2 + 8;
    const SUBMEAS_FB_SIZE: usize = ControlsScreen::SUBMEAS_WIDTH * ControlsScreen::SUBMEAS_HEIGHT;

    pub fn draw_submeasurements<D>(
        &mut self,
        target: &mut D,
        fonts: &Fonts,
        channel_accent: Rgb565,
        set_select: Option<SetSelect>,
        limits: Limits,
        confirm_state: ConfirmState,
        select_precision: Option<DecimalPrecision>,
    ) -> Result<(), ()>
    where
        D: Display,
    {
        let (font, select_font) = (&fonts.readout_small, &fonts.icons_1x);

        let select_index = match set_select {
            Some(SetSelect::Voltage) => Some(0),
            Some(SetSelect::Current) => Some(1),
            _ => None,
        };

        let values = [limits.voltage, limits.current];
        for (i, value) in values.iter().enumerate() {
            let mut fbuf_data = [theme::BACKGROUND; ControlsScreen::SUBMEAS_FB_SIZE];
            let mut fbuf = FrameBuf::new(
                &mut fbuf_data,
                ControlsScreen::SUBMEAS_WIDTH,
                ControlsScreen::SUBMEAS_HEIGHT,
            );

            let selected = select_index == Some(i);

            let color = if selected {
                match confirm_state {
                    ConfirmState::AwaitConfirmModify(_) => channel_accent,
                    ConfirmState::AwaitModify => theme::SELECTED,
                }
            } else {
                theme::UNSELECTED
            };

            font.render_aligned(
                format_f32::<5>(*value, 2).as_str(),
                Point::new(ControlsScreen::SUBMEAS_WIDTH as i32, -1),
                VerticalPosition::Top,
                HorizontalAlignment::Right,
                FontColor::Transparent(color),
                &mut fbuf,
            )
            .map_err(|_| ())?;

            if selected {
                if let Some(precision) = &select_precision {
                    let exp = precision.get_exponent();
                    let digit_index = (exp + 2) as i32;
                    let offset = if digit_index < 2 { 4 } else { 10 };
                    let padding_right: i32 = digit_index * 10 + offset;

                    select_font
                        .render_aligned(
                            icons_1x::UP_ARROW_THICK,
                            Point::new(ControlsScreen::SUBMEAS_WIDTH as i32 - padding_right, 15),
                            VerticalPosition::Top,
                            HorizontalAlignment::Center,
                            FontColor::Transparent(color),
                            &mut fbuf,
                        )
                        .map_err(|_| ())?;
                }
            }

            let top_left = Point::new(
                122 - ControlsScreen::SUBMEAS_WIDTH as i32,
                30 + 36 + 62 * i as i32,
            );
            let area = Rectangle::new(top_left, fbuf.size());

            target.fill_contiguous(&area, fbuf_data).map_err(|_| ())?;
        }

        Ok(())
    }

    const TAG_WIDTH: usize = 22;
    const TAG_HEIGHT: usize = 8;
    const TAG_FB_SIZE: usize = ControlsScreen::TAG_WIDTH * ControlsScreen::TAG_HEIGHT;

    pub fn draw_submeasurements_tag<D>(
        &mut self,
        target: &mut D,
        fonts: &Fonts,
        channel_accent: Rgb565,
        set_select: Option<SetSelect>,
        top_tag: &'static str,
        bottom_tag: &'static str,
        confirm_state: ConfirmState,
    ) -> Result<(), ()>
    where
        D: Display,
    {
        let mode_font = &fonts.info_small;
        let tags = [top_tag, bottom_tag];

        let select_index = match set_select {
            Some(SetSelect::Voltage) => Some(0),
            Some(SetSelect::Current) => Some(1),
            _ => None,
        };

        for (i, tag) in tags.iter().enumerate() {
            let mut fbuf_data = [theme::BACKGROUND; ControlsScreen::TAG_FB_SIZE];
            let mut fbuf = FrameBuf::new(
                &mut fbuf_data,
                ControlsScreen::TAG_WIDTH,
                ControlsScreen::TAG_HEIGHT,
            );

            let color = if select_index == Some(i) {
                match confirm_state {
                    ConfirmState::AwaitConfirmModify(_) => channel_accent,
                    ConfirmState::AwaitModify => theme::SELECTED,
                }
            } else {
                theme::UNSELECTED
            };

            mode_font
                .render_aligned(
                    *tag,
                    Point::new(ControlsScreen::TAG_WIDTH as i32 / 2, -1),
                    VerticalPosition::Top,
                    HorizontalAlignment::Center,
                    FontColor::Transparent(color),
                    &mut fbuf,
                )
                .map_err(|_| ())?;

            let top_left = Point::new(
                140 - ControlsScreen::TAG_WIDTH as i32 / 2,
                30 + 36 + 62 * i as i32,
            );
            let area = Rectangle::new(top_left, fbuf.size());

            target.fill_contiguous(&area, fbuf_data).map_err(|_| ())?;
        }

        Ok(())
    }
}
