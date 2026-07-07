//! Factory flash sector: provisioned serial, hardware revision, date, and attestation.

use crate::layout::{ERASE_PAGE_SIZE, FACTORY_START, FLASH_BASE};

pub const FACTORY_MAGIC: [u8; 4] = *b"PFAC";
pub const FACTORY_VERSION: u8 = 1;

pub const FACTORY_HW_REV_MAX: usize = 8;
pub const FACTORY_SERIAL_MAX: usize = 16;
pub const FACTORY_SIGNATURE_LEN: usize = 64;

/// On-flash record size (256 B — padded for programming; sector is 4 KiB).
pub const FACTORY_RECORD_SIZE: usize = 256;

pub const FACTORY_XIP_ADDR: u32 = FLASH_BASE + FACTORY_START;

/// Binary layout at the start of the FACTORY partition (`__factory_start` in the linker script).
#[repr(C, align(4))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FactoryRecord {
    pub magic: [u8; 4],
    pub version: u8,
    pub hw_rev_len: u8,
    pub serial_len: u8,
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub _pad: u8,
    pub hw_rev: [u8; FACTORY_HW_REV_MAX],
    pub serial: [u8; FACTORY_SERIAL_MAX],
    pub signature: [u8; FACTORY_SIGNATURE_LEN],
}

impl FactoryRecord {
    pub const fn blank() -> Self {
        Self {
            magic: [0xFF; 4],
            version: 0xFF,
            hw_rev_len: 0,
            serial_len: 0,
            year: 0xFFFF,
            month: 0xFF,
            day: 0xFF,
            _pad: 0xFF,
            hw_rev: [0xFF; FACTORY_HW_REV_MAX],
            serial: [0xFF; FACTORY_SERIAL_MAX],
            signature: [0xFF; FACTORY_SIGNATURE_LEN],
        }
    }

    pub fn encode(
        hw_revision: &str,
        serial: &str,
        year: u16,
        month: u8,
        day: u8,
        signature: &[u8; FACTORY_SIGNATURE_LEN],
    ) -> Result<[u8; FACTORY_RECORD_SIZE], FactoryEncodeError> {
        validate_string(hw_revision, FACTORY_HW_REV_MAX)?;
        validate_string(serial, FACTORY_SERIAL_MAX)?;
        validate_date(year, month, day)?;

        let mut record = Self::blank();
        record.magic = FACTORY_MAGIC;
        record.version = FACTORY_VERSION;
        record.hw_rev_len = hw_revision.len() as u8;
        record.serial_len = serial.len() as u8;
        record.year = year;
        record.month = month;
        record.day = day;
        record._pad = 0;
        copy_ascii(hw_revision, &mut record.hw_rev);
        copy_ascii(serial, &mut record.serial);
        record.signature = *signature;

        let mut out = [0xFFu8; FACTORY_RECORD_SIZE];
        out[..core::mem::size_of::<Self>()].copy_from_slice(record.as_bytes());
        Ok(out)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, FactoryDecodeError> {
        if bytes.len() < core::mem::size_of::<Self>() {
            return Err(FactoryDecodeError::TooShort);
        }
        let record = unsafe { core::ptr::read_unaligned(bytes.as_ptr().cast::<Self>()) };
        record.validate()?;
        Ok(record)
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                (self as *const Self).cast::<u8>(),
                core::mem::size_of::<Self>(),
            )
        }
    }

    pub fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }

    pub fn validate(&self) -> Result<(), FactoryDecodeError> {
        if self.magic != FACTORY_MAGIC {
            return Err(FactoryDecodeError::BadMagic);
        }
        if self.version != FACTORY_VERSION {
            return Err(FactoryDecodeError::BadVersion);
        }
        if self.hw_rev_len == 0 || self.hw_rev_len as usize > FACTORY_HW_REV_MAX {
            return Err(FactoryDecodeError::BadHwRevLen);
        }
        if self.serial_len == 0 || self.serial_len as usize > FACTORY_SERIAL_MAX {
            return Err(FactoryDecodeError::BadSerialLen);
        }
        if !(1..=12).contains(&self.month) || !(1..=31).contains(&self.day) || self.year < 2000 {
            return Err(FactoryDecodeError::InvalidDate);
        }
        validate_ascii(&self.hw_rev[..self.hw_rev_len as usize])?;
        validate_ascii(&self.serial[..self.serial_len as usize])?;
        Ok(())
    }

    pub fn hw_revision(&self) -> Result<&str, FactoryDecodeError> {
        self.validate()?;
        core::str::from_utf8(&self.hw_rev[..self.hw_rev_len as usize])
            .map_err(|_| FactoryDecodeError::BadUtf8)
    }

    pub fn serial(&self) -> Result<&str, FactoryDecodeError> {
        self.validate()?;
        core::str::from_utf8(&self.serial[..self.serial_len as usize])
            .map_err(|_| FactoryDecodeError::BadUtf8)
    }

    pub fn manufacturing_date(&self) -> Result<(u16, u8, u8), FactoryDecodeError> {
        self.validate()?;
        Ok((self.year, self.month, self.day))
    }
}

/// Read the factory record from memory-mapped XIP flash.
pub fn read_xip() -> Option<FactoryRecord> {
    let ptr = FACTORY_XIP_ADDR as *const FactoryRecord;
    let record = unsafe { core::ptr::read_unaligned(ptr) };
    record.is_valid().then_some(record)
}

pub fn factory_flash_offset() -> usize {
    FACTORY_START as usize
}

pub fn factory_erase_size() -> usize {
    ERASE_PAGE_SIZE as usize
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactoryEncodeError {
    HwRevisionTooLong,
    SerialTooLong,
    EmptyField,
    InvalidDate,
    NonAscii,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactoryDecodeError {
    TooShort,
    BadMagic,
    BadVersion,
    BadHwRevLen,
    BadSerialLen,
    InvalidDate,
    BadUtf8,
    NonAscii,
}

fn validate_string(value: &str, max: usize) -> Result<(), FactoryEncodeError> {
    if value.is_empty() {
        return Err(FactoryEncodeError::EmptyField);
    }
    if value.len() > max {
        return Err(if max == FACTORY_HW_REV_MAX {
            FactoryEncodeError::HwRevisionTooLong
        } else {
            FactoryEncodeError::SerialTooLong
        });
    }
    if !value.is_ascii() {
        return Err(FactoryEncodeError::NonAscii);
    }
    Ok(())
}

fn validate_date(year: u16, month: u8, day: u8) -> Result<(), FactoryEncodeError> {
    if (1..=12).contains(&month) && (1..=31).contains(&day) && year >= 2000 {
        Ok(())
    } else {
        Err(FactoryEncodeError::InvalidDate)
    }
}

fn validate_ascii(bytes: &[u8]) -> Result<(), FactoryDecodeError> {
    if bytes.iter().all(|b| b.is_ascii()) {
        Ok(())
    } else {
        Err(FactoryDecodeError::NonAscii)
    }
}

fn copy_ascii(src: &str, dst: &mut [u8]) {
    dst.fill(0);
    dst[..src.len()].copy_from_slice(src.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let sig = [0xAB; 64];
        let bytes = FactoryRecord::encode("A.1", "550e8400", 2026, 6, 27, &sig).unwrap();
        let record = FactoryRecord::decode(&bytes).unwrap();
        assert_eq!(record.hw_revision().unwrap(), "A.1");
        assert_eq!(record.serial().unwrap(), "550e8400");
        assert_eq!(record.manufacturing_date().unwrap(), (2026, 6, 27));
        assert_eq!(record.signature, sig);
    }

    #[test]
    fn blank_record_is_invalid() {
        assert!(!FactoryRecord::blank().is_valid());
    }

    #[test]
    fn factory_xip_addr_matches_linker() {
        assert_eq!(FACTORY_XIP_ADDR, 0x101FF000);
    }
}
