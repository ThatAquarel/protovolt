//! Central firmware configuration: product identity, factory defaults, and protection thresholds.
//!
//! Future EEPROM persistence will load/save a `FactorySettings`-shaped snapshot (plus checksum);
//! runtime overrides remain in [`crate::app::App`] and [`crate::scpi::state::ScpiState`].

mod appearance;
mod channel;
mod hardware;
mod product;
mod protection;
mod provisioned;
mod telemetry;

#[cfg(any(test, feature = "simulator"))]
mod identity;

pub use appearance::{
    AppearanceDefaults, DARKEN_DEN, DARKEN_NUM, DEFAULT as APPEARANCE_DEFAULT, DEFAULT_CH1,
    DEFAULT_CH2, DEFAULT_LCD_BRIGHTNESS, DEFAULT_LED_BRIGHTNESS, Rgb,
};
pub use channel::{
    CH1_FACTORY, CH2_FACTORY, CURRENT_EDIT_RANGE, ChannelProfile, VOLTAGE_EDIT_RANGE,
};
pub use hardware::{HardwareProfile, PROFILE as HARDWARE_PROFILE};
#[cfg(any(test, feature = "simulator"))]
pub use identity::{DeviceIdentity, format_idat_with, format_idn_with};
pub use product::{
    DEFAULT_FLASH_UNIQUE_ID, FIRMWARE_REVISION, MANUFACTURER, PRODUCT_NAME, SCPI_SYSTEM_VERSION,
    USB_MAX_POWER_MA, USB_PID, USB_VID, format_idat, format_idat_parts, format_idn,
    format_idn_parts,
};
pub use protection::{CH_OTP_C, MCU_OTP_C};
pub use provisioned::{
    DEFAULT_MANUFACTURING_DATE, DEFAULT_SERIAL_NUMBER, default_hardware_revision,
    hardware_revision, init as init_product_identity, init_from_factory_flash, manufacturing_date,
    manufacturing_day, manufacturing_month, manufacturing_year, serial_attestation, serial_number,
};
pub use telemetry::{BOOT_TEMP_CH_A, BOOT_TEMP_CH_B, BOOT_TEMP_MCU};

#[derive(Clone, Copy, Debug)]
pub struct FactorySettings {
    pub ch1: ChannelProfile,
    pub ch2: ChannelProfile,
    pub appearance: AppearanceDefaults,
}

pub const FACTORY: FactorySettings = FactorySettings {
    ch1: CH1_FACTORY,
    ch2: CH2_FACTORY,
    appearance: APPEARANCE_DEFAULT,
};

#[cfg(test)]
mod tests;
