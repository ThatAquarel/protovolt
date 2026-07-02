//! Non-volatile memory layout for ProtoV (2 MiB XIP flash).
#![no_std]
//!
//! Partition map lives in three places that must stay consistent:
//!
//! - [`linker/memory-bootloader.x`](../linker/memory-bootloader.x)
//! - [`linker/memory-application.x`](../linker/memory-application.x)
//! - [`layout`](layout) (`src/layout.rs`)
//!
//! See the crate README for the sync checklist.
//!
//! # Firmware crates
//!
//! ```toml
//! # Bootloader Cargo.toml
//! protov-nvm = { path = "../protov-nvm", features = ["bootloader"] }
//!
//! # Application Cargo.toml
//! protov-nvm = { path = "../protov-nvm", features = ["application"] }
//! ```
//!
//! ```ignore
//! // In the firmware crate's build.rs:
//! fn main() {
//!     let nvm = std::env::var("DEP_PROTOV_NVM_OUT_DIR").unwrap();
//!     println!("cargo:rustc-link-search={nvm}");
//!     println!("cargo:rustc-link-arg-bins=-Tlink.x");
//! }
//! ```

pub mod layout;

pub use layout::*;

#[cfg(test)]
mod layout_tests;
