//! SCPI firmware-update protocol limits derived from flash layout.

use crate::layout::{ACTIVE_SIZE, ERASE_PAGE_SIZE};

/// Maximum firmware image size accepted over SCPI (ACTIVE partition capacity).
pub const FWUP_MAX_IMAGE_SIZE: u32 = ACTIVE_SIZE;

/// One SCPI DATA block equals one flash erase page.
pub const FWUP_PAGE_SIZE: u32 = ERASE_PAGE_SIZE;

/// Maximum payload bytes per `SYST:FWUP:DATA` block.
pub const FWUP_MAX_BLOCK_LEN: usize = ERASE_PAGE_SIZE as usize;

/// Ed25519 signature length for `SYST:FWUP:APPL`.
pub const FWUP_SIGNATURE_LEN: usize = 64;

/// PUBLIC Key for software binaries.
pub const PUBLIC_KEY: &[u8; 32] = &[
    0xe6, 0x35, 0xea, 0x50, 0xff, 0xcc, 0xb4, 0x1a, 0xde, 0xf2, 0xb2, 0x5c, 0x78, 0x44, 0x89, 0xf1,
    0xd1, 0x76, 0x5d, 0x7e, 0x5b, 0xac, 0x3f, 0x21, 0x26, 0x13, 0x12, 0x1f, 0x6a, 0x2f, 0xda, 0xcc,
];
