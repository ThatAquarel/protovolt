use embedded_graphics::{image::Image, pixelcolor::Rgb565, prelude::*};
use tinybmp::Bmp;
use u8g2_fonts::{
    FontRenderer, fonts,
    types::{FontColor, HorizontalAlignment, VerticalPosition},
};

use crate::ui::{color_scheme::{FONT_MAIN, FONT_SMALL}, icons_2x, Fonts, Layout};

pub struct BootScreen<'b> {
    logo_bmp: Bmp<'b, Rgb565>,
}

impl<'b> BootScreen<'b> {
    pub fn new() -> Self {
        Self {
            logo_bmp: Bmp::from_slice(include_bytes!("../assets/protovolt_mini.bmp")).unwrap(),
        }
    }

    pub fn draw_splash_screen<D>(&mut self, target: &mut D, layout: &mut Layout) -> Result<(), ()>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let image_width = self.logo_bmp.bounding_box().size.width;
        let left_padding = (layout.width() as i32 - image_width as i32) / 2;

        let image: Image<'_, Bmp<'_, Rgb565>> =
            Image::new(&self.logo_bmp, Point::new(left_padding, 32));

        image.draw(target).map_err(|_| ())?;

        Ok(())
    }

    pub fn draw_splash_text<D>(
        &mut self,
        target: &mut D,
        layout: &mut Layout,
        fonts: &Fonts,
        pos: u8,
        title: &'static str,
        subtitle: &'static str,
        valid: bool,
    ) -> Result<(), ()>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let font = &fonts.info_small;
        let icons = &fonts.icons_2x;

        let y_skew = 28 * pos as i32;
        let x_skew = 60;

        let center = layout.center_x();

        font.render_aligned(
            title,
            Point::new(center - x_skew, 0 + 100 + 48 + y_skew),
            VerticalPosition::Center,
            HorizontalAlignment::Left,
            FontColor::Transparent(FONT_MAIN),
            target,
        )
        .map_err(|_| ())?;

        font.render_aligned(
            subtitle,
            Point::new(center - x_skew + 10, 12 + 100 + 48 + y_skew),
            VerticalPosition::Center,
            HorizontalAlignment::Left,
            FontColor::Transparent(FONT_SMALL),
            target,
        )
        .map_err(|_| ())?;

        let icon = if valid { icons_2x::CHECKMARK } else { icons_2x::CROSS };
        icons
            .render_aligned(
                icon,
                Point::new(center + x_skew, 0 + 100 + 48 + y_skew),
                VerticalPosition::Center,
                HorizontalAlignment::Right,
                FontColor::Transparent(FONT_MAIN),
                target,
            )
            .map_err(|_| ())?;

        Ok(())
    }
}
