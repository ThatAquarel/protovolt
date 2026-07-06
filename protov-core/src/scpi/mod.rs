pub mod colors;
pub mod parser;
pub mod registers;
pub mod state;
pub mod telemetry;

pub use protov_scpi::{
    LINE_BUF, RESPONSE_BUF, RegisterChannel, Rgb, ScpiChannel, ScpiCommand, ScpiResponse,
};

pub trait ScpiChannelExt {
    fn to_hal(self) -> crate::model::Channel;
    fn from_hal(ch: crate::model::Channel) -> Self;
}

impl ScpiChannelExt for ScpiChannel {
    fn to_hal(self) -> crate::model::Channel {
        match self {
            ScpiChannel::Ch1 => crate::model::Channel::A,
            ScpiChannel::Ch2 => crate::model::Channel::B,
        }
    }

    fn from_hal(ch: crate::model::Channel) -> Self {
        match ch {
            crate::model::Channel::A => ScpiChannel::Ch1,
            crate::model::Channel::B => ScpiChannel::Ch2,
        }
    }
}

pub trait RegisterChannelExt {
    fn to_hal(self) -> crate::model::Channel;
}

impl RegisterChannelExt for RegisterChannel {
    fn to_hal(self) -> crate::model::Channel {
        match self {
            RegisterChannel::Cha => crate::model::Channel::A,
            RegisterChannel::Chb => crate::model::Channel::B,
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{BOOT_TEMP_CH_A, BOOT_TEMP_CH_B, BOOT_TEMP_MCU};
    use crate::model::Channel;
    use crate::scpi::ScpiChannelExt;

    #[test]
    fn scpi_channel_hal_roundtrip() {
        assert_eq!(ScpiChannel::Ch1.to_hal(), Channel::A);
        assert_eq!(ScpiChannel::Ch2.to_hal(), Channel::B);
        assert_eq!(ScpiChannel::from_hal(Channel::A), ScpiChannel::Ch1);
        assert_eq!(ScpiChannel::from_hal(Channel::B), ScpiChannel::Ch2);
    }

    #[test]
    fn register_channel_maps_to_hal() {
        use crate::scpi::RegisterChannelExt;
        assert_eq!(RegisterChannel::Cha.to_hal(), Channel::A);
        assert_eq!(RegisterChannel::Chb.to_hal(), Channel::B);
    }

    #[test]
    fn scpi_response_constructors() {
        assert!(ScpiResponse::none().text.is_none());
        assert_eq!(ScpiResponse::ok().text.unwrap().as_str(), "OK");
        let mut text = heapless::String::<RESPONSE_BUF>::new();
        let _ = text.push_str("5.000");
        assert_eq!(
            ScpiResponse::with_text(text).text.unwrap().as_str(),
            "5.000"
        );
    }

    #[test]
    fn scpi_context_default_boot_temps() {
        let ctx = ScpiContext::default();
        assert_eq!(ctx.temp_ch_a, BOOT_TEMP_CH_A);
        assert_eq!(ctx.temp_ch_b, BOOT_TEMP_CH_B);
        assert_eq!(ctx.temp_mcu, BOOT_TEMP_MCU);
        assert!(ctx.input_type_pd);
        assert!(ctx.sense_ok);
        assert!(ctx.converter_ok);
        assert!(!ctx.prot_latched_a);
        assert!(!ctx.prot_latched_b);
    }
}
