use super::layout::*;

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
