//! Embedded trust-chain public keys and manifest verification.

use ed25519_dalek::{Signature, Verifier, VerifyingKey};

// Root key used to sign PUBLIC_KEY_CIX manifest
pub const PUBLIC_KEY_ROOT: &[u8; 32] = include_bytes!("../keys/protov_public.key");

// Manifest containing three public keys for CI release signing
pub const PUBLIC_KEY_CIX: &[u8; 96] = include_bytes!("../keys/protov_public_CIx.key");
// Signature of PUBLIC_KEY_CIX manifest using root key
pub const PUBLIC_KEY_CIX_SIG: &[u8; 64] = include_bytes!("../keys/protov_public_CIx.key.sig");

pub const CI_KEY_LEN: usize = 32;
pub const CI_KEY_COUNT: usize = 3;

/// CI manifest signature verification failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestVerifyError {
    InvalidRootKey,
    InvalidSignature,
}

/// Split the embedded CI manifest into three raw Ed25519 public keys (CI0, CI1, CI2).
pub fn ci_public_keys() -> [[u8; CI_KEY_LEN]; CI_KEY_COUNT] {
    let mut keys = [[0u8; CI_KEY_LEN]; CI_KEY_COUNT];
    for (i, key) in keys.iter_mut().enumerate() {
        let start = i * CI_KEY_LEN;
        key.copy_from_slice(&PUBLIC_KEY_CIX[start..start + CI_KEY_LEN]);
    }
    keys
}

/// Verify the embedded CI manifest signature against the root public key.
pub fn verify_cix_manifest() -> Result<(), ManifestVerifyError> {
    let root = VerifyingKey::from_bytes(PUBLIC_KEY_ROOT)
        .map_err(|_| ManifestVerifyError::InvalidRootKey)?;
    let signature = Signature::from_bytes(PUBLIC_KEY_CIX_SIG);
    root.verify(PUBLIC_KEY_CIX, &signature)
        .map_err(|_| ManifestVerifyError::InvalidSignature)
}
