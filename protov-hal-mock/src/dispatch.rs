pub use crate::scpi::{POOL_FULL_RESPONSE, dispatch_command};

pub fn handle_command(device: &mut crate::device::MockDevice, raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    protov_scpi::parse_command(trimmed).and_then(|cmd| dispatch_command(device, cmd))
}
