//! Partition constants for 2 MiB XIP flash.
//!
//! **Keep in sync with** [`linker/memory-bootloader.x`](../linker/memory-bootloader.x) and
//! [`linker/memory-application.x`](../linker/memory-application.x). See README.md §
//! “Keeping the layout in sync”.
//!
//! All `*_START` values are byte offsets from `FLASH_BASE` (the BOOT2 address).

pub const FLASH_SIZE: usize = 2 * 1024 * 1024;
pub const FLASH_BASE: u32 = 0x1000_0000;

pub const BOOT2_SIZE: u32 = 0x100;
pub const BOOTLOADER_FLASH_SIZE: u32 = 24 * 1024 - BOOT2_SIZE;
pub const STATE_SIZE: u32 = 4 * 1024;
pub const ACTIVE_SIZE: u32 = 800 * 1024;
/// DFU must be one erase page (4096 B) larger than ACTIVE for embassy-boot swap.
pub const DFU_SIZE: u32 = ACTIVE_SIZE + 4096;
pub const RESERVED_SIZE: u32 = 156 * 1024;
pub const CONFIG_SIZE: u32 = 256 * 1024;
pub const FACTORY_SIZE: u32 = 4 * 1024;
pub const ERASE_PAGE_SIZE: u32 = 4096;

pub const BOOT2_START: u32 = 0;
pub const BOOTLOADER_FLASH_START: u32 = BOOT2_SIZE;
pub const STATE_START: u32 = 0x6000;
pub const ACTIVE_START: u32 = 0x7000;
pub const DFU_START: u32 = ACTIVE_START + ACTIVE_SIZE;
pub const RESERVED_START: u32 = DFU_START + DFU_SIZE;
pub const CONFIG_START: u32 = RESERVED_START + RESERVED_SIZE;
pub const FACTORY_START: u32 = CONFIG_START + CONFIG_SIZE;

pub const ACTIVE_CAPACITY: u32 = ACTIVE_SIZE;
pub const DFU_CAPACITY: u32 = DFU_SIZE;

pub const RAM_ORIGIN: u32 = 0x2000_0000;
pub const RAM_LENGTH: u32 = 264 * 1024;
