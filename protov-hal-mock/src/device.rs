use protov_core::app::AppCore;
use protov_core::scpi::ScpiContext;
use protov_core::scpi::state::ScpiState;

use crate::scpi::{MockFirmwareStore, ScpiReader};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MockIdentity {
    pub serial: String,
    pub fw_version: String,
    pub hw_version: String,
}

impl MockIdentity {
    pub fn new(serial: &str, fw_version: &str, hw_version: &str) -> Self {
        Self {
            serial: serial.to_owned(),
            fw_version: fw_version.to_owned(),
            hw_version: hw_version.to_owned(),
        }
    }
}

pub struct MockDevice {
    pub app: AppCore,
    pub scpi: ScpiState,
    pub ctx: ScpiContext,
    pub identity: MockIdentity,
    pub firmware: MockFirmwareStore,
}

impl MockDevice {
    pub fn with_identity(identity: MockIdentity) -> Self {
        let mut app = AppCore::default();
        app.force_standby();
        Self {
            app,
            scpi: ScpiState::default(),
            ctx: ScpiContext::default(),
            identity,
            firmware: MockFirmwareStore::default(),
        }
    }

    pub fn reset_to_profile(&mut self, identity: MockIdentity) {
        self.identity = identity;
        self.app.reset_to_factory(&mut self.scpi);
        self.ctx = ScpiContext::default();
        self.firmware.abort();
    }

    pub fn handle(&mut self, command: &str) -> Option<String> {
        crate::dispatch::handle_command(self, command)
    }

    pub fn handle_bytes(&mut self, chunk: &[u8]) -> Vec<String> {
        let mut reader = ScpiReader::new();
        reader.push(chunk);
        reader.drain(self)
    }

    pub fn handle_bytes_with_reader(
        &mut self,
        reader: &mut ScpiReader,
        chunk: &[u8],
    ) -> Vec<String> {
        reader.push(chunk);
        reader.drain(self)
    }
}
