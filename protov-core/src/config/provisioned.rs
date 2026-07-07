//! Product identity loaded from the factory flash sector (or dev defaults on host).

use core::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

use protov_nvm::{FactoryRecord, factory::FACTORY_SERIAL_MAX};
use static_cell::ConstStaticCell;

use super::hardware::PROFILE as HARDWARE_PROFILE;

const HW_REV_CAP: usize = 8;

pub const DEFAULT_MANUFACTURING_DATE: (u16, u8, u8) = (2026, 6, 27);

#[derive(Clone, Copy)]
struct ProvisionedProduct {
    serial: [u8; FACTORY_SERIAL_MAX],
    serial_len: u8,
    hw_revision: [u8; HW_REV_CAP],
    hw_rev_len: u8,
    year: u16,
    month: u8,
    day: u8,
    signature: [u8; 64],
}

impl ProvisionedProduct {
    fn dev_default() -> Self {
        let mut product = Self {
            serial: *b"00000011\0\0\0\0\0\0\0\0",
            serial_len: 8,
            hw_revision: [0; HW_REV_CAP],
            hw_rev_len: 0,
            year: DEFAULT_MANUFACTURING_DATE.0,
            month: DEFAULT_MANUFACTURING_DATE.1,
            day: DEFAULT_MANUFACTURING_DATE.2,
            signature: [0; 64],
        };
        product.set_hw_revision(HARDWARE_PROFILE.revision);
        product
    }

    fn from_record(record: &FactoryRecord) -> Self {
        let mut product = Self::dev_default();
        if let Ok(serial) = record.serial() {
            product.set_serial(serial);
        }
        if let Ok(hw) = record.hw_revision() {
            product.set_hw_revision(hw);
        }
        if let Ok(date) = record.manufacturing_date() {
            product.year = date.0;
            product.month = date.1;
            product.day = date.2;
        }
        product.signature = record.signature;
        product
    }

    fn set_serial(&mut self, serial: &str) {
        self.serial.fill(0);
        let len = serial.len().min(FACTORY_SERIAL_MAX);
        self.serial[..len].copy_from_slice(&serial.as_bytes()[..len]);
        self.serial_len = len as u8;
    }

    fn set_hw_revision(&mut self, hw: &str) {
        self.hw_revision.fill(0);
        let len = hw.len().min(HW_REV_CAP);
        self.hw_revision[..len].copy_from_slice(&hw.as_bytes()[..len]);
        self.hw_rev_len = len as u8;
    }

    fn serial_str(&self) -> &str {
        core::str::from_utf8(&self.serial[..self.serial_len as usize]).unwrap_or("")
    }

    fn hw_revision_str(&self) -> &str {
        core::str::from_utf8(&self.hw_revision[..self.hw_rev_len as usize]).unwrap_or("")
    }

    fn manufacturing_date(&self) -> (u16, u8, u8) {
        (self.year, self.month, self.day)
    }
}

static PROVISIONED: ConstStaticCell<ProvisionedProduct> =
    ConstStaticCell::new(ProvisionedProduct {
        serial: [0; FACTORY_SERIAL_MAX],
        serial_len: 0,
        hw_revision: [0; HW_REV_CAP],
        hw_rev_len: 0,
        year: 0,
        month: 0,
        day: 0,
        signature: [0; 64],
    });
static PROVISIONED_PTR: AtomicPtr<ProvisionedProduct> = AtomicPtr::new(core::ptr::null_mut());
static PROVISIONED_READY: AtomicBool = AtomicBool::new(false);

/// Load identity from an optional factory record; uses dev defaults when `None` or invalid.
pub fn init(record: Option<&FactoryRecord>) {
    if PROVISIONED_READY.load(Ordering::Acquire) {
        return;
    }
    let slot = ConstStaticCell::take(&PROVISIONED);
    *slot = match record {
        Some(record) if record.is_valid() => ProvisionedProduct::from_record(record),
        _ => ProvisionedProduct::dev_default(),
    };
    PROVISIONED_PTR.store(slot, Ordering::Release);
    PROVISIONED_READY.store(true, Ordering::Release);
}

/// Read the factory sector over XIP and install product identity (embedded boot).
pub fn init_from_factory_flash() {
    init(protov_nvm::read_xip().as_ref());
}

fn ensure_initialized() {
    if !PROVISIONED_READY.load(Ordering::Acquire) {
        init(None);
    }
}

fn product() -> &'static ProvisionedProduct {
    ensure_initialized();
    let ptr = PROVISIONED_PTR.load(Ordering::Acquire);
    debug_assert!(!ptr.is_null());
    unsafe { &*ptr }
}

pub fn serial_number() -> &'static str {
    product().serial_str()
}

pub fn hardware_revision() -> &'static str {
    product().hw_revision_str()
}

pub fn manufacturing_date() -> (u16, u8, u8) {
    product().manufacturing_date()
}

pub fn manufacturing_year() -> u16 {
    manufacturing_date().0
}

pub fn manufacturing_month() -> u8 {
    manufacturing_date().1
}

pub fn manufacturing_day() -> u8 {
    manufacturing_date().2
}

pub fn serial_attestation() -> &'static [u8; 64] {
    &product().signature
}

/// Dev-only default serial (before factory init or on host).
pub const DEFAULT_SERIAL_NUMBER: &str = "00000011";

/// Dev-only default HW revision aligned with the selected [`HARDWARE_PROFILE`].
pub fn default_hardware_revision() -> &'static str {
    HARDWARE_PROFILE.revision
}
