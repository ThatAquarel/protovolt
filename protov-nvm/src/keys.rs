//! Embedded trust-chain public keys and manifest verification.

use ed25519_dalek::{Signature, Verifier, VerifyingKey};

use crate::attestation::encode_attestation_message;

// Root key used to sign trust manifests (CIx, HWx, …)
pub const PUBLIC_KEY_ROOT: &[u8; 32] = include_bytes!("../keys/protov_public.key");

// Manifest containing three public keys for CI release signing
pub const PUBLIC_KEY_CIX: &[u8; 96] = include_bytes!("../keys/protov_public_CIx.key");
// Signature of PUBLIC_KEY_CIX manifest using root key
pub const PUBLIC_KEY_CIX_SIG: &[u8; 64] = include_bytes!("../keys/protov_public_CIx.key.sig");

// Manifest containing three public keys for hardware serial attestation
pub const PUBLIC_KEY_HWX: &[u8; 96] = include_bytes!("../keys/protov_public_HWx.key");
// Signature of PUBLIC_KEY_HWX manifest using root key
pub const PUBLIC_KEY_HWX_SIG: &[u8; 64] = include_bytes!("../keys/protov_public_HWx.key.sig");

pub const MANIFEST_KEY_LEN: usize = 32;
pub const MANIFEST_KEY_COUNT: usize = 3;

pub const CI_KEY_LEN: usize = MANIFEST_KEY_LEN;
pub const CI_KEY_COUNT: usize = MANIFEST_KEY_COUNT;
pub const HW_KEY_LEN: usize = MANIFEST_KEY_LEN;
pub const HW_KEY_COUNT: usize = MANIFEST_KEY_COUNT;

/// Trust manifest signature verification failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestVerifyError {
    InvalidRootKey,
    InvalidSignature,
}

/// Split a 96-byte trust manifest into three raw Ed25519 public keys.
pub fn manifest_public_keys(
    manifest: &[u8; MANIFEST_KEY_LEN * MANIFEST_KEY_COUNT],
) -> [[u8; MANIFEST_KEY_LEN]; MANIFEST_KEY_COUNT] {
    let mut keys = [[0u8; MANIFEST_KEY_LEN]; MANIFEST_KEY_COUNT];
    for (i, key) in keys.iter_mut().enumerate() {
        let start = i * MANIFEST_KEY_LEN;
        key.copy_from_slice(&manifest[start..start + MANIFEST_KEY_LEN]);
    }
    keys
}

/// Split the embedded CI manifest into three raw Ed25519 public keys (CI0, CI1, CI2).
pub fn ci_public_keys() -> [[u8; CI_KEY_LEN]; CI_KEY_COUNT] {
    manifest_public_keys(PUBLIC_KEY_CIX)
}

/// Split the embedded HW manifest into three raw Ed25519 public keys (HW0, HW1, HW2).
pub fn hw_public_keys() -> [[u8; HW_KEY_LEN]; HW_KEY_COUNT] {
    manifest_public_keys(PUBLIC_KEY_HWX)
}

fn verify_manifest(manifest: &[u8], signature: &[u8; 64]) -> Result<(), ManifestVerifyError> {
    let root = VerifyingKey::from_bytes(PUBLIC_KEY_ROOT)
        .map_err(|_| ManifestVerifyError::InvalidRootKey)?;
    let signature = Signature::from_bytes(signature);
    root.verify(manifest, &signature)
        .map_err(|_| ManifestVerifyError::InvalidSignature)
}

/// Verify the embedded CI manifest signature against the root public key.
pub fn verify_cix_manifest() -> Result<(), ManifestVerifyError> {
    verify_manifest(PUBLIC_KEY_CIX, PUBLIC_KEY_CIX_SIG)
}

/// Verify the embedded HW manifest signature against the root public key.
pub fn verify_hwx_manifest() -> Result<(), ManifestVerifyError> {
    verify_manifest(PUBLIC_KEY_HWX, PUBLIC_KEY_HWX_SIG)
}

/// Serial attestation verification failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerialAttestationError {
    InvalidHwManifest,
    InvalidMessage,
    InvalidSignature,
}

/// Verify an Ed25519 signature over the canonical attestation message using the embedded HW key set.
///
/// The HW manifest must be root-signed (`verify_hwx_manifest`). The signed payload is
/// `{serial},{hw_revision},{YYYY-MM-DD}` (see [`crate::encode_attestation_message`]).
/// Verification succeeds when any trusted HW key validates the signature (raw, no pre-hash).
pub fn verify_serial_attestation(
    serial: &str,
    hw_revision: &str,
    manufacturing_date: (u16, u8, u8),
    signature: &[u8; 64],
) -> Result<(), SerialAttestationError> {
    verify_hwx_manifest().map_err(|_| SerialAttestationError::InvalidHwManifest)?;
    let message = encode_attestation_message(serial, hw_revision, manufacturing_date)
        .map_err(|_| SerialAttestationError::InvalidMessage)?;
    let sig = Signature::from_bytes(signature);
    for key in hw_public_keys() {
        let Ok(verifier) = VerifyingKey::from_bytes(&key) else {
            continue;
        };
        if verifier.verify(message.as_bytes(), &sig).is_ok() {
            return Ok(());
        }
    }
    Err(SerialAttestationError::InvalidSignature)
}
