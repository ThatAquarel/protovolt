use core::fmt::Error;

use embedded_graphics::{pixelcolor::Rgb565, prelude::DrawTarget};

pub mod controls;
pub mod boot;

pub trait Display: DrawTarget<Color = Rgb565> {}
impl<T: DrawTarget<Color = Rgb565>> Display for T {}
