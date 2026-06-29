use protov_core::config::format_idn_parts;
use protov_core::scpi::RESPONSE_BUF;
use protov_core::scpi::parser::ScpiCommand;
use protov_core::scpi::parser::parse_command;
use protov_core::scpi::registers::{format_ina226_response, format_tps55289_response};

use crate::device::MockDevice;

pub fn handle_command(device: &mut MockDevice, raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let cmd = parse_command(trimmed)?;

    match cmd {
        ScpiCommand::IdnQuery => {
            let mut buf = heapless::String::<RESPONSE_BUF>::new();
            format_idn_parts(
                &device.identity.serial,
                &device.identity.fw_version,
                &device.identity.hw_version,
                &mut buf,
            );
            Some(buf.to_string())
        }
        ScpiCommand::Ina226RegQuery { channel } => {
            format_ina226_response(channel).map(|s| s.to_string())
        }
        ScpiCommand::Tps55289RegQuery { channel } => {
            format_tps55289_response(channel).map(|s| s.to_string())
        }
        _ => {
            let result = device.app.handle_scpi(cmd, &mut device.scpi, &device.ctx);
            result.response.text.map(|s| s.to_string())
        }
    }
}

pub const POOL_FULL_RESPONSE: &str = r#"-221,"All four mock device slots are in use""#;
