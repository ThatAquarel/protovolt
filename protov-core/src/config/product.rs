//! Product identity and SCPI/USB metadata (not user-persisted).

use core::fmt::Write;

pub const MANUFACTURER: &str = "FBRD Inc.";
pub const PRODUCT_NAME: &str = "ProtoV MINI";
pub const SERIAL_NUMBER: &str = "00000011";

// Raspberry PI assigned PID and VID allocation for USB-IF
// https://github.com/raspberrypi/usb-pid
// ProtoV MINI
pub const USB_VID: u16 = 0x2E8A;
pub const USB_PID: u16 = 0x112B;
pub const USB_MAX_POWER_MA: u16 = 500;

pub use super::hardware::PROFILE as HARDWARE_PROFILE;
pub const HARDWARE_REVISION: &str = HARDWARE_PROFILE.revision;
pub const FIRMWARE_REVISION: &str = env!("CARGO_PKG_VERSION");

pub const SCPI_SYSTEM_VERSION: &str = "1999.0";

/// Ed25519 signature over the device serial number, provisioned at manufacturing.
/// Replace via `include_bytes!` or build-time injection for production units.
pub const SERIAL_ATTESTATION: [u8; 64] = [0u8; 64];

pub fn format_idn<const N: usize>(buf: &mut heapless::String<N>) {
    format_idn_parts(
        SERIAL_NUMBER,
        FIRMWARE_REVISION,
        HARDWARE_REVISION,
        buf,
    );
}

pub fn format_idn_parts<const N: usize>(
    serial: &str,
    fw_version: &str,
    hw_version: &str,
    buf: &mut heapless::String<N>,
) {
    let _ = write!(
        buf,
        "{},{},{},{},{}",
        MANUFACTURER, PRODUCT_NAME, serial, fw_version, hw_version,
    );
}

/// `{serial},{hw_version},#H<128 hex>` — authenticated device identification.
pub fn format_idat_parts<const N: usize>(
    serial: &str,
    hw_version: &str,
    serial_signature: &[u8; 64],
    buf: &mut heapless::String<N>,
) -> Result<(), protov_scpi::HexFormatError> {
    use protov_scpi::format_hex_block;

    let _ = write!(buf, "{serial},{hw_version},");
    format_hex_block(serial_signature, buf)
}

pub fn format_idat<const N: usize>(buf: &mut heapless::String<N>) -> Result<(), protov_scpi::HexFormatError> {
    format_idat_parts(
        SERIAL_NUMBER,
        HARDWARE_REVISION,
        &SERIAL_ATTESTATION,
        buf,
    )
}
