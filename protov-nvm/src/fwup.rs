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
