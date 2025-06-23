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
    lib::event::Readout,
    ui::{
        Display, Fonts, Layout, color_scheme,
        labels,
    },
};

use core::fmt::Write;
use heapless::String;

use embedded_graphics_framebuf::{FrameBuf, backends::FrameBufferBackend};

pub fn format_f32<const N: usize>(value: f32, decimals: u32) -> String<N> {
    let mut buf = String::<N>::new();

    let is_negative = value.is_sign_negative();
    let abs_value = if is_negative { -value } else { value };

    let int_part = abs_value as u32;
    let mut frac_part = abs_value - (int_part as f32);

    // Scale fractional part manually (no powi)
    let mut scale = 1.0;
    for _ in 0..decimals {
        scale *= 10.0;
    }

    frac_part = frac_part * scale;
    let frac_part = frac_part as u32;

    if is_negative {
        let _ = buf.write_char('-');
    }

    if int_part < 10 {
        let _ = buf.write_char('0');
    }

    let _ = write!(buf, "{}", int_part);
    if decimals > 0 {
        let _ = buf.write_char('.');
        let _ = write!(buf, "{:0width$}", frac_part, width = decimals as usize);
    }

    buf
}
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

    pub fn draw_submeasurement<D>(target: &mut D) -> Result<(), ()>
    where
        D: Display,
    {
        let font = FontRenderer::new::<fonts::u8g2_font_logisoso16_tn>();

        font.render_aligned(
            "12.03",
            Point::new(122, 30 + 36),
            VerticalPosition::Top,
            HorizontalAlignment::Right,
            // FontColor::Transparent(Rgb565::CSS_BLACK),
            FontColor::Transparent(Rgb565::CSS_DIM_GRAY),
            target,
        )
        .map_err(|_| ())?;

        font.render_aligned(
            "5.65",
            Point::new(122, 30 + 36 + 62),
            VerticalPosition::Top,
            HorizontalAlignment::Right,
            FontColor::Transparent(Rgb565::CSS_DIM_GRAY),
            target,
        )
        .map_err(|_| ())?;

        let mode_font = FontRenderer::new::<fonts::u8g2_font_helvB08_tf>();

        mode_font
            .render_aligned(
                "SET",
                Point::new(140, 30 + 36),
                VerticalPosition::Top,
                HorizontalAlignment::Center,
                FontColor::Transparent(Rgb565::CSS_DIM_GRAY),
                target,
            )
            .map_err(|_| ())?;

        mode_font
            .render_aligned(
                "SET",
                Point::new(140, 30 + 36 + 62),
                VerticalPosition::Top,
                HorizontalAlignment::Center,
                FontColor::Transparent(Rgb565::CSS_DIM_GRAY),
                target,
            )
            .map_err(|_| ())?;

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
            let mut measurement_data = [color_scheme::BACKGROUND; ControlsScreen::MEAS_FB_SIZE];

            let mut framebuf = FrameBuf::new(
                &mut measurement_data,
                ControlsScreen::MEAS_WIDTH,
                ControlsScreen::MEAS_HEIGHT,
            );

            framebuf.clear(color_scheme::BACKGROUND);

            font.render_aligned(
                format_f32::<8>(*value, 3).as_str(),
                Point::new(ControlsScreen::MEAS_WIDTH as i32 + 1, -1),
                VerticalPosition::Top,
                HorizontalAlignment::Right,
                FontColor::Transparent(Rgb565::CSS_WHITE),
                &mut framebuf,
            )
            .map_err(|_| ())?;

            let top_left = Point::new(122 - ControlsScreen::MEAS_WIDTH as i32, 30 + 62 * i as i32);
            let area = Rectangle::new(top_left, framebuf.size());

            target
                .fill_contiguous(&area, measurement_data)
                .map_err(|_| ())?;
        }

        Ok(())
    }

    pub fn draw_power_header<D>(target: &mut D) -> Result<(), ()>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let font = FontRenderer::new::<fonts::u8g2_font_open_iconic_all_4x_t>();

        font.render_aligned(
            "\u{0060}",
            Point::new(0, 16),
            VerticalPosition::Center,
            HorizontalAlignment::Left,
            FontColor::Transparent(Rgb565::CSS_DARK_GRAY),
            target,
        )
        .map_err(|_| ())?;

        let mode_font = FontRenderer::new::<fonts::u8g2_font_helvB08_tf>();

        mode_font
            .render_aligned(
                "USB-C PD",
                Point::new(36, 6),
                VerticalPosition::Center,
                HorizontalAlignment::Left,
                FontColor::Transparent(Rgb565::CSS_DARK_GRAY),
                target,
            )
            .map_err(|_| ())?;

        mode_font
            .render_aligned(
                "20 V  5 A",
                Point::new(36, 16),
                VerticalPosition::Center,
                HorizontalAlignment::Left,
                FontColor::Transparent(Rgb565::CSS_DARK_GRAY),
                target,
            )
            .map_err(|_| ())?;

        mode_font
            .render_aligned(
                "100 W",
                Point::new(36, 26),
                VerticalPosition::Center,
                HorizontalAlignment::Left,
                FontColor::Transparent(Rgb565::CSS_DARK_GRAY),
                target,
            )
            .map_err(|_| ())?;

        Ok(())
    }

    pub fn draw_buttons<D>(target: &mut D) -> Result<(), ()>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let font = FontRenderer::new::<fonts::u8g2_font_open_iconic_all_2x_t>();
        let box_style = PrimitiveStyleBuilder::new()
            // .fill_color(Rgb565::CSS_WHITE)
            .stroke_color(Rgb565::CSS_WHITE)
            .stroke_width(2)
            .stroke_alignment(StrokeAlignment::Inside)
            .build();

        //https://github.com/olikraus/u8g2/wiki/fntgrpiconic

        let gap = 64;
        let w = 60;

        let icons = ['\u{0078}', '\u{0081}', '\u{00CC}'];

        for (i, &icon) in icons.iter().enumerate() {
            let center = 354 - (3 - i as i32) * gap;
            let left = center - w / 2;

            RoundedRectangle::new(
                Rectangle::new(Point::new(left, 0), Size::new(w as u32, 30)),
                CornerRadii::new(Size::new(10, 10)),
            )
            .into_styled(box_style)
            .draw(target)
            .map_err(|_| ())?;

            font.render_aligned(
                icon,
                Point::new(center, 15),
                VerticalPosition::Center,
                HorizontalAlignment::Center,
                FontColor::Transparent(Rgb565::CSS_WHITE),
                target,
            )
            .map_err(|_| ())?;
        }

        Ok(())
    }
}
