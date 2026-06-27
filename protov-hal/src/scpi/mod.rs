pub mod usb;

pub use protov_core::scpi::colors;
pub use protov_core::scpi::parser;
pub use protov_core::scpi::state;
pub use protov_core::scpi::telemetry;
pub use protov_core::scpi::*;
pub use usb::USB_ENUM_GRACE_MS;

use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;

use protov_core::scpi::ScpiCommand;

pub static SCPI_CMD: Channel<ThreadModeRawMutex, ScpiCommand, 4> = Channel::new();
pub static SCPI_RESP: Channel<ThreadModeRawMutex, ScpiResponse, 4> = Channel::new();
