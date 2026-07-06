use crate::command::ScpiCommand;

#[cfg(not(feature = "std"))]
pub fn encode_command_line(_cmd: &ScpiCommand) -> heapless::String<128> {
    heapless::String::new()
}

#[cfg(feature = "std")]
mod labels {
    use crate::command::{ChannelParam, TempSlot};
    use crate::types::{RegisterChannel, ScpiChannel};

    pub fn channel_label(ch: ScpiChannel) -> &'static str {
        match ch {
            ScpiChannel::Ch1 => "CH1",
            ScpiChannel::Ch2 => "CH2",
        }
    }

    pub fn channel_num(ch: ScpiChannel) -> u8 {
        match ch {
            ScpiChannel::Ch1 => 1,
            ScpiChannel::Ch2 => 2,
        }
    }

    pub fn channel_prefix(ch: ScpiChannel) -> &'static str {
        channel_label(ch)
    }

    pub fn param_label(p: ChannelParam) -> &'static str {
        match p {
            ChannelParam::Volt => "VOLT",
            ChannelParam::Curr => "CURR",
            ChannelParam::Ovp => "OVP",
            ChannelParam::Ocp => "OCP",
            ChannelParam::Colr => "COLR",
            ChannelParam::Mode => "MODE",
        }
    }

    pub fn temp_slot_label(s: TempSlot) -> &'static str {
        match s {
            TempSlot::Cha => "CHA",
            TempSlot::Chb => "CHB",
            TempSlot::Mcu => "MCU",
        }
    }

    pub fn reg_channel_label(ch: RegisterChannel) -> &'static str {
        match ch {
            RegisterChannel::Cha => "CHA",
            RegisterChannel::Chb => "CHB",
        }
    }
}

#[cfg(feature = "std")]
pub fn encode_command_line(cmd: &ScpiCommand) -> alloc::string::String {
    use core::fmt::Write;
    use labels::*;
    match cmd {
        ScpiCommand::IdnQuery => "*IDN?\n".into(),
        ScpiCommand::Rst => "*RST\n".into(),
        ScpiCommand::Sav { slot } => format!("*SAV {slot}\n"),
        ScpiCommand::Rcl { slot } => format!("*RCL {slot}\n"),
        ScpiCommand::Del { slot } => format!("*DEL {slot}\n"),
        ScpiCommand::MeasQuery { kind, channel } => {
            let k = match kind {
                crate::command::MeasKind::Volt => "VOLT",
                crate::command::MeasKind::Curr => "CURR",
                crate::command::MeasKind::Pow => "POW",
            };
            format!("MEAS:{k}? {}\n", channel_label(*channel))
        }
        ScpiCommand::ChannelQuery { channel, param } => {
            format!(
                "{}:{}? {}\n",
                channel_prefix(*channel),
                param_label(*param),
                channel_label(*channel)
            )
        }
        ScpiCommand::ChannelSet {
            channel,
            param,
            value,
        } => {
            format!(
                "{}:{} {} {:.6}\n",
                channel_prefix(*channel),
                param_label(*param),
                channel_label(*channel),
                value
            )
        }
        ScpiCommand::ColorSet { channel, rgb } => {
            format!(
                "CH{}:COLR {} {},{},{}\n",
                channel_num(*channel),
                channel_label(*channel),
                rgb.r,
                rgb.g,
                rgb.b
            )
        }
        ScpiCommand::LcdBrightnessSet { value } => format!("SYST:BRIG:LCD {value}\n"),
        ScpiCommand::LcdBrightnessQuery => "SYST:BRIG:LCD?\n".into(),
        ScpiCommand::LedBrightnessSet { value } => format!("SYST:BRIG:LED {value}\n"),
        ScpiCommand::LedBrightnessQuery => "SYST:BRIG:LED?\n".into(),
        ScpiCommand::OutputSet { channel, on } => {
            format!(
                "OUTP {} {}\n",
                channel_label(*channel),
                if *on { "ON" } else { "OFF" }
            )
        }
        ScpiCommand::OutputQuery { channel } => {
            format!("OUTP? {}\n", channel_label(*channel))
        }
        ScpiCommand::ResetProt { channel } => match channel {
            Some(ch) => format!("OUTP:RESET:PROT {}\n", channel_label(*ch)),
            None => "OUTP:RESET:PROT\n".into(),
        },
        ScpiCommand::SystErrQuery => "SYST:ERR?\n".into(),
        ScpiCommand::SystVersQuery => "SYST:VERS?\n".into(),
        ScpiCommand::SystLoc => "SYST:LOC\n".into(),
        ScpiCommand::SystRem => "SYST:REM\n".into(),
        ScpiCommand::TelemQuery => "SYST:TELEM?\n".into(),
        ScpiCommand::TempQuery { slot } => {
            format!("SYST:TEMP? {}\n", temp_slot_label(*slot))
        }
        ScpiCommand::InpQuery => "INP?\n".into(),
        ScpiCommand::DiagQuery => "DIAG?\n".into(),
        ScpiCommand::Ina226RegQuery { channel } => {
            format!("DIAG:INA226? {}\n", reg_channel_label(*channel))
        }
        ScpiCommand::Tps55289RegQuery { channel } => {
            format!("DIAG:TPS55289? {}\n", reg_channel_label(*channel))
        }
        ScpiCommand::FwupStatQuery => "SYST:FWUP:STAT?\n".into(),
        ScpiCommand::FwupStar { size } => format!("SYST:FWUP:STAR {size}\n"),
        ScpiCommand::FwupData => "SYST:FWUP:DATA\n".into(),
        ScpiCommand::FwupAppl { signature } => {
            crate::block::encode_fwup_appl_line(signature)
                .trim_end()
                .to_string()
                + "\n"
        }
        ScpiCommand::FwupAbor => "SYST:FWUP:ABOR\n".into(),
        ScpiCommand::Unknown {
            command,
            command_len,
        } => {
            let len = *command_len as usize;
            let mut s = alloc::string::String::new();
            let _ = s.write_str(core::str::from_utf8(&command[..len]).unwrap_or(""));
            s.push('\n');
            s
        }
    }
}
