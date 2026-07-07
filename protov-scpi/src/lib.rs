//! ProtoV SCPI wire protocol: parse, encode, block framing, and optional host client.
//!
//! `no_std` on bare-metal; host builds (tests, coverage) use the normal std environment.

#![cfg_attr(all(not(feature = "std"), target_os = "none"), no_std)]

#[cfg(feature = "std")]
extern crate alloc;

pub mod block;
pub mod command;
pub mod encode;
pub mod parse;
pub mod policy;
pub mod types;

#[cfg(feature = "std")]
pub mod client;

#[cfg(feature = "wasm")]
mod wasm;

#[cfg(feature = "wasm")]
pub use wasm::WasmScpiClient;

pub use block::{
    HexFormatError, decode_hex_block, format_hex_block, is_valid_block_len,
    parse_definite_block_at, parse_definite_block_in_ascii,
};
#[cfg(feature = "std")]
pub use block::{encode_definite_block, encode_fwup_appl_line, encode_fwup_data_line};
pub use command::{
    ChannelParam, FWUP_DATA_PREFIX, FWUP_MAX_BLOCK_LEN, FWUP_MAX_IMAGE_SIZE, FWUP_PAGE_SIZE,
    FWUP_SIGNATURE_LEN, MeasKind, ScpiCommand, TempSlot,
};
pub use encode::encode_command_line;
pub use parse::{normalize_command, parse_command};
pub use policy::{is_allowed_in_update_mode, is_mutation, requires_active_update_session};
pub use types::{
    LINE_BUF, RESPONSE_BUF, RegisterChannel, Rgb, ScpiChannel, ScpiResponse, format_rgb,
    parse_brightness, parse_rgb_triplet,
};

#[cfg(feature = "std")]
pub use client::{ClientError, IoTransport, ScpiClient, Transport, TransportError};
