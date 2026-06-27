pub mod colors;
pub mod parser;
pub mod state;
pub mod telemetry;

pub use parser::ScpiCommand;

pub const RESPONSE_BUF: usize = 512;
pub const LINE_BUF: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScpiChannel {
    Ch1,
    Ch2,
}

impl ScpiChannel {
    pub fn to_hal(self) -> crate::model::Channel {
        match self {
            ScpiChannel::Ch1 => crate::model::Channel::A,
            ScpiChannel::Ch2 => crate::model::Channel::B,
        }
    }

    pub fn from_hal(ch: crate::model::Channel) -> Self {
        match ch {
            crate::model::Channel::A => ScpiChannel::Ch1,
            crate::model::Channel::B => ScpiChannel::Ch2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterChannel {
    Cha,
    Chb,
}

impl RegisterChannel {
    pub fn to_hal(self) -> crate::model::Channel {
        match self {
            RegisterChannel::Cha => crate::model::Channel::A,
            RegisterChannel::Chb => crate::model::Channel::B,
        }
    }
}

#[derive(Debug)]
pub struct ScpiResponse {
    pub text: Option<heapless::String<RESPONSE_BUF>>,
}

impl ScpiResponse {
    pub fn none() -> Self {
        Self { text: None }
    }

    pub fn with_text(text: heapless::String<RESPONSE_BUF>) -> Self {
        Self { text: Some(text) }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ScpiContext {
    pub temp_ch_a: f32,
    pub temp_ch_b: f32,
    pub temp_mcu: f32,
    pub input_type_pd: bool,
    pub input_voltage: f32,
    pub input_current: f32,
    pub sense_ok: bool,
    pub converter_ok: bool,
    pub prot_latched_a: bool,
    pub prot_latched_b: bool,
}

impl Default for ScpiContext {
    fn default() -> Self {
        use crate::config::{BOOT_TEMP_CH_A, BOOT_TEMP_CH_B, BOOT_TEMP_MCU};

        Self {
            temp_ch_a: BOOT_TEMP_CH_A,
            temp_ch_b: BOOT_TEMP_CH_B,
            temp_mcu: BOOT_TEMP_MCU,
            input_type_pd: true,
            input_voltage: 20.0,
            input_current: 0.35,
            sense_ok: true,
            converter_ok: true,
            prot_latched_a: false,
            prot_latched_b: false,
        }
    }
}

pub struct ScpiHandleResult {
    pub response: ScpiResponse,
    pub tasks: Option<crate::model::AppTask>,
}
