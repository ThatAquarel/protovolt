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
    app::SetSelect,
    lib::event::{Channel, ConfirmState, Limits, Readout},
    ui::{Display, Fonts, Layout, color_scheme, fmt::format_f32, labels},
};

use embedded_graphics_framebuf::{FrameBuf, backends::FrameBufferBackend};

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

        // Rectangle::new(Point::zero(), Size::new(r, r))
        Rectangle::new(
            Point::new(0, (text_corner_height - r) as i32),
            Size::new(r, r),
        )
        .into_styled(box_style)
        // .translate(Point::new(0, (text_corner_height - r) as i32))
        .draw(target)?;

        // Rectangle::new(Point::zero(), Size::new(r, r))
        Rectangle::new(
            Point::new((text_corner_width - r) as i32, 0),
            Size::new(r, r),
        )
        .into_styled(box_style)
        // .translate(Point::new((text_corner_width - r) as i32, 0))
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
            FontColor::Transparent(color_scheme::BACKGROUND),
            target,
        )
        .map_err(|_| ())?;

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
            let mut fbuf_data = [color_scheme::BACKGROUND; ControlsScreen::MEAS_FB_SIZE];
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
                FontColor::Transparent(color_scheme::FONT_MAIN),
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
    const SUBMEAS_HEIGHT: usize = ControlsScreen::MEAS_HEIGHT / 2;
    const SUBMEAS_FB_SIZE: usize = ControlsScreen::SUBMEAS_WIDTH * ControlsScreen::SUBMEAS_HEIGHT;

    pub fn draw_submeasurements<D>(
        &mut self,
        target: &mut D,
        fonts: &Fonts,
        set_select: Option<SetSelect>,
        limits: Limits,
        channel: Channel,
        confirm_state: ConfirmState,
    ) -> Result<(), ()>
    where
        D: Display,
    {
        let font = &fonts.readout_small;

        let select_index = match set_select {
            Some(SetSelect::Voltage) => Some(0),
            Some(SetSelect::Current) => Some(1),
            _ => None,
        };

        let await_confirm_modify_color = match channel {
            Channel::A => color_scheme::CH_A_SELECTED,
            Channel::B => color_scheme::CH_B_SELECTED,
        };

        let values = [limits.voltage, limits.current];
        for (i, value) in values.iter().enumerate() {
            let mut fbuf_data = [color_scheme::BACKGROUND; ControlsScreen::SUBMEAS_FB_SIZE];
            let mut fbuf = FrameBuf::new(
                &mut fbuf_data,
                ControlsScreen::SUBMEAS_WIDTH,
                ControlsScreen::SUBMEAS_HEIGHT,
            );

            let color = if select_index == Some(i) {
                match confirm_state {
                    ConfirmState::AwaitConfirmModify => await_confirm_modify_color,
                    ConfirmState::AwaitModify => color_scheme::SELECTED,
                }
            } else {
                color_scheme::UNSELECTED
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
        set_select: Option<SetSelect>,
        top_tag: &'static str,
        bottom_tag: &'static str,
        channel: Channel,
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

        let await_confirm_modify_color = match channel {
            Channel::A => color_scheme::CH_A_SELECTED,
            Channel::B => color_scheme::CH_B_SELECTED,
        };

        for (i, tag) in tags.iter().enumerate() {
            let mut fbuf_data = [color_scheme::BACKGROUND; ControlsScreen::TAG_FB_SIZE];
            let mut fbuf = FrameBuf::new(
                &mut fbuf_data,
                ControlsScreen::TAG_WIDTH,
                ControlsScreen::TAG_HEIGHT,
            );

            let color = if select_index == Some(i) {
                match confirm_state {
                    ConfirmState::AwaitConfirmModify => await_confirm_modify_color,
                    ConfirmState::AwaitModify => color_scheme::SELECTED,
                }
            } else {
                color_scheme::UNSELECTED
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
