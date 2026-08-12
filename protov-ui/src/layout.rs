use embedded_graphics::{
    draw_target::Translated,
    draw_target::DrawTargetExt,
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point},
};
use protov_core::model::Channel;

use crate::geometry::DisplayGeometry;

pub struct Layout {
    geometry: DisplayGeometry,
}

impl Layout {
    pub fn new(geometry: DisplayGeometry) -> Self {
        Self { geometry }
    }

    pub fn width(&self) -> u16 {
        self.geometry.width
    }

    pub fn center_x(&self) -> i32 {
        self.width() as i32 / 2
    }

    pub fn channel_section<'a, D>(
        &'a self,
        target: &'a mut D,
        channel: Channel,
    ) -> Translated<'a, D>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        match channel {
            Channel::A => self.ch_a_section(target),
            Channel::B => self.ch_b_section(target),
        }
    }

    pub fn settings_section<'a, D>(&'a self, target: &'a mut D) -> Translated<'a, D>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        self.ch_a_section(target)
    }

    fn ch_a_section<'a, D>(&'a self, target: &'a mut D) -> Translated<'a, D>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        target.translated(Point::new(0, 40))
    }

    fn ch_b_section<'a, D>(&'a self, target: &'a mut D) -> Translated<'a, D>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        target.translated(Point::new(163, 40))
    }
}
