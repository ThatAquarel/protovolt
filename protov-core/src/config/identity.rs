//! Per-device identity overrides for simulator / multi-slot mock pools.

use core::fmt::Write;

use super::product::FIRMWARE_REVISION;
use super::{HARDWARE_REVISION, MANUFACTURER, PRODUCT_NAME, SERIAL_NUMBER};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceIdentity {
    pub serial: &'static str,
    pub fw_version: &'static str,
    pub hw_version: &'static str,
}

impl Default for DeviceIdentity {
    fn default() -> Self {
        Self {
            serial: SERIAL_NUMBER,
            fw_version: FIRMWARE_REVISION,
            hw_version: HARDWARE_REVISION,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{FIRMWARE_REVISION, HARDWARE_REVISION, SERIAL_NUMBER, format_idn};

    #[test]
    fn default_identity_matches_product_constants() {
        let identity = DeviceIdentity::default();
        assert_eq!(identity.serial, SERIAL_NUMBER);
        assert_eq!(identity.fw_version, FIRMWARE_REVISION);
        assert_eq!(identity.hw_version, HARDWARE_REVISION);
    }

    #[test]
    fn default_identity_formats_like_product_idn() {
        let identity = DeviceIdentity::default();
        let mut from_identity = heapless::String::<128>::new();
        let mut from_product = heapless::String::<128>::new();
        format_idn_with(&identity, &mut from_identity);
        format_idn(&mut from_product);
        assert_eq!(from_identity.as_str(), from_product.as_str());
    }

    #[test]
    fn format_idn_with_custom_serial() {
        let identity = DeviceIdentity {
            serial: "550e8400",
            fw_version: "1.0.0",
            hw_version: "A.1",
        };
        let mut buf = heapless::String::<128>::new();
        format_idn_with(&identity, &mut buf);
        assert_eq!(buf.as_str(), "FBRD Inc.,ProtoV MINI,550e8400,1.0.0,A.1");
    }
}
