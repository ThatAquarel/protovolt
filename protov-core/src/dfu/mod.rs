mod block;
mod error;
mod session;

pub use block::{
    decode_hex_block, is_valid_block_len, parse_definite_block_at, parse_definite_block_in_ascii,
};
pub use error::DfuError;
pub use session::{DfuAction, DfuPhase, DfuSession};

pub use protov_scpi::{
    FWUP_MAX_BLOCK_LEN, FWUP_MAX_IMAGE_SIZE, FWUP_PAGE_SIZE, FWUP_SIGNATURE_LEN,
};
