use crate::scpi::{RegisterChannel, ScpiChannel, colors::Rgb};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScpiCommand {
    IdnQuery,
    Rst,
    Sav {
        slot: u8,
    },
    Rcl {
        slot: u8,
    },
    Del {
        slot: u8,
    },
    MeasQuery {
        kind: MeasKind,
        channel: ScpiChannel,
    },
    ChannelQuery {
        channel: ScpiChannel,
        param: ChannelParam,
    },
    ChannelSet {
        channel: ScpiChannel,
        param: ChannelParam,
        value: f32,
    },
    ColorSet {
        channel: ScpiChannel,
        rgb: Rgb,
    },
    LcdBrightnessSet {
        value: u8,
    },
    LcdBrightnessQuery,
    LedBrightnessSet {
        value: u8,
    },
    LedBrightnessQuery,
    OutputSet {
        channel: ScpiChannel,
        on: bool,
    },
    OutputQuery {
        channel: ScpiChannel,
    },
    ResetProt {
        channel: Option<ScpiChannel>,
    },
    SystErrQuery,
    SystVersQuery,
    SystLoc,
    SystRem,
    TelemQuery,
    TempQuery {
        slot: TempSlot,
    },
    InpQuery,
    DiagQuery,
    Ina226RegQuery {
        channel: RegisterChannel,
    },
    Tps55289RegQuery {
        channel: RegisterChannel,
    },
    Unknown {
        command: [u8; 64],
        command_len: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeasKind {
    Volt,
    Curr,
    Pow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelParam {
    Volt,
    Curr,
    Ovp,
    Ocp,
    Colr,
    Mode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TempSlot {
    Cha,
    Chb,
    Mcu,
}

pub fn normalize_command(raw: &str) -> heapless::String<256> {
    let trimmed = raw.trim();
    let mut out = heapless::String::<256>::new();
    let mut prev_space = true;
    for b in trimmed.bytes() {
        if b == b' ' || b == b'\t' {
            if !prev_space && !out.is_empty() {
                let _ = out.push(' ');
                prev_space = true;
            }
        } else {
            let c = if b.is_ascii_lowercase() { b - 32 } else { b };
            let _ = out.push(c as char);
            prev_space = false;
        }
    }
    while out.ends_with(' ') {
        out.pop();
    }
    out
}

pub fn parse_command(raw: &str) -> Option<ScpiCommand> {
    let cmd = normalize_command(raw);
    if cmd.is_empty() {
        return None;
    }

    let s = cmd.as_str();

    if s == "*IDN?" {
        return Some(ScpiCommand::IdnQuery);
    }
    if s == "*RST" {
        return Some(ScpiCommand::Rst);
    }
    if let Some(slot) = parse_star_slot(s, "*SAV ") {
        return Some(ScpiCommand::Sav { slot });
    }
    if let Some(slot) = parse_star_slot(s, "*RCL ") {
        return Some(ScpiCommand::Rcl { slot });
    }
    if let Some(slot) = parse_star_slot(s, "*DEL ") {
        return Some(ScpiCommand::Del { slot });
    }
    if s == "SYST:ERR?" {
        return Some(ScpiCommand::SystErrQuery);
    }
    if s == "SYST:VERS?" {
        return Some(ScpiCommand::SystVersQuery);
    }
    if s == "SYST:LOC" {
        return Some(ScpiCommand::SystLoc);
    }
    if s == "SYST:REM" {
        return Some(ScpiCommand::SystRem);
    }
    if s == "TELEM?" {
        return Some(ScpiCommand::TelemQuery);
    }
    if s == "INP?" {
        return Some(ScpiCommand::InpQuery);
    }
    if s == "DIAG?" {
        return Some(ScpiCommand::DiagQuery);
    }
    if s == "LCD:BRIG?" {
        return Some(ScpiCommand::LcdBrightnessQuery);
    }
    if s == "LED:BRIG?" {
        return Some(ScpiCommand::LedBrightnessQuery);
    }
    if let Some(value) = parse_brightness_set(s, "LCD:BRIG ") {
        return Some(ScpiCommand::LcdBrightnessSet { value });
    }
    if let Some(value) = parse_brightness_set(s, "LED:BRIG ") {
        return Some(ScpiCommand::LedBrightnessSet { value });
    }
    if s == "OUTP:RESET:PROT" {
        return Some(ScpiCommand::ResetProt { channel: None });
    }

    if let Some(rest) = s.strip_prefix("MEAS:")
        && let Some((kind, ch)) = parse_meas(rest)
    {
        return Some(ScpiCommand::MeasQuery { kind, channel: ch });
    }

    if let Some((ch, param)) = parse_channel_query(s) {
        return Some(ScpiCommand::ChannelQuery { channel: ch, param });
    }

    if let Some((ch, on)) = parse_output_set(s) {
        return Some(ScpiCommand::OutputSet { channel: ch, on });
    }
    if let Some(ch) = parse_output_query(s) {
        return Some(ScpiCommand::OutputQuery { channel: ch });
    }
    if let Some(ch) = parse_reset_prot(s) {
        return Some(ScpiCommand::ResetProt { channel: Some(ch) });
    }
    if let Some((slot, _)) = parse_temp_query(s) {
        return Some(ScpiCommand::TempQuery { slot });
    }
    if let Some(ch) = parse_ina226_reg(s) {
        return Some(ScpiCommand::Ina226RegQuery { channel: ch });
    }
    if let Some(ch) = parse_tps55289_reg(s) {
        return Some(ScpiCommand::Tps55289RegQuery { channel: ch });
    }
    if let Some((ch, rgb)) = parse_color_set(s) {
        return Some(ScpiCommand::ColorSet { channel: ch, rgb });
    }
    if let Some((ch, param, value)) = parse_channel_set(s) {
        return Some(ScpiCommand::ChannelSet {
            channel: ch,
            param,
            value,
        });
    }
    if let Some(ch) = parse_informal_dump(s, "INA226 DUMP CH") {
        return Some(ScpiCommand::Ina226RegQuery { channel: ch });
    }
    if let Some(ch) = parse_informal_dump(s, "TPS55289 DUMP CH") {
        return Some(ScpiCommand::Tps55289RegQuery { channel: ch });
    }

    let mut command = [0u8; 64];
    let bytes = s.as_bytes();
    let len = bytes.len().min(64);
    command[..len].copy_from_slice(&bytes[..len]);
    Some(ScpiCommand::Unknown {
        command,
        command_len: len as u8,
    })
}

fn parse_star_slot(s: &str, prefix: &str) -> Option<u8> {
    if !s.starts_with(prefix) {
        return None;
    }
    let digit = s.as_bytes().get(prefix.len())?;
    if *digit < b'1' || *digit > b'9' {
        return None;
    }
    if s.len() != prefix.len() + 1 {
        return None;
    }
    Some(*digit - b'0')
}

fn parse_scpi_channel(token: &str) -> Option<ScpiChannel> {
    match token {
        "CH1" => Some(ScpiChannel::Ch1),
        "CH2" => Some(ScpiChannel::Ch2),
        _ => None,
    }
}

fn parse_reg_channel(token: &str) -> Option<RegisterChannel> {
    match token {
        "CHA" => Some(RegisterChannel::Cha),
        "CHB" => Some(RegisterChannel::Chb),
        _ => None,
    }
}

fn parse_meas(rest: &str) -> Option<(MeasKind, ScpiChannel)> {
    let (kind_str, ch_str) = rest.split_once('?')?;
    let ch = parse_scpi_channel(ch_str.trim_start())?;
    let kind = match kind_str.trim_end_matches('?') {
        "VOLT" => MeasKind::Volt,
        "CURR" => MeasKind::Curr,
        "POW" => MeasKind::Pow,
        _ => return None,
    };
    Some((kind, ch))
}

fn ch_prefix_matches(s: &str, ch: &str, suffix: &str) -> bool {
    let mut expected = heapless::String::<16>::new();
    let _ = expected.push_str(ch);
    let _ = expected.push_str(suffix);
    s == expected.as_str()
}

fn parse_channel_query(s: &str) -> Option<(ScpiChannel, ChannelParam)> {
    for (ch_str, ch) in [("CH1", ScpiChannel::Ch1), ("CH2", ScpiChannel::Ch2)] {
        for (suffix, param) in [
            (":VOLT?", ChannelParam::Volt),
            (":CURR?", ChannelParam::Curr),
            (":OVP?", ChannelParam::Ovp),
            (":OCP?", ChannelParam::Ocp),
            (":COLR?", ChannelParam::Colr),
            (":MODE?", ChannelParam::Mode),
        ] {
            if ch_prefix_matches(s, ch_str, suffix) {
                return Some((ch, param));
            }
        }
    }
    None
}

fn parse_channel_set(s: &str) -> Option<(ScpiChannel, ChannelParam, f32)> {
    let parts: heapless::Vec<&str, 4> = s.split(' ').collect();
    if parts.len() != 2 {
        return None;
    }
    let (ch, param) = parse_channel_prefix(parts[0])?;
    let value = parse_float(parts[1])?;
    Some((ch, param, value))
}

fn parse_channel_prefix(s: &str) -> Option<(ScpiChannel, ChannelParam)> {
    for (ch_str, ch) in [("CH1", ScpiChannel::Ch1), ("CH2", ScpiChannel::Ch2)] {
        for (suffix, param) in [
            (":VOLT", ChannelParam::Volt),
            (":CURR", ChannelParam::Curr),
            (":OVP", ChannelParam::Ovp),
            (":OCP", ChannelParam::Ocp),
        ] {
            if ch_prefix_matches(s, ch_str, suffix) {
                return Some((ch, param));
            }
        }
    }
    None
}

fn parse_color_set(s: &str) -> Option<(ScpiChannel, Rgb)> {
    let parts: heapless::Vec<&str, 3> = s.split(' ').collect();
    if parts.len() != 2 {
        return None;
    }
    let ch = match parts[0] {
        "CH1:COLR" => ScpiChannel::Ch1,
        "CH2:COLR" => ScpiChannel::Ch2,
        _ => return None,
    };
    crate::scpi::colors::parse_rgb_triplet(parts[1])
        .ok()
        .map(|rgb| (ch, rgb))
}

fn parse_brightness_set(s: &str, prefix: &str) -> Option<u8> {
    if !s.starts_with(prefix) {
        return None;
    }
    let rest = &s[prefix.len()..];
    if rest.contains(' ') {
        return None;
    }
    crate::scpi::colors::parse_brightness(rest).ok()
}

fn parse_output_set(s: &str) -> Option<(ScpiChannel, bool)> {
    if !s.starts_with("OUTP ") {
        return None;
    }
    let rest = &s[5..];
    let (ch_str, mode) = rest.split_once(',')?;
    let ch = parse_scpi_channel(ch_str)?;
    let on = match mode {
        "ON" => true,
        "OFF" => false,
        _ => return None,
    };
    Some((ch, on))
}

fn parse_output_query(s: &str) -> Option<ScpiChannel> {
    if !s.starts_with("OUTP? ") {
        return None;
    }
    parse_scpi_channel(s.strip_prefix("OUTP? ")?)
}

fn parse_reset_prot(s: &str) -> Option<ScpiChannel> {
    if !s.starts_with("OUTP:RESET:PROT ") {
        return None;
    }
    parse_scpi_channel(s.strip_prefix("OUTP:RESET:PROT ")?)
}

fn parse_temp_query(s: &str) -> Option<(TempSlot, ())> {
    if !s.starts_with("TEMP? ") {
        return None;
    }
    let slot = match s.strip_prefix("TEMP? ")? {
        "CHA" => TempSlot::Cha,
        "CHB" => TempSlot::Chb,
        "MCU" => TempSlot::Mcu,
        _ => return None,
    };
    Some((slot, ()))
}

fn parse_ina226_reg(s: &str) -> Option<RegisterChannel> {
    if !s.starts_with("INA226:REG? ") {
        return None;
    }
    parse_reg_channel(s.strip_prefix("INA226:REG? ")?)
}

fn parse_tps55289_reg(s: &str) -> Option<RegisterChannel> {
    if !s.starts_with("TPS55289:REG? ") {
        return None;
    }
    parse_reg_channel(s.strip_prefix("TPS55289:REG? ")?)
}

fn parse_informal_dump(s: &str, prefix: &str) -> Option<RegisterChannel> {
    if !s.starts_with(prefix) {
        return None;
    }
    match s.strip_prefix(prefix)? {
        "A" => Some(RegisterChannel::Cha),
        "B" => Some(RegisterChannel::Chb),
        _ => None,
    }
}

fn parse_float(s: &str) -> Option<f32> {
    let mut sign = 1.0f32;
    let mut i = 0;
    let bytes = s.as_bytes();
    if bytes.first() == Some(&b'+') {
        i = 1;
    } else if bytes.first() == Some(&b'-') {
        sign = -1.0;
        i = 1;
    }

    let mut int_part: f32 = 0.0;
    let mut frac_part: f32 = 0.0;
    let mut frac_div: f32 = 1.0;
    let mut seen_dot = false;

    while i < bytes.len() {
        let b = bytes[i];
        if b == b'.' {
            if seen_dot {
                return None;
            }
            seen_dot = true;
            i += 1;
            continue;
        }
        if !b.is_ascii_digit() {
            return None;
        }
        let digit = (b - b'0') as f32;
        if seen_dot {
            frac_div *= 10.0;
            frac_part += digit / frac_div;
        } else {
            int_part = int_part * 10.0 + digit;
        }
        i += 1;
    }

    Some(sign * (int_part + frac_part))
}

pub fn is_mutation(cmd: &ScpiCommand) -> bool {
    !matches!(
        cmd,
        ScpiCommand::IdnQuery
            | ScpiCommand::MeasQuery { .. }
            | ScpiCommand::ChannelQuery { .. }
            | ScpiCommand::OutputQuery { .. }
            | ScpiCommand::LcdBrightnessQuery
            | ScpiCommand::LedBrightnessQuery
            | ScpiCommand::SystErrQuery
            | ScpiCommand::SystVersQuery
            | ScpiCommand::TelemQuery
            | ScpiCommand::TempQuery { .. }
            | ScpiCommand::InpQuery
            | ScpiCommand::DiagQuery
            | ScpiCommand::Ina226RegQuery { .. }
            | ScpiCommand::Tps55289RegQuery { .. }
            | ScpiCommand::Unknown { .. }
    )
}

#[cfg(test)]
mod tests;
