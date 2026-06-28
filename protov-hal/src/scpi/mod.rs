pub mod usb;

pub use protov_core::scpi::parser;
pub use protov_core::scpi::state;
pub use protov_core::scpi::*;

use core::sync::atomic::{AtomicBool, Ordering};

use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;

pub static SCPI_CMD: Channel<ThreadModeRawMutex, ScpiCommand, 4> = Channel::new();
pub static SCPI_RESP: Channel<ThreadModeRawMutex, ScpiResponse, 4> = Channel::new();

static SERIAL_CONNECTED: AtomicBool = AtomicBool::new(false);

pub fn serial_connected() -> bool {
    SERIAL_CONNECTED.load(Ordering::Relaxed)
}

pub fn set_serial_connected(connected: bool) {
    SERIAL_CONNECTED.store(connected, Ordering::Relaxed);
}
