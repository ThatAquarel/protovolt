use embedded_graphics::{
    image::Image,
    pixelcolor::Rgb565,
    prelude::*
};
use tinybmp::Bmp;
use u8g2_fonts::{
    FontRenderer, fonts,
    types::{FontColor, HorizontalAlignment, VerticalPosition},
};

pub fn draw_splash_screen<D>(target: &mut D) -> Result<(), ()>
where
    D: DrawTarget<Color = Rgb565>,
{
    let bmp: Bmp<Rgb565> = Bmp::from_slice(include_bytes!("../assets/protovolt_mini.bmp")).unwrap();

    let left_padding = (320 - 220) / 2;
    let image: Image<'_, Bmp<'_, Rgb565>> = Image::new(&bmp, Point::new(left_padding, 32));

    image.draw(target).map_err(|_| ())?;

    Ok(())
}

pub fn draw_splash_text<D>(
    target: &mut D,
    pos: i32,
    title: &'static str,
    subtitle: &'static str,
    valid: bool,
) -> Result<(), ()>
where
    D: DrawTarget<Color = Rgb565>,
{
    let font = FontRenderer::new::<fonts::u8g2_font_helvB08_tf>();
    let y_skew = pos * 28;
    let x_skew = 60;

    font.render_aligned(
        title,
        Point::new(160 - x_skew, 0 + 100 + 48 + y_skew),
        VerticalPosition::Center,
        HorizontalAlignment::Left,
        FontColor::Transparent(Rgb565::CSS_WHITE),
        target,
    )
    .map_err(|_| ())?;

    font.render_aligned(
        subtitle,
        Point::new(160 - x_skew + 10, 12 + 100 + 48 + y_skew),
        VerticalPosition::Center,
        HorizontalAlignment::Left,
        FontColor::Transparent(Rgb565::CSS_DIM_GRAY),
        target,
    )
    .map_err(|_| ())?;

    let icons = FontRenderer::new::<fonts::u8g2_font_open_iconic_all_2x_t>();
    let icon = if valid {"\u{0073}"} else {"\u{011B}"};
    icons
        .render_aligned(
            icon,
            Point::new(160 + x_skew, 0 + 100 + 48 + y_skew),
            VerticalPosition::Center,
            HorizontalAlignment::Right,
            FontColor::Transparent(Rgb565::CSS_WHITE),
            target,
        )
        .map_err(|_| ())?;

    Ok(())
}
