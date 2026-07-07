//! Product identity and SCPI/USB metadata (not user-persisted).

use core::fmt::Write;

use super::provisioned::{
    hardware_revision, manufacturing_date, serial_attestation, serial_number,
};

pub const MANUFACTURER: &str = "FBRD Inc.";
pub const PRODUCT_NAME: &str = "ProtoV MINI";

// Raspberry PI assigned PID and VID allocation for USB-IF
// https://github.com/raspberrypi/usb-pid
// ProtoV MINI
pub const USB_VID: u16 = 0x2E8A;
pub const USB_PID: u16 = 0x112B;
pub const USB_MAX_POWER_MA: u16 = 500;

pub const FIRMWARE_REVISION: &str = env!("CARGO_PKG_VERSION");

pub const SCPI_SYSTEM_VERSION: &str = "1999.0";

/// RP2040 flash unique ID placeholder until HAL reads hardware at boot.
pub const DEFAULT_FLASH_UNIQUE_ID: [u8; 8] = [0; 8];

pub fn format_idn<const N: usize>(buf: &mut heapless::String<N>) {
    format_idn_parts(serial_number(), FIRMWARE_REVISION, hardware_revision(), buf);
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

/// `{serial},{hw_version},{YYYY-MM-DD},#H<16 hex>,#H<128 hex>` — authenticated device identification.
pub fn format_idat_parts<const N: usize>(
    serial: &str,
    hw_version: &str,
    mfg_date: (u16, u8, u8),
    flash_unique_id: &[u8; 8],
    serial_signature: &[u8; 64],
    buf: &mut heapless::String<N>,
) -> Result<(), protov_scpi::HexFormatError> {
    use protov_scpi::format_hex_block;

    let (year, month, day) = mfg_date;
    let _ = write!(buf, "{serial},{hw_version},{year:04}-{month:02}-{day:02},");
    format_hex_block(flash_unique_id, buf)?;
    buf.push(',').map_err(|_| protov_scpi::HexFormatError)?;
    format_hex_block(serial_signature, buf)
}

pub fn format_idat<const N: usize>(
    buf: &mut heapless::String<N>,
) -> Result<(), protov_scpi::HexFormatError> {
    format_idat_parts(
        serial_number(),
        hardware_revision(),
        manufacturing_date(),
        &DEFAULT_FLASH_UNIQUE_ID,
        serial_attestation(),
        buf,
    )
}
