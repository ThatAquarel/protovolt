mod schema;

pub use schema::{
    ChannelSnapshot, ControlRequest, ControlResponse, IdnSnapshot, InterfaceSnapshot,
    MeasuredSnapshot, PowerSnapshot, StateSnapshot, UiMode,
};

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use protov_core::app::ArrowsFunction;
use protov_core::config::{
    CH1_FACTORY, CH2_FACTORY, DEFAULT_CH1, DEFAULT_CH2, FACTORY, MANUFACTURER, PRODUCT_NAME,
};
use protov_core::model::{
    Channel, ChannelHardwareState, ConverterFlags, FunctionButton, Limits, PowerType, Readout,
    SetSelect, SetState,
};
use protov_core::scpi::ScpiChannel;
use protov_core::scpi::colors::Rgb;

use crate::device::MockDevice;
use crate::telemetry::refresh_context;

pub fn apply_snapshot(device: &mut MockDevice, snapshot: &StateSnapshot) {
    device.app.reset_to_factory(&mut device.scpi);
    device.app.force_standby();

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

    apply_power(device, snapshot.power.as_ref());
    apply_interface(device, snapshot.interface.as_ref());

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

fn apply_power(device: &mut MockDevice, power: Option<&PowerSnapshot>) {
    let Some(power) = power else {
        return;
    };

    let limits = Limits {
        voltage: power.voltage.unwrap_or(5.0),
        current: power.current.unwrap_or(0.5),
    };
    device.app.set_power_type(match power.kind.as_deref() {
        Some("power_delivery") | Some("PowerDelivery") | Some("PD") => {
            PowerType::PowerDelivery(limits)
        }
        _ => PowerType::Standard(limits),
    });
    device.ctx.input_type_pd = matches!(device.app.power_type(), PowerType::PowerDelivery(_));
    device.ctx.input_voltage = limits.voltage;
    device.ctx.input_current = limits.current;
}

fn apply_interface(device: &mut MockDevice, interface: Option<&InterfaceSnapshot>) {
    let Some(interface) = interface else {
        return;
    };

    if let Some(channel) = &interface.selected_channel {
        device.app.set_selected_channel(parse_channel(channel));
    }

    device.app.set_settings_open(interface.settings_open);

    if let Some(set_state) = &interface.set_state {
        device.app.set_set_state(parse_set_state(set_state));
    }

    if let Some(arrows) = &interface.arrows_function {
        device
            .app
            .set_arrows_function(parse_arrows_function(arrows));
    }

    if let (Some(channel), Some(set_select)) = (&interface.selected_channel, &interface.set_select)
    {
        if let Some(ch) = parse_channel(channel) {
            device
                .app
                .set_channel_set_select(ch, parse_set_select(set_select));
        }
    }

    if let Some(exp) = interface.edit_precision {
        device.app.set_edit_precision_exponent(exp);
    }
}

pub fn nav_button_from_snapshot(interface: Option<&InterfaceSnapshot>) -> Option<FunctionButton> {
    let nav = interface.and_then(|i| i.nav_button.as_deref())?;
    match nav.to_ascii_lowercase().as_str() {
        "enter" => Some(FunctionButton::Enter),
        "switch" => Some(FunctionButton::Switch),
        "settings" => Some(FunctionButton::Settings),
        _ => None,
    }
}

pub fn serial_connected_from_snapshot(power: Option<&PowerSnapshot>) -> bool {
    power.is_some_and(|p| p.serial_connected)
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

    let (power_kind, power_voltage, power_current) = match device.app.power_type() {
        PowerType::PowerDelivery(limits) => (
            Some("power_delivery".to_owned()),
            limits.voltage,
            limits.current,
        ),
        PowerType::Standard(limits) => {
            (Some("standard".to_owned()), limits.voltage, limits.current)
        }
    };

    StateSnapshot {
        description: String::new(),
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
        hardware_state: Some("Standby".to_owned()),
        interface: Some(InterfaceSnapshot {
            selected_channel: device
                .app
                .selected_channel()
                .map(|ch| channel_name(ch).to_owned()),
            settings_open: device.app.settings_open(),
            set_state: Some(set_state_name(device.app.set_state()).to_owned()),
            arrows_function: Some(if device.app.is_setpoint_edit() {
                "setpoint_edit".to_owned()
            } else {
                "navigation".to_owned()
            }),
            set_select: device
                .app
                .selected_channel()
                .map(|ch| set_select_name(device.app.set_select(ch)).to_owned()),
            edit_precision: None,
            nav_button: None,
        }),
        power: Some(PowerSnapshot {
            kind: power_kind,
            voltage: Some(power_voltage),
            current: Some(power_current),
            serial_connected: false,
        }),
        ui: Some(UiMode::Main),
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
    let overlay = serde_json::from_str::<StateSnapshot>(text)?;
    Ok(merge_with_default(overlay))
}

pub fn load_state_file(path: &Path) -> Result<StateSnapshot, StateLoadError> {
    let value = load_state_value(path)?;
    serde_yaml::from_value(value).map_err(|source| StateLoadError::YamlParse { source })
}

fn load_state_value(path: &Path) -> Result<serde_yaml::Value, StateLoadError> {
    let content = std::fs::read_to_string(path).map_err(|source| StateLoadError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut value: serde_yaml::Value =
        serde_yaml::from_str(&content).map_err(|source| StateLoadError::YamlParse { source })?;

    if let Some(extends) = value
        .as_mapping()
        .and_then(|map| map.get(serde_yaml::Value::from("extends")))
        .and_then(|v| v.as_str())
    {
        let parent_path = path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(extends);
        let mut base = load_state_value(&parent_path)?;
        if let Some(map) = value.as_mapping_mut() {
            map.remove(serde_yaml::Value::from("extends"));
        }
        merge_yaml_values(&mut base, value);
        return Ok(base);
    }

    Ok(value)
}

pub fn merge_with_default(overlay: StateSnapshot) -> StateSnapshot {
    let default = default_snapshot();
    let mut base_value = serde_json::to_value(&default).expect("default snapshot serializes");
    let overlay_value = serde_json::to_value(&overlay).expect("overlay snapshot serializes");
    merge_json_values(&mut base_value, overlay_value);
    serde_json::from_value(base_value).expect("merged snapshot deserializes")
}

pub fn default_snapshot() -> StateSnapshot {
    StateSnapshot {
        description: "Post-boot standby main screen".to_owned(),
        idn: IdnSnapshot {
            serial: "550e8400".to_owned(),
            fw_version: "1.0.0".to_owned(),
            hw_version: "A.1".to_owned(),
            ..Default::default()
        },
        remote: true,
        lcd_brightness: Some(FACTORY.appearance.lcd_brightness),
        led_brightness: Some(FACTORY.appearance.led_brightness),
        hardware_state: Some("Standby".to_owned()),
        interface: Some(InterfaceSnapshot {
            selected_channel: Some("A".to_owned()),
            settings_open: false,
            set_state: Some("set".to_owned()),
            arrows_function: Some("navigation".to_owned()),
            set_select: Some("voltage".to_owned()),
            edit_precision: None,
            nav_button: None,
        }),
        power: Some(PowerSnapshot {
            kind: Some("standard".to_owned()),
            voltage: Some(5.0),
            current: Some(0.5),
            serial_connected: false,
        }),
        ui: Some(UiMode::Main),
        channels: HashMap::from([
            (
                "CH1".to_owned(),
                channel_from_factory(
                    CH1_FACTORY.voltage_set,
                    CH1_FACTORY.current_set,
                    CH1_FACTORY.ovp,
                    CH1_FACTORY.ocp,
                ),
            ),
            (
                "CH2".to_owned(),
                channel_from_factory(
                    CH2_FACTORY.voltage_set,
                    CH2_FACTORY.current_set,
                    CH2_FACTORY.ovp,
                    CH2_FACTORY.ocp,
                ),
            ),
        ]),
        error_queue: Vec::new(),
    }
}

fn channel_from_factory(voltage: f32, current: f32, ovp: f32, ocp: f32) -> ChannelSnapshot {
    ChannelSnapshot {
        voltage,
        current,
        ovp,
        ocp,
        output: false,
        prot_latched: false,
        latched_mode: None,
        color: None,
        color_r: None,
        color_g: None,
        color_b: None,
        measured: None,
        load_ratio: None,
        voltage_droop: None,
    }
}

fn merge_yaml_values(base: &mut serde_yaml::Value, overlay: serde_yaml::Value) {
    match (base, overlay) {
        (serde_yaml::Value::Mapping(base_map), serde_yaml::Value::Mapping(overlay_map)) => {
            for (key, overlay_val) in overlay_map {
                if key.as_str() == Some("extends") {
                    continue;
                }
                match base_map.get_mut(&key) {
                    Some(base_val) if base_val.is_mapping() && overlay_val.is_mapping() => {
                        merge_yaml_values(base_val, overlay_val);
                    }
                    _ => {
                        base_map.insert(key, overlay_val);
                    }
                }
            }
        }
        (base_slot, overlay) => *base_slot = overlay,
    }
}

fn merge_json_values(base: &mut serde_json::Value, overlay: serde_json::Value) {
    match (base, overlay) {
        (serde_json::Value::Object(base_map), serde_json::Value::Object(overlay_map)) => {
            for (key, overlay_val) in overlay_map {
                if key == "extends" {
                    continue;
                }
                match base_map.get_mut(&key) {
                    Some(base_val) if base_val.is_object() && overlay_val.is_object() => {
                        merge_json_values(base_val, overlay_val);
                    }
                    _ => {
                        base_map.insert(key, overlay_val);
                    }
                }
            }
        }
        (base_slot, overlay) => *base_slot = overlay,
    }
}

fn parse_channel(name: &str) -> Option<Channel> {
    match name.to_ascii_uppercase().as_str() {
        "A" | "CH1" | "CH 1" | "1" => Some(Channel::A),
        "B" | "CH2" | "CH 2" | "2" => Some(Channel::B),
        _ => None,
    }
}

fn channel_name(channel: Channel) -> &'static str {
    match channel {
        Channel::A => "A",
        Channel::B => "B",
    }
}

fn parse_set_state(name: &str) -> SetState {
    match name.to_ascii_lowercase().as_str() {
        "limits" => SetState::Limits,
        _ => SetState::Set,
    }
}

fn set_state_name(state: SetState) -> &'static str {
    match state {
        SetState::Set => "set",
        SetState::Limits => "limits",
    }
}

fn parse_arrows_function(name: &str) -> ArrowsFunction {
    match name.to_ascii_lowercase().as_str() {
        "setpoint_edit" | "edit" => ArrowsFunction::SetpointEdit,
        _ => ArrowsFunction::Navigation,
    }
}

fn parse_set_select(name: &str) -> SetSelect {
    match name.to_ascii_lowercase().as_str() {
        "current" | "i" | "a" => SetSelect::Current,
        _ => SetSelect::Voltage,
    }
}

fn set_select_name(select: SetSelect) -> &'static str {
    match select {
        SetSelect::Voltage => "voltage",
        SetSelect::Current => "current",
    }
}

#[derive(Debug)]
pub enum StateLoadError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    YamlParse {
        source: serde_yaml::Error,
    },
}

impl std::fmt::Display for StateLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "failed to read {}: {source}", path.display()),
            Self::YamlParse { source } => write!(f, "failed to parse state yaml: {source}"),
        }
    }
}

impl std::error::Error for StateLoadError {}
