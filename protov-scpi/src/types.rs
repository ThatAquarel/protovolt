use core::fmt::Write;

pub const RESPONSE_BUF: usize = 512;
pub const LINE_BUF: usize = 256;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScpiChannel {
    Ch1,
    Ch2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterChannel {
    Cha,
    Chb,
}

#[derive(Debug)]
pub struct ScpiResponse {
    pub text: Option<heapless::String<RESPONSE_BUF>>,
}

impl ScpiResponse {
    /// No bus line (mutations, *RST, etc.). Errors are reported via `SYST:ERR?`.
    pub fn none() -> Self {
        Self { text: None }
    }

    /// FWUP protocol acknowledgement (`SYST:FWUP:STAR`, `ABOR`, etc.).
    pub fn ok() -> Self {
        let mut text = heapless::String::<RESPONSE_BUF>::new();
        let _ = text.push_str("OK");
        Self { text: Some(text) }
    }

    pub fn with_text(text: heapless::String<RESPONSE_BUF>) -> Self {
        Self { text: Some(text) }
    }
}

pub fn format_rgb(rgb: Rgb) -> heapless::String<16> {
    let mut buf = heapless::String::new();
    let _ = write!(buf, "{},{},{}", rgb.r, rgb.g, rgb.b);
    buf
}

#[allow(clippy::result_unit_err)]
pub fn parse_rgb_triplet(raw: &str) -> Result<Rgb, ()> {
    let parts: heapless::Vec<&str, 3> = raw.split(',').collect();
    if parts.len() != 3 {
        return Err(());
    }
    let r = parts[0].trim().parse::<u16>().map_err(|_| ())?;
    let g = parts[1].trim().parse::<u16>().map_err(|_| ())?;
    let b = parts[2].trim().parse::<u16>().map_err(|_| ())?;
    if r > 255 || g > 255 || b > 255 {
        return Err(());
    }
    Ok(Rgb {
        r: r as u8,
        g: g as u8,
        b: b as u8,
    })
}

#[allow(clippy::result_unit_err)]
pub fn parse_brightness(raw: &str) -> Result<u8, ()> {
    let value = raw.trim().parse::<u16>().map_err(|_| ())?;
    if value > 255 {
        return Err(());
    }
    Ok(value as u8)
}
