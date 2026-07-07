//! Canonical attestation message encoding for HW Ed25519 signatures.

use core::fmt::{self, Write};

pub const ATTESTATION_MESSAGE_CAP: usize = 64;

/// UTF-8 message signed by HW keys: `{serial},{hw_revision},{YYYY-MM-DD}`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttestationMessage {
    len: u8,
    bytes: [u8; ATTESTATION_MESSAGE_CAP],
}

impl AttestationMessage {
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len as usize]
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(self.as_bytes()).expect("attestation message is ASCII")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttestationMessageError {
    EmptyField,
    SerialTooLong,
    HwRevisionTooLong,
    InvalidDate,
    NonAscii,
    BufferTooSmall,
}

struct MessageWriter<'a> {
    buf: &'a mut [u8],
    len: usize,
}

impl Write for MessageWriter<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self.len + s.len() > self.buf.len() {
            return Err(fmt::Error);
        }
        self.buf[self.len..self.len + s.len()].copy_from_slice(s.as_bytes());
        self.len += s.len();
        Ok(())
    }
}

/// Build the canonical attestation bytes for Ed25519 sign/verify (raw, no pre-hash).
pub fn encode_attestation_message(
    serial: &str,
    hw_revision: &str,
    manufacturing_date: (u16, u8, u8),
) -> Result<AttestationMessage, AttestationMessageError> {
    validate_field(
        serial,
        AttestationMessageError::EmptyField,
        16,
        AttestationMessageError::SerialTooLong,
    )?;
    validate_field(
        hw_revision,
        AttestationMessageError::EmptyField,
        8,
        AttestationMessageError::HwRevisionTooLong,
    )?;
    let (year, month, day) = manufacturing_date;
    validate_date(year, month, day)?;

    let mut bytes = [0u8; ATTESTATION_MESSAGE_CAP];
    let mut writer = MessageWriter {
        buf: &mut bytes,
        len: 0,
    };
    write!(
        writer,
        "{serial},{hw_revision},{year:04}-{month:02}-{day:02}"
    )
    .map_err(|_| AttestationMessageError::BufferTooSmall)?;

    Ok(AttestationMessage {
        len: writer.len as u8,
        bytes,
    })
}

fn validate_field(
    value: &str,
    empty: AttestationMessageError,
    max_len: usize,
    too_long: AttestationMessageError,
) -> Result<(), AttestationMessageError> {
    if value.is_empty() {
        return Err(empty);
    }
    if value.len() > max_len {
        return Err(too_long);
    }
    if !value.is_ascii() {
        return Err(AttestationMessageError::NonAscii);
    }
    Ok(())
}

fn validate_date(year: u16, month: u8, day: u8) -> Result<(), AttestationMessageError> {
    if (1..=12).contains(&month) && (1..=31).contains(&day) && year >= 2000 {
        Ok(())
    } else {
        Err(AttestationMessageError::InvalidDate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};

    const TEST_SIGNING_KEY: [u8; 32] = [0xA5; 32];

    #[test]
    fn example_production_unit_message() {
        let msg = encode_attestation_message("550e8400", "A.1", (2026, 6, 27)).unwrap();
        assert_eq!(msg.as_str(), "550e8400,A.1,2026-06-27");
    }

    #[test]
    fn example_second_serial_and_revision() {
        let msg = encode_attestation_message("32983fe4", "B.2", (2025, 12, 1)).unwrap();
        assert_eq!(msg.as_str(), "32983fe4,B.2,2025-12-01");
    }

    #[test]
    fn serial_change_changes_message() {
        let a = encode_attestation_message("550e8400", "A.1", (2026, 6, 27)).unwrap();
        let b = encode_attestation_message("550e8401", "A.1", (2026, 6, 27)).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn hw_revision_change_changes_message() {
        let a = encode_attestation_message("550e8400", "A.1", (2026, 6, 27)).unwrap();
        let b = encode_attestation_message("550e8400", "A.2", (2026, 6, 27)).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn date_change_changes_message() {
        let a = encode_attestation_message("550e8400", "A.1", (2026, 6, 27)).unwrap();
        let b = encode_attestation_message("550e8400", "A.1", (2026, 6, 28)).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn sign_and_verify_roundtrip() {
        let message = encode_attestation_message("deadbeef", "C.1", (2024, 3, 15)).unwrap();
        let signing_key = SigningKey::from_bytes(&TEST_SIGNING_KEY);
        let signature = signing_key.sign(message.as_bytes());
        let verifying_key =
            VerifyingKey::from_bytes(&signing_key.verifying_key().to_bytes()).unwrap();
        verifying_key
            .verify(message.as_bytes(), &signature)
            .expect("signature must verify canonical attestation message");
    }

    #[test]
    fn signature_invalid_if_hw_revision_mismatch() {
        let signed = encode_attestation_message("deadbeef", "C.1", (2024, 3, 15)).unwrap();
        let claimed = encode_attestation_message("deadbeef", "C.2", (2024, 3, 15)).unwrap();
        let signing_key = SigningKey::from_bytes(&TEST_SIGNING_KEY);
        let signature = signing_key.sign(signed.as_bytes());
        let verifying_key =
            VerifyingKey::from_bytes(&signing_key.verifying_key().to_bytes()).unwrap();
        assert!(
            verifying_key
                .verify(claimed.as_bytes(), &signature)
                .is_err()
        );
    }
}
