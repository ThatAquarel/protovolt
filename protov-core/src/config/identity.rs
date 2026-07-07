//! Per-device identity overrides for simulator / multi-slot mock pools.

use super::product::{
    DEFAULT_FLASH_UNIQUE_ID, FIRMWARE_REVISION, format_idat_parts, format_idn_parts,
};
use super::{hardware_revision, manufacturing_date, serial_attestation, serial_number};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceIdentity {
    pub serial: &'static str,
    pub fw_version: &'static str,
    pub hw_version: &'static str,
    pub manufacturing_date: (u16, u8, u8),
    pub flash_unique_id: [u8; 8],
    pub serial_signature: &'static [u8; 64],
}

impl Default for DeviceIdentity {
    fn default() -> Self {
        Self {
            serial: serial_number(),
            fw_version: FIRMWARE_REVISION,
            hw_version: hardware_revision(),
            manufacturing_date: manufacturing_date(),
            flash_unique_id: DEFAULT_FLASH_UNIQUE_ID,
            serial_signature: serial_attestation(),
        }
    }
}

pub fn format_idn_with<const N: usize>(identity: &DeviceIdentity, buf: &mut heapless::String<N>) {
    format_idn_parts(
        identity.serial,
        identity.fw_version,
        identity.hw_version,
        buf,
    );
}

pub fn format_idat_with<const N: usize>(
    identity: &DeviceIdentity,
    buf: &mut heapless::String<N>,
) -> Result<(), protov_scpi::HexFormatError> {
    format_idat_parts(
        identity.serial,
        identity.hw_version,
        identity.manufacturing_date,
        &identity.flash_unique_id,
        identity.serial_signature,
        buf,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{format_idat_parts, format_idn, serial_attestation, serial_number};

    #[test]
    fn default_identity_matches_product_constants() {
        let identity = DeviceIdentity::default();
        assert_eq!(identity.serial, serial_number());
        assert_eq!(identity.serial_signature, serial_attestation());
    }

    #[test]
    fn default_identity_formats_like_product_idn() {
        let identity = DeviceIdentity::default();
        let mut from_identity = heapless::String::<128>::new();
        let mut from_product = heapless::String::<128>::new();
        super::format_idn_with(&identity, &mut from_identity);
        format_idn(&mut from_product);
        assert_eq!(from_identity.as_str(), from_product.as_str());
    }

    #[test]
    fn format_idat_with_custom_serial() {
        let sig = [0xAB; 64];
        let mut buf = heapless::String::<256>::new();
        format_idat_parts("550e8400", "A.1", (2026, 6, 27), &[0x11; 8], &sig, &mut buf).unwrap();
        assert_eq!(
            &buf.as_str()[.."550e8400,A.1,2026-06-27,#H".len()],
            "550e8400,A.1,2026-06-27,#H"
        );
    }

    #[test]
    fn format_idat_parts_emits_serial_hw_and_hex_signature() {
        let sig = [0xAB; 64];
        let mut buf = heapless::String::<256>::new();
        format_idat_parts("550e8400", "A.1", (2026, 6, 27), &[0x11; 8], &sig, &mut buf).unwrap();
        assert!(buf.starts_with("550e8400,A.1,2026-06-27,#H1111111111111111,#H"));
        assert_eq!(
            buf.len(),
            "550e8400,A.1,2026-06-27,#H1111111111111111,#H".len() + 128
        );
        assert!(buf.ends_with("AB"));
    }
}
