#![no_std]

pub mod appearance;
pub mod boot;
pub mod controls;
pub mod dispatch;
pub mod display;
pub mod fonts;
pub mod geometry;
pub mod labels;
pub mod layout;
pub mod navbar;
pub mod platform;
pub mod renderer;
pub mod settings;
pub mod theme;

pub use dispatch::dispatch_display_task;
pub use geometry::DisplayGeometry;
pub use platform::{NullPlatform, UiPlatform};
pub use renderer::UiRenderer;
