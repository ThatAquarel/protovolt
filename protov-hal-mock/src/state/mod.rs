mod schema;

pub use schema::{
    ChannelSnapshot, ControlRequest, ControlResponse, IdnSnapshot, MeasuredSnapshot, StateSnapshot,
};

use std::collections::HashMap;

use protov_core::config::{DEFAULT_CH1, DEFAULT_CH2, MANUFACTURER, PRODUCT_NAME};
use protov_core::model::{Channel, ChannelHardwareState, ConverterFlags, Readout};
use protov_core::scpi::ScpiChannel;
use protov_core::scpi::colors::Rgb;

use crate::device::MockDevice;
use crate::telemetry::refresh_context;

pub fn apply_snapshot(device: &mut MockDevice, snapshot: &StateSnapshot) {
    device.app.reset_to_factory(&mut device.scpi);

    if !snapshot.idn.serial.is_empty() {
        device.identity.serial = snapshot.idn.serial.clone();
    }
    if !snapshot.idn.fw_version.is_empty() {
        device.identity.fw_version = snapshot.idn.fw_version.clone();
    }
    if !snapshot.idn.hw_version.is_empty() {
        device.identity.hw_version = snapshot.idn.hw_version.clone();
    }

    device.scpi.remote = snapshot.remote;

    if let Some(value) = snapshot.lcd_brightness {
        device.scpi.lcd_brightness = value;
    }
    if let Some(value) = snapshot.led_brightness {
        device.scpi.led_brightness = value;
    }

    if let Some(ch) = snapshot.channel("CH1") {
        apply_channel(device, Channel::A, ScpiChannel::Ch1, ch, DEFAULT_CH1);
    }
    if let Some(ch) = snapshot.channel("CH2") {
        apply_channel(device, Channel::B, ScpiChannel::Ch2, ch, DEFAULT_CH2);
    }

    device.scpi.clear_error_queue();
    for entry in &snapshot.error_queue {
        if let (Some(code), Some(message)) = (entry[0].as_i64(), entry[1].as_str()) {
            device.scpi.seed_error(code as i32, message);
        }
    }

    refresh_context(
        &mut device.ctx,
        device.scpi.prot_latched(ScpiChannel::Ch1),
        device.scpi.prot_latched(ScpiChannel::Ch2),
    );
}

pub fn dump_snapshot(device: &MockDevice) -> StateSnapshot {
    let mut channels = HashMap::new();
    channels.insert(
        "CH1".to_owned(),
        dump_channel(device, Channel::A, ScpiChannel::Ch1),
    );
    channels.insert(
        "CH2".to_owned(),
        dump_channel(device, Channel::B, ScpiChannel::Ch2),
    );

    StateSnapshot {
        idn: IdnSnapshot {
            manufacturer: MANUFACTURER.to_owned(),
            model: PRODUCT_NAME.to_owned(),
            serial: device.identity.serial.clone(),
            fw_version: device.identity.fw_version.clone(),
            hw_version: device.identity.hw_version.clone(),
        },
        remote: device.scpi.remote,
        lcd_brightness: Some(device.scpi.lcd_brightness),
        led_brightness: Some(device.scpi.led_brightness),
        channels,
        error_queue: Vec::new(),
    }
}

fn apply_channel(
    device: &mut MockDevice,
    hal_ch: Channel,
    scpi_ch: ScpiChannel,
    ch: &ChannelSnapshot,
    default_color: protov_core::config::Rgb,
) {
    device
        .app
        .set_channel_setpoints(hal_ch, ch.voltage, ch.current, ch.ovp, ch.ocp);
    device.app.set_channel_enable(hal_ch, ch.output);

    let rgb = channel_color(ch, default_color);
    device.scpi.set_color(
        scpi_ch,
        Rgb {
            r: rgb.r,
            g: rgb.g,
            b: rgb.b,
        },
    );

    device.scpi.set_prot_latched(Some(scpi_ch), ch.prot_latched);

    let readout = measured_readout(ch);
    if let Some(readout) = readout {
        device.app.set_channel_readout(hal_ch, readout);
    }

    let hw_state = if ch.prot_latched {
        ch.latched_mode
            .as_deref()
            .and_then(parse_hw_mode)
            .unwrap_or(ChannelHardwareState::Off)
    } else if ch.output {
        derive_output_mode(hal_ch, ch, readout, device)
    } else {
        ChannelHardwareState::Off
    };
    device.app.set_channel_hw_state(hal_ch, hw_state);

    if ch.output {
        if let Some(readout) = readout {
            device.app.set_converter_flags(
                hal_ch,
                ConverterFlags {
                    enabled: true,
                    scp: false,
                    ovp: false,
                    ocp: ch.current > 0.0 && readout.current >= ch.current * 0.98,
                },
            );
        }
    } else {
        device.app.set_converter_flags(
            hal_ch,
            ConverterFlags {
                enabled: false,
                scp: false,
                ocp: false,
                ovp: false,
            },
        );
    }
}

fn dump_channel(device: &MockDevice, hal_ch: Channel, scpi_ch: ScpiChannel) -> ChannelSnapshot {
    let rgb = device.scpi.color(scpi_ch);
    let readout = device.app.channel_readout(hal_ch);
    ChannelSnapshot {
        voltage: device.app.target_voltage(hal_ch),
        current: device.app.target_current(hal_ch),
        ovp: device.app.limit_voltage(hal_ch),
        ocp: device.app.limit_current(hal_ch),
        output: device.app.channel_enable(hal_ch),
        prot_latched: device.scpi.prot_latched(scpi_ch),
        latched_mode: Some(device.app.hw_state(hal_ch).mode_str().to_owned()),
        color: Some([rgb.r, rgb.g, rgb.b]),
        color_r: None,
        color_g: None,
        color_b: None,
        measured: readout.map(|r| MeasuredSnapshot {
            voltage: r.voltage,
            current: r.current,
            power: r.power,
        }),
        load_ratio: None,
        voltage_droop: None,
    }
}

fn channel_color(
    ch: &ChannelSnapshot,
    default: protov_core::config::Rgb,
) -> protov_core::config::Rgb {
    if let Some([r, g, b]) = ch.color {
        return protov_core::config::Rgb { r, g, b };
    }
    protov_core::config::Rgb {
        r: ch.color_r.unwrap_or(default.r),
        g: ch.color_g.unwrap_or(default.g),
        b: ch.color_b.unwrap_or(default.b),
    }
}

fn measured_readout(ch: &ChannelSnapshot) -> Option<Readout> {
    let measured = ch.measured?;
    Some(Readout {
        voltage: measured.voltage,
        current: measured.current,
        power: measured.power,
    })
}

fn derive_output_mode(
    hal_ch: Channel,
    ch: &ChannelSnapshot,
    readout: Option<Readout>,
    device: &MockDevice,
) -> ChannelHardwareState {
    use protov_core::protection;

    let limits = protov_core::model::Limits {
        voltage: ch.ovp,
        current: ch.ocp,
    };
    let flags = if ch.output {
        readout.map(|r| ConverterFlags {
            enabled: true,
            scp: false,
            ovp: false,
            ocp: ch.current > 0.0 && r.current >= ch.current * 0.98,
        })
    } else {
        None
    };

    protection::derive_hw_state(
        hal_ch,
        ch.output,
        flags,
        readout,
        limits,
        &device.app.last_temp(),
        false,
        ChannelHardwareState::Off,
    )
}

pub fn parse_hw_mode(mode: &str) -> Option<ChannelHardwareState> {
    match mode.to_ascii_uppercase().as_str() {
        "OFF" => Some(ChannelHardwareState::Off),
        "CV" => Some(ChannelHardwareState::ConstantVoltage),
        "CC" => Some(ChannelHardwareState::ConstantCurrent),
        "SHORT" => Some(ChannelHardwareState::ShortCircuit),
        "TEMP" => Some(ChannelHardwareState::OverTemperature),
        "OCP" => Some(ChannelHardwareState::OverCurrent),
        "OVP" => Some(ChannelHardwareState::OverVoltage),
        _ => None,
    }
}

pub fn load_snapshot_json(text: &str) -> Result<StateSnapshot, serde_json::Error> {
    serde_json::from_str(text)
}
