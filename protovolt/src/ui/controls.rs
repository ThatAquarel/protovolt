use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{
        CornerRadii, PrimitiveStyleBuilder, Rectangle, RoundedRectangle,
        StrokeAlignment,
    },
    text::{
        Alignment, Baseline, Text, TextStyleBuilder,
        renderer::CharacterStyle,
    },
};

use u8g2_fonts::{
    FontRenderer, U8g2TextStyle, fonts,
    types::{FontColor, HorizontalAlignment, VerticalPosition},
};

use crate::{lib::event::Readout, ui::Display};

pub fn clear<D>(target: &mut D) -> Result<(), D::Error>
where
    D: Display,
{
    target.clear(Rgb565::BLACK)
}

pub fn draw_channel_background<D>(target: &mut D, color: Rgb565) -> Result<(), D::Error>
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
        Rectangle::new(Point::new(0, 0), Size::new(157, 200)),
        CornerRadii::new(Size::new(10, 10)),
    )
    .into_styled(outline_style)
    .draw(target)?;

    RoundedRectangle::new(
        Rectangle::new(Point::new(0, 0), Size::new(75, 20)),
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
        .translate(Point::new(75 - 10, 0))
        .draw(target)
}

pub fn draw_header_text<D>(target: &mut D, text: &'static str) -> Result<(), ()>
where
    D: Display,
{
    let font = FontRenderer::new::<fonts::u8g2_font_helvB08_tf>();

    font.render_aligned(
        text,
        Point::new(6, 11),
        VerticalPosition::Center,
        HorizontalAlignment::Left,
        FontColor::Transparent(Rgb565::CSS_BLACK),
        target,
    )
    .map_err(|_| ())?;

    Ok(())
}

pub fn draw_units<D>(target: &mut D) -> Result<(), ()>
where
    D: Display,
{
    let font = FontRenderer::new::<fonts::u8g2_font_helvR14_tr>();

    let units = ["V", "A", "W"];

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

use core::fmt::Write;
use heapless::String;

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

use embedded_graphics_framebuf::FrameBuf;

pub fn draw_measurements<D>(target: &mut D, readout: Readout) -> Result<(), ()>
where
    D: Display,
{
    // let mut style = U8g2TextStyle::new(fonts::u8g2_font_logisoso32_tn, Rgb565::CSS_WHITE);

    let font = FontRenderer::new::<fonts::u8g2_font_logisoso32_tn>();


    // let align = TextStyleBuilder::new()
    //     .alignment(Alignment::Left)
    //     .baseline(Baseline::Top)
    //     .build();
    
    let readouts = [readout.voltage, readout.current, readout.power];
    for (i, value) in readouts.iter().enumerate() {
    
        let mut data = [Rgb565::BLACK; 108 * 32];
        let mut fbuf = FrameBuf::new(&mut data, 108, 32);

        // Text::with_text_style(
        //     format_f32::<8>(*value, 3).as_str(),
        //     Point::new(-2, -1),
        //     style.clone(),
        //     align,
        // )
        // .draw(&mut fbuf)
        // .map_err(|_| ())?;

        font.render_aligned(
        format_f32::<8>(*value, 3).as_str(),
            Point::new(108 + 1, -1),
        VerticalPosition::Top,
        HorizontalAlignment::Right,
        FontColor::Transparent(Rgb565::CSS_WHITE),
        &mut fbuf,
        )
        .map_err(|_| ())?;

        let top_left = Point::new(122 - 108, 30 + 62 * i as i32);
        let area = Rectangle::new(top_left, fbuf.size());

        target.fill_contiguous(&area, data).map_err(|_| ())?;
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
