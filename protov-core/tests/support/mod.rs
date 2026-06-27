use protov_core::app::AppCore;
use protov_core::scpi::parser::parse_command;
use protov_core::scpi::state::ScpiState;
use protov_core::scpi::{RESPONSE_BUF, ScpiContext};

pub struct TestBench {
    pub app: AppCore,
    pub scpi: ScpiState,
    pub ctx: ScpiContext,
}

impl TestBench {
    pub fn standby() -> Self {
        let mut app = AppCore::default();
        app.force_standby();
        Self {
            app,
            scpi: ScpiState::default(),
            ctx: ScpiContext::default(),
        }
    }

    pub fn exec(&mut self, cmd: &str) -> Option<heapless::String<RESPONSE_BUF>> {
        let parsed = parse_command(cmd)?;
        let result = self.app.handle_scpi(parsed, &mut self.scpi, &self.ctx);
        result.response.text
    }

    pub fn last_error(&mut self) -> (i32, heapless::String<64>) {
        self.scpi.pop_error()
    }
}
