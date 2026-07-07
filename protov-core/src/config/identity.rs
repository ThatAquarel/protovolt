//! Per-device identity overrides for simulator / multi-slot mock pools.

use super::product::{FIRMWARE_REVISION, SERIAL_ATTESTATION, format_idat_parts, format_idn_parts};
use super::{HARDWARE_REVISION, SERIAL_NUMBER};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceIdentity {
    pub serial: &'static str,
    pub fw_version: &'static str,
    pub hw_version: &'static str,
    pub serial_signature: &'static [u8; 64],
}

impl Default for DeviceIdentity {
    fn default() -> Self {
        Self {
            serial: SERIAL_NUMBER,
            fw_version: FIRMWARE_REVISION,
            hw_version: HARDWARE_REVISION,
            serial_signature: &SERIAL_ATTESTATION,
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
        identity.serial_signature,
        buf,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{format_idat_parts, format_idn, SERIAL_ATTESTATION};

    #[test]
    fn default_identity_matches_product_constants() {
        let identity = DeviceIdentity::default();
        assert_eq!(identity.serial, SERIAL_NUMBER);
        assert_eq!(identity.serial_signature, &SERIAL_ATTESTATION);
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
        format_idat_parts("550e8400", "A.1", &sig, &mut buf).unwrap();
        assert_eq!(
            &buf.as_str()[.."550e8400,A.1,#H".len()],
            "550e8400,A.1,#H"
        );
    }

    #[test]
    fn format_idat_parts_emits_serial_hw_and_hex_signature() {
        let sig = [0xAB; 64];
        let mut buf = heapless::String::<256>::new();
        format_idat_parts("550e8400", "A.1", &sig, &mut buf).unwrap();
        assert!(buf.starts_with("550e8400,A.1,#H"));
        assert_eq!(buf.len(), "550e8400,A.1,#H".len() + 128);
        assert!(buf.ends_with("AB"));
    }
}
