use protov_core::app::AppCore;
use protov_core::scpi::ScpiContext;
use protov_core::scpi::state::ScpiState;
use protov_scpi::ScpiCommand;

use crate::scpi::{MockFirmwareStore, ScpiReader};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FwupAfter {
    #[default]
    None,
    StarDelay,
    ApplReboot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MockIdentity {
    pub serial: String,
    pub fw_version: String,
    pub hw_version: String,
    pub manufacturing_date: (u16, u8, u8),
    pub flash_unique_id: [u8; 8],
    pub serial_signature: [u8; 64],
}

impl MockIdentity {
    pub fn new(
        serial: &str,
        fw_version: &str,
        hw_version: &str,
        manufacturing_date: (u16, u8, u8),
        flash_unique_id: [u8; 8],
        serial_signature: [u8; 64],
    ) -> Self {
        Self {
            serial: serial.to_owned(),
            fw_version: fw_version.to_owned(),
            hw_version: hw_version.to_owned(),
            manufacturing_date,
            flash_unique_id,
            serial_signature,
        }
    }
}

pub struct MockDevice {
    pub app: AppCore,
    pub scpi: ScpiState,
    pub ctx: ScpiContext,
    pub identity: MockIdentity,
    pub firmware: MockFirmwareStore,
    pub fwup_after: FwupAfter,
}

impl MockDevice {
    pub fn with_identity(identity: MockIdentity) -> Self {
        let mut app = AppCore::default();
        app.force_standby();
        let mut ctx = ScpiContext::default();
        ctx.flash_unique_id = identity.flash_unique_id;
        Self {
            app,
            scpi: ScpiState::default(),
            ctx,
            identity,
            firmware: MockFirmwareStore::default(),
            fwup_after: FwupAfter::None,
        }
    }

    pub fn complete_firmware_reboot(&mut self, identity: MockIdentity) {
        self.identity = identity;
        if self.app.is_update_mode() {
            let _ = self
                .app
                .handle_scpi(ScpiCommand::FwupAbor, &mut self.scpi, &self.ctx);
        }
        self.app.reset_to_factory(&mut self.scpi);
        self.app.force_standby();
        self.ctx = ScpiContext::default();
        self.ctx.flash_unique_id = self.identity.flash_unique_id;
        self.firmware.abort();
        self.fwup_after = FwupAfter::None;
    }

    pub fn reset_to_profile(&mut self, identity: MockIdentity) {
        self.identity = identity;
        self.app.reset_to_factory(&mut self.scpi);
        self.ctx = ScpiContext::default();
        self.ctx.flash_unique_id = self.identity.flash_unique_id;
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
