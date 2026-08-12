//! Logical panel geometry (ST7789 @ 270° rotation: 320×240 landscape).

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisplayGeometry {
    pub width: u16,
    pub height: u16,
}

impl Default for DisplayGeometry {
    fn default() -> Self {
        Self {
            width: 320,
            height: 240,
        }
    }
}

impl DisplayGeometry {
    pub const fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }
}
