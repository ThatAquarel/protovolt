//! RP2040 watchdog — clears protov-bootloader timer and guards flash operations.

use core::sync::atomic::{AtomicBool, Ordering};

use embassy_rp::peripherals::WATCHDOG;
use embassy_rp::watchdog::Watchdog;
use static_cell::StaticCell;

use crate::config::{WATCHDOG_ENABLED, WATCHDOG_TIMEOUT};

static WDOG: StaticCell<Watchdog> = StaticCell::new();
static INITIALIZED: AtomicBool = AtomicBool::new(false);
static mut WDOG_REF: *mut Watchdog = core::ptr::null_mut();

pub fn init(peri: embassy_rp::Peri<'static, WATCHDOG>) {
    let w = WDOG.init(Watchdog::new(peri));
    unsafe {
        WDOG_REF = w;
    }
    INITIALIZED.store(true, Ordering::Release);
}

pub fn stop_bootloader() {
    with_wdog(|w| w.stop());
}

pub fn pause_for_flash() {
    with_wdog(|w| w.stop());
}

pub fn resume_after_flash() {
    if !WATCHDOG_ENABLED {
        return;
    }
    with_wdog(|w| {
        w.start(WATCHDOG_TIMEOUT);
        w.feed(WATCHDOG_TIMEOUT);
    });
}

/// Leave watchdog stopped for normal PSU operation (Embassy b.rs end state).
pub fn cancel_after_boot() {
    with_wdog(|w| w.stop());
}

fn with_wdog(f: impl FnOnce(&mut Watchdog)) {
    if !INITIALIZED.load(Ordering::Acquire) {
        return;
    }
    unsafe {
        if WDOG_REF.is_null() {
            return;
        }
        f(&mut *WDOG_REF);
    }
}
