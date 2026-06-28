//! Manufacturing NVM baseline for STUSB4500 (program once, not every boot).

/// Default ST reference sectors with REQ_SRC_CURRENT=1 and 20 V / 15 V / 5 V ladder.
///
/// Sector 4 byte 6: bit 4 = REQ_SRC_CURRENT, bit 3 = POWER_ONLY_ABOVE_5V (0 = allow 5 V path).
pub const MANUFACTURING_SECTORS: [[u8; 8]; 5] = [
    [0x00, 0x00, 0xB0, 0xAA, 0x00, 0x45, 0x00, 0x00],
    [0x10, 0x40, 0x9C, 0x1C, 0xFF, 0x01, 0x3C, 0xDF],
    [0x02, 0x40, 0x0F, 0x00, 0x32, 0x00, 0xFC, 0xF1],
    // PDO1 5V/3A (0x56 nibble), 3 PDOs (0x06), REQ_SRC in sector 4
    [0x00, 0x19, 0x56, 0xAF, 0xF5, 0x35, 0x5F, 0x00],
    // PDO2 15V, PDO3 20V, REQ_SRC_CURRENT=1 (0x10), flex current
    [0x00, 0x4B, 0x90, 0x21, 0x43, 0x00, 0x50, 0xFB],
];

/// Returns true if the loaded NVM sectors already match our manufacturing baseline.
pub fn sectors_match_baseline(sectors: &[[u8; 8]; 5]) -> bool {
    sectors == &MANUFACTURING_SECTORS
}
