use super::layout::*;

/// RP2040 / embassy-rp [`Flash::WRITE_SIZE`](https://docs.embassy.dev/embassy-rp/git/rp2040/flash/struct.Flash.html).
const EMBASSY_RP_FLASH_WRITE_SIZE: u32 = 1;

/// Default `BootLoader` swap buffer on RP2040 (`embassy-boot-rp`: `BUFFER_SIZE = ERASE_SIZE`).
const EMBASSY_BOOT_ALIGNED_BUF_LEN: u32 = ERASE_PAGE_SIZE;

/// embassy-boot `BootLoader::PAGE_SIZE` (`max(ACTIVE::ERASE_SIZE, DFU::ERASE_SIZE)`).
const fn embassy_boot_page_size(active_erase_size: u32, dfu_erase_size: u32) -> u32 {
    if active_erase_size > dfu_erase_size {
        active_erase_size
    } else {
        dfu_erase_size
    }
}

/// Mirrors compile-time and runtime checks at the start of `prepare_boot` in
/// `embassy-boot/src/boot_loader.rs`.
fn assert_embassy_boot_prepare_boot(
    page_size: u32,
    active_write_size: u32,
    active_erase_size: u32,
    dfu_write_size: u32,
    dfu_erase_size: u32,
    state_write_size: u32,
    aligned_buf_len: u32,
) {
    assert_eq!(
        page_size % active_write_size,
        0,
        "PAGE_SIZE must be a multiple of ACTIVE write size"
    );
    assert_eq!(
        page_size % active_erase_size,
        0,
        "PAGE_SIZE must be a multiple of ACTIVE erase size"
    );
    assert_eq!(
        page_size % dfu_write_size,
        0,
        "PAGE_SIZE must be a multiple of DFU write size"
    );
    assert_eq!(
        page_size % dfu_erase_size,
        0,
        "PAGE_SIZE must be a multiple of DFU erase size"
    );

    assert_eq!(
        page_size % aligned_buf_len,
        0,
        "PAGE_SIZE must be a multiple of swap buffer length"
    );
    assert!(
        aligned_buf_len >= state_write_size,
        "swap buffer must hold at least one STATE write unit"
    );
    assert_eq!(
        aligned_buf_len % active_write_size,
        0,
        "swap buffer must be aligned to ACTIVE write size"
    );
    assert_eq!(
        aligned_buf_len % dfu_write_size,
        0,
        "swap buffer must be aligned to DFU write size"
    );
}

/// Mirrors [`assert_partitions`] in `embassy-boot/src/boot_loader.rs`.
fn assert_embassy_boot_partitions(
    active_capacity: u32,
    dfu_capacity: u32,
    state_capacity: u32,
    page_size: u32,
    state_write_size: u32,
) {
    assert_eq!(
        active_capacity % page_size,
        0,
        "ACTIVE capacity must be a multiple of page size"
    );
    assert_eq!(
        dfu_capacity % page_size,
        0,
        "DFU capacity must be a multiple of page size"
    );
    assert!(
        dfu_capacity - active_capacity >= page_size,
        "DFU partition must be at least one page larger than ACTIVE"
    );
    assert!(
        2 + 4 * (active_capacity / page_size) <= state_capacity / state_write_size,
        "BOOTLOADER_STATE partition too small for embassy-boot swap metadata"
    );
}

#[test]
fn partition_chain_covers_two_megabytes() {
    assert_eq!(BOOT2_START, 0);
    assert_eq!(BOOTLOADER_FLASH_START, 0x100);
    assert_eq!(STATE_START, 0x6000);
    assert_eq!(ACTIVE_START, 0x7000);
    assert_eq!(DFU_START, 0xCF_000);
    assert_eq!(RESERVED_START, 0x198_000);
    assert_eq!(CONFIG_START, 0x1BF_000);
    assert_eq!(FACTORY_START, 0x1FF_000);
    assert_eq!(FACTORY_START + FACTORY_SIZE, FLASH_SIZE as u32);
}

#[test]
fn dfu_larger_than_active_by_one_page() {
    assert_eq!(DFU_SIZE, ACTIVE_SIZE + ERASE_PAGE_SIZE);
}

#[test]
fn embassy_boot_partition_requirements() {
    let flash_erase = ERASE_PAGE_SIZE;
    let page_size = embassy_boot_page_size(flash_erase, flash_erase);

    assert_embassy_boot_prepare_boot(
        page_size,
        EMBASSY_RP_FLASH_WRITE_SIZE,
        flash_erase,
        EMBASSY_RP_FLASH_WRITE_SIZE,
        flash_erase,
        EMBASSY_RP_FLASH_WRITE_SIZE,
        EMBASSY_BOOT_ALIGNED_BUF_LEN,
    );

    assert_embassy_boot_partitions(
        ACTIVE_CAPACITY,
        DFU_CAPACITY,
        STATE_SIZE,
        page_size,
        EMBASSY_RP_FLASH_WRITE_SIZE,
    );
}
