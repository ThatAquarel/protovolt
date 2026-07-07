// Root key used to sign PUBLIC_KEY_CIX manifest
pub const PUBLIC_KEY_ROOT: &[u8; 32] = include_bytes!("../keys/protov_public.key");

// Manifest containing three public keys for CI release signing
pub const PUBLIC_KEY_CIX: &[u8; 96] = include_bytes!("../keys/protov_public_CIx.key");
// Signature of PUBLIC_KEY_CIX manifest using root key
pub const PUBLIC_KEY_CIX_SIG: &[u8; 64] = include_bytes!("../keys/protov_public_CIx.key.sig");
