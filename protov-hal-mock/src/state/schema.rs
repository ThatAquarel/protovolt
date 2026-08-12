use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct StateSnapshot {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(default)]
    pub idn: IdnSnapshot,
    #[serde(default)]
    pub remote: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lcd_brightness: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub led_brightness: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hardware_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interface: Option<InterfaceSnapshot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub power: Option<PowerSnapshot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui: Option<UiMode>,
    #[serde(default)]
    pub channels: HashMap<String, ChannelSnapshot>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub error_queue: Vec<[serde_json::Value; 2]>,
}

impl StateSnapshot {
    pub fn channel(&self, name: &str) -> Option<&ChannelSnapshot> {
        self.channels
            .get(name)
            .or_else(|| self.channels.get(&name.to_ascii_uppercase()))
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct InterfaceSnapshot {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_channel: Option<String>,
    #[serde(default)]
    pub settings_open: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub set_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arrows_function: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub set_select: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edit_precision: Option<i8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nav_button: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PowerSnapshot {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voltage: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current: Option<f32>,
    #[serde(default)]
    pub serial_connected: bool,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UiMode {
    #[default]
    Main,
    BootSplash,
    BootInputPd,
    BootInputStd,
    BootSensePass,
    BootSenseFail,
    BootConverterPass,
    BootConverterFail,
    Settings,
    DfuPreparing,
    DfuTransferring,
    DfuVerified,
    DfuFlashing,
    DfuFailed,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct IdnSnapshot {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub manufacturer: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub model: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub serial: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub fw_version: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub hw_version: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ChannelSnapshot {
    #[serde(default, alias = "voltage_set")]
    pub voltage: f32,
    #[serde(default, alias = "current_set")]
    pub current: f32,
    #[serde(default = "default_ovp")]
    pub ovp: f32,
    #[serde(default = "default_ocp")]
    pub ocp: f32,
    #[serde(default, alias = "output_on")]
    pub output: bool,
    #[serde(default)]
    pub prot_latched: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latched_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<[u8; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color_r: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color_g: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color_b: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub measured: Option<MeasuredSnapshot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_ratio: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voltage_droop: Option<f32>,
}

fn default_ovp() -> f32 {
    18.0
}

fn default_ocp() -> f32 {
    1.0
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct MeasuredSnapshot {
    pub voltage: f32,
    pub current: f32,
    pub power: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "lowercase")]
pub enum ControlRequest {
    Ping,
    Status {
        #[serde(default)]
        slot: u8,
    },
    Reset {
        #[serde(default)]
        slot: u8,
    },
    Load {
        #[serde(default)]
        slot: u8,
        state: StateSnapshot,
    },
    #[serde(rename = "release_all")]
    ReleaseAll,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlResponse {
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slot: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<StateSnapshot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ControlResponse {
    pub fn pong() -> Self {
        Self {
            ok: true,
            message: Some("pong".to_owned()),
            slot: None,
            state: None,
            error: None,
        }
    }

    pub fn ok_slot(slot: u8) -> Self {
        Self {
            ok: true,
            message: None,
            slot: Some(slot),
            state: None,
            error: None,
        }
    }

    pub fn status(slot: u8, state: StateSnapshot) -> Self {
        Self {
            ok: true,
            message: None,
            slot: Some(slot),
            state: Some(state),
            error: None,
        }
    }

    pub fn released_all() -> Self {
        Self {
            ok: true,
            message: Some("released_all".to_owned()),
            slot: None,
            state: None,
            error: None,
        }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Self {
            ok: false,
            message: None,
            slot: None,
            state: None,
            error: Some(message.into()),
        }
    }
}
