//! Central firmware configuration: product identity, factory defaults, and protection thresholds.
//!
//! Future EEPROM persistence will load/save a `FactorySettings`-shaped snapshot (plus checksum);
//! runtime overrides remain in [`crate::app::App`] and [`crate::scpi::state::ScpiState`].

mod appearance;
mod channel;
mod product;
mod protection;
mod telemetry;

pub use appearance::{
    AppearanceDefaults, DARKEN_DEN, DARKEN_NUM, DEFAULT as APPEARANCE_DEFAULT, DEFAULT_CH1,
    DEFAULT_CH2, DEFAULT_LCD_BRIGHTNESS, DEFAULT_LED_BRIGHTNESS, Rgb,
};
pub use channel::{
    ChannelProfile, CH1_FACTORY, CH2_FACTORY, CURRENT_EDIT_RANGE, VOLTAGE_EDIT_RANGE,
};
pub use product::{
    format_idn, MANUFACTURER, PRODUCT_NAME, SCPI_REVISION, SCPI_SYSTEM_VERSION, SERIAL_NUMBER,
    USB_MAX_POWER_MA, USB_PID, USB_VID,
};
pub use protection::{CH_OTP_C, MCU_OTP_C};
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
