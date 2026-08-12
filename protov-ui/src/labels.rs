// Boot
pub const INPUT: &str = "INPUT";
pub const PD: &str = "USB-C PD";
pub const STD: &str = "USB 2.0";

pub const SENSE: &str = "SENSE";
pub const CONVERTER: &str = "CONVERTER";

pub const PASS: &str = "PASS";
pub const FAIL: &str = "FAIL";

#[cfg(feature = "demo")]
pub const DEMO: &str = concat!("DEMO BUILD v", env!("CARGO_PKG_VERSION"));

// Controls
pub const CHANNEL_A: &str = "CHANNEL A";
pub const CHANNEL_B: &str = "CHANNEL B";

pub const VOLT: &str = "V";
pub const AMPERE: &str = "A";
pub const WATT: &str = "W";

pub const SET: &str = "SET";
pub const OVP: &str = "OVP";
pub const OCP: &str = "OCP";

// Channel hardware state
pub const CONSTANT_VOLTAGE: &str = "CV";
pub const CONSTANT_CURRENT: &str = "CC";

pub const SHORT_CIRCUIT: &str = "SHORT";
pub const OVER_TEMPERATURE: &str = "TEMP";
pub const OVER_CURRENT: &str = "OCP";
pub const OVER_VOLTAGE: &str = "OVP";

// Settings
pub const SETTINGS: &str = "SETTINGS";
pub const MANAGE_AT: &str = "MANAGE AT";
pub const WEBSITE: &str = "www.protov.app";
pub const FW_VERSION: &str = "FW VERSION";
pub const HW_VERSION: &str = "HW VERSION";
pub const SERIAL_NUMBER: &str = "SERIAL NUMBER";

// Firmware update
pub const DFU_PREPARING: &str = "PREPARING.";
pub const DFU_TRANSFERRING: &str = "TRANSFERRING.";
pub const DFU_VERIFIED: &str = "VERIFIED.";
pub const DFU_FLASHING: &str = "BOOTLOADER FLASHING.";
pub const DFU_DO_NOT_DISCONNECT: &str = "DO NOT DISCONNECT.";
pub const DFU_FAILED: &str = "UPDATE FAILED.";
