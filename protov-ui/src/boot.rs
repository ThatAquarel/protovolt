use core::fmt::Write;

use embedded_graphics::{image::Image, pixelcolor::Rgb565, prelude::*, primitives::Rectangle};
use embedded_graphics_framebuf::FrameBuf;
use heapless::String;
use tinybmp::Bmp;
use u8g2_fonts::{
    FontRenderer,
    types::{FontColor, HorizontalAlignment, VerticalPosition},
};

use protov_core::model::DfuStatus;

use crate::{
    display::Display,
    fonts::Fonts,
    geometry::DisplayGeometry,
    labels,
    layout::Layout,
    theme::{self, BACKGROUND, FONT_MAIN, FONT_SMALL},
};

#[cfg(feature = "demo")]
use crate::labels::DEMO;

const DFU_STATUS_Y: i32 = 148;
const DFU_DETAIL_Y: i32 = 160;
const DFU_WARNING_Y: i32 = 184;

/// Pixel height of `u8g2_font_helvB08_tf` (see `ControlsScreen::HEADER_HEIGHT`).
const DFU_STRIPE_HEIGHT: usize = 8;
const DFU_STRIPE_WIDTH: usize = 320;
const DFU_STRIPE_FB_SIZE: usize = DFU_STRIPE_WIDTH * DFU_STRIPE_HEIGHT;

pub struct BootScreen<'b> {
    logo_bmp: Bmp<'b, Rgb565>,
}

fn draw_dfu_stripe<D>(
    target: &mut D,
    font: &FontRenderer,
    text: &str,
    y_center: i32,
    fg: Rgb565,
) -> Result<(), ()>
where
    D: Display,
{
    let mut fbuf_data = [BACKGROUND; DFU_STRIPE_FB_SIZE];
    let mut fbuf = FrameBuf::new(&mut fbuf_data, DFU_STRIPE_WIDTH, DFU_STRIPE_HEIGHT);

    font.render_aligned(
        text,
        Point::new(DFU_STRIPE_WIDTH as i32 / 2, -1),
        VerticalPosition::Top,
        HorizontalAlignment::Center,
        FontColor::Transparent(fg),
        &mut fbuf,
    )
    .map_err(|_| ())?;

    let top = y_center - DFU_STRIPE_HEIGHT as i32 / 2;
    let area = Rectangle::new(
        Point::new(0, top),
        Size::new(DFU_STRIPE_WIDTH as u32, DFU_STRIPE_HEIGHT as u32),
    );
    target.fill_contiguous(&area, fbuf_data).map_err(|_| ())
}

fn format_progress(received: u32, total: u32, buf: &mut String<16>) {
    buf.clear();
    let _ = write!(buf, "{received:>6}/{total:<6}");
}

impl<'b> BootScreen<'b> {
    pub fn new(_geometry: DisplayGeometry) -> Self {
        Self {
            logo_bmp: Bmp::from_slice(include_bytes!("../assets/protov_mini.bmp")).unwrap(),
        }
    }

    #[cfg(feature = "demo")]
    pub fn draw_demo_screen<D>(
        &mut self,
        target: &mut D,
        layout: &Layout,
        fonts: &Fonts,
    ) -> Result<(), ()>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let font = &fonts.info_small;
        let center = layout.center_x();
        let x_skew = 60;

        font.render_aligned(
            DEMO,
            Point::new(center - x_skew + 10, 100),
            VerticalPosition::Center,
            HorizontalAlignment::Left,
            FontColor::Transparent(FONT_SMALL),
            target,
        )
        .map_err(|_| ())?;

        Ok(())
    }

    pub fn draw_splash_screen<D>(&mut self, target: &mut D, layout: &Layout) -> Result<(), ()>
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

    pub fn draw_dfu_text<D>(
        &self,
        target: &mut D,
        fonts: &Fonts,
        status: DfuStatus,
    ) -> Result<(), ()>
    where
        D: Display,
    {
        let headline = match status {
            DfuStatus::Preparing { .. } => labels::DFU_PREPARING,
            DfuStatus::Receiving { .. } | DfuStatus::Ready { .. } => labels::DFU_TRANSFERRING,
            DfuStatus::Verified => labels::DFU_VERIFIED,
            DfuStatus::Flashing => labels::DFU_FLASHING,
            DfuStatus::Error => labels::DFU_FAILED,
            DfuStatus::Idle => return Ok(()),
        };

        let font = &fonts.info_small;
        draw_dfu_stripe(
            target,
            font,
            headline,
            DFU_STATUS_Y,
            FONT_MAIN,
        )?;

        let (received, total) = match status {
            DfuStatus::Preparing { total } => (0, total),
            DfuStatus::Receiving { received, total } => (received, total),
            DfuStatus::Ready { total } => (total, total),
            _ => (0, 0),
        };

        if matches!(
            status,
            DfuStatus::Preparing { .. } | DfuStatus::Receiving { .. } | DfuStatus::Ready { .. }
        ) {
            let mut progress_buf = String::<16>::new();
            format_progress(received, total, &mut progress_buf);
            draw_dfu_stripe(
                target,
                font,
                progress_buf.as_str(),
                DFU_DETAIL_Y,
                FONT_SMALL,
            )?;
        }

        draw_dfu_stripe(
            target,
            font,
            labels::DFU_DO_NOT_DISCONNECT,
            DFU_WARNING_Y,
            theme::WARNING,
        )
    }

    pub fn draw_splash_text<D>(
        &mut self,
        target: &mut D,
        layout: &Layout,
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

        let icon = if valid {
            theme::icons_2x::CHECKMARK
        } else {
            theme::icons_2x::CROSS
        };
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
