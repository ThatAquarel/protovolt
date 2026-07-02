pub use protov_core::config::*;

use embassy_time::Duration;

/// When false, all watchdog helpers in [`crate::hal::watchdog`] are no-ops.
pub const WATCHDOG_ENABLED: bool = true;

pub const WATCHDOG_TIMEOUT: Duration = Duration::from_secs(8);
