pub mod parser;
pub mod state;
pub mod telemetry;
pub mod usb;

pub use usb::USB_ENUM_GRACE_MS;

use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;

use parser::ScpiCommand;

pub const RESPONSE_BUF: usize = 512;
pub const LINE_BUF: usize = 256;

pub static SCPI_CMD: Channel<ThreadModeRawMutex, ScpiCommand, 4> = Channel::new();
pub static SCPI_RESP: Channel<ThreadModeRawMutex, ScpiResponse, 4> = Channel::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScpiChannel {
    Ch1,
    Ch2,
}

impl ScpiChannel {
    pub fn to_hal(self) -> crate::hal::event::Channel {
        match self {
            ScpiChannel::Ch1 => crate::hal::event::Channel::A,
            ScpiChannel::Ch2 => crate::hal::event::Channel::B,
        }
    }

    pub fn from_hal(ch: crate::hal::event::Channel) -> Self {
        match ch {
            crate::hal::event::Channel::A => ScpiChannel::Ch1,
            crate::hal::event::Channel::B => ScpiChannel::Ch2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterChannel {
    Cha,
    Chb,
}

impl RegisterChannel {
    pub fn to_hal(self) -> crate::hal::event::Channel {
        match self {
            RegisterChannel::Cha => crate::hal::event::Channel::A,
            RegisterChannel::Chb => crate::hal::event::Channel::B,
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
        Self {
            temp_ch_a: 28.0,
            temp_ch_b: 27.5,
            temp_mcu: 34.0,
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
    pub tasks: Option<crate::hal::event::AppTask>,
}
