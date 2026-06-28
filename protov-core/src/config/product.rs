//! Product identity and SCPI/USB metadata (not user-persisted).

use core::fmt::Write;

pub const MANUFACTURER: &str = "FBRD Inc.";
pub const PRODUCT_NAME: &str = "ProtoV MINI";
pub const SERIAL_NUMBER: &str = "00000011";

pub const USB_VID: u16 = 0x2E8A;
pub const USB_PID: u16 = 0x111F;
pub const USB_MAX_POWER_MA: u16 = 500;

pub use super::hardware::PROFILE as HARDWARE_PROFILE;
pub const HARDWARE_REVISION: &str = HARDWARE_PROFILE.revision;
pub const FIRMWARE_REVISION: &str = env!("CARGO_PKG_VERSION");

pub const SCPI_SYSTEM_VERSION: &str = "1999.0";

pub fn format_idn<const N: usize>(buf: &mut heapless::String<N>) {
    let _ = write!(
        buf,
        "{},{},{},{},{}",
        MANUFACTURER, PRODUCT_NAME, SERIAL_NUMBER, FIRMWARE_REVISION, HARDWARE_REVISION,
    );
}
