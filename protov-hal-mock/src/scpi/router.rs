//! HAL-style SCPI command routing and response assembly.

use protov_core::config::format_idn_parts;
use protov_core::scpi::RESPONSE_BUF;
use protov_core::scpi::registers::{format_ina226_response, format_tps55289_response};
use protov_scpi::ScpiCommand;

use crate::device::MockDevice;

use super::tasks::run_tasks;

pub fn dispatch_command(device: &mut MockDevice, cmd: ScpiCommand) -> Option<String> {
    let (mut response, tasks) = match cmd {
        ScpiCommand::IdnQuery => {
            let mut buf = heapless::String::<RESPONSE_BUF>::new();
            format_idn_parts(
                &device.identity.serial,
                &device.identity.fw_version,
                &device.identity.hw_version,
                &mut buf,
            );
            return Some(buf.to_string());
        }
        ScpiCommand::Ina226RegQuery { channel } => {
            return format_ina226_response(channel).map(|s| s.to_string());
        }
        ScpiCommand::Tps55289RegQuery { channel } => {
            return format_tps55289_response(channel).map(|s| s.to_string());
        }
        ScpiCommand::FwupData => {
            let len = device.firmware.payload_len() as u32;
            let result = device
                .app
                .handle_fwup_data(len, &mut device.scpi);
            (result.response, result.tasks)
        }
        ScpiCommand::FwupAbor => {
            let result = device.app.handle_scpi(cmd, &mut device.scpi, &device.ctx);
            device.firmware.abort();
            (result.response, result.tasks)
        }
        _ => {
            let result = device.app.handle_scpi(cmd, &mut device.scpi, &device.ctx);
            (result.response, result.tasks)
        }
    };

    if let Some(tasks) = tasks {
        response = run_tasks(
            &mut device.app,
            &mut device.scpi,
            &device.ctx,
            &mut device.firmware,
            tasks,
            response,
        );
    }

    if matches!(cmd, ScpiCommand::FwupData) {
        device.firmware.clear_payload();
    }

    response.text.map(|s| s.to_string())
}

pub const POOL_FULL_RESPONSE: &str = r#"-221,"All four mock device slots are in use""#;
