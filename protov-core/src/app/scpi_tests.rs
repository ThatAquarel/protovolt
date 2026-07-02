use crate::config::{SCPI_SYSTEM_VERSION, format_idn};
use crate::scpi::parser::parse_command;
use crate::scpi::state::ScpiState;
use crate::scpi::{RESPONSE_BUF, ScpiContext};

use super::AppCore;

struct TestBench {
    app: AppCore,
    scpi: ScpiState,
    ctx: ScpiContext,
}

impl TestBench {
    fn standby() -> Self {
        let mut app = AppCore::default();
        app.force_standby();
        Self {
            app,
            scpi: ScpiState::default(),
            ctx: ScpiContext::default(),
        }
    }

    fn exec(&mut self, cmd: &str) -> Option<heapless::String<RESPONSE_BUF>> {
        let parsed = parse_command(cmd)?;
        let result = self.app.handle_scpi(parsed, &mut self.scpi, &self.ctx);
        result.response.text
    }

    fn last_error(&mut self) -> (i32, heapless::String<64>) {
        self.scpi.pop_error()
    }
}

#[test]
fn fwup_stat_idle_in_standby() {
    let mut bench = TestBench::standby();
    assert_eq!(bench.exec("SYST:FWUP:STAT?").unwrap().as_str(), "IDLE");
    assert_eq!(bench.exec("SYST:FWUP:ABOR").unwrap().as_str(), "OK");
}

#[test]
fn idn_query() {
    let mut bench = TestBench::standby();
    let resp = bench.exec("*IDN?").unwrap();
    let mut expected = heapless::String::<128>::new();
    format_idn(&mut expected);
    assert_eq!(resp.as_str(), expected.as_str());
}

#[test]
fn syst_vers() {
    let mut bench = TestBench::standby();
    assert_eq!(
        bench.exec("SYST:VERS?").unwrap().as_str(),
        SCPI_SYSTEM_VERSION
    );
}

#[test]
fn rst_resets_channels() {
    let mut bench = TestBench::standby();
    bench.exec("CH1:VOLT 12.0");
    bench.exec("*RST");
    let resp = bench.exec("CH1:VOLT?").unwrap();
    assert!(resp.as_str().starts_with("5"));
}

#[test]
fn outp_on_off() {
    let mut bench = TestBench::standby();
    bench.exec("OUTP CH1,ON");
    assert!(bench.app.channel_enable(crate::model::Channel::A));
    assert_eq!(bench.exec("OUTP? CH1").unwrap().as_str(), "ON");
    bench.exec("OUTP CH1,OFF");
    assert_eq!(bench.exec("OUTP? CH1").unwrap().as_str(), "OFF");
}

#[test]
fn mode_query_off_by_default() {
    let mut bench = TestBench::standby();
    assert_eq!(bench.exec("CH1:MODE?").unwrap().as_str(), "OFF");
}

#[test]
fn meas_zero_when_off() {
    let mut bench = TestBench::standby();
    assert_eq!(bench.exec("MEAS:VOLT? CH1").unwrap().as_str(), "0.000");
}

#[test]
fn mutation_rejected_before_standby() {
    let mut app = AppCore::default();
    let mut scpi = ScpiState::default();
    let ctx = ScpiContext::default();
    let cmd = parse_command("OUTP CH1,ON").unwrap();
    let result = app.handle_scpi(cmd, &mut scpi, &ctx);
    assert!(result.response.text.is_none());
    let (code, _) = scpi.pop_error();
    assert_eq!(code, -221);
}

#[test]
fn reset_prot_clears_latch() {
    let mut bench = TestBench::standby();
    bench
        .scpi
        .set_prot_latched(Some(crate::scpi::ScpiChannel::Ch1), true);
    bench.exec("OUTP:RESET:PROT CH1");
    assert!(!bench.scpi.prot_latched(crate::scpi::ScpiChannel::Ch1));
}

#[test]
fn sav_rcl_slot() {
    let mut bench = TestBench::standby();
    bench.exec("CH1:VOLT 7.5");
    bench.exec("*SAV 2");
    bench.exec("CH1:VOLT 1.0");
    bench.exec("*RCL 2");
    let resp = bench.exec("CH1:VOLT?").unwrap();
    assert!(resp.as_str().starts_with("7.5"));
}

#[test]
fn unknown_command_error() {
    let mut bench = TestBench::standby();
    bench.exec("GARBAGE");
    let (code, msg) = bench.last_error();
    assert_eq!(code, -113);
    assert!(msg.as_str().contains("Unknown"));
}

#[test]
fn appearance_color_query() {
    let mut bench = TestBench::standby();
    bench.exec("CH1:COLR 255,0,128");
    let resp = bench.exec("CH1:COLR?").unwrap();
    assert_eq!(resp.as_str(), "255,0,128");
}

#[test]
fn syst_err_after_push() {
    let mut bench = TestBench::standby();
    bench.scpi.push_error(-100, "test fault");
    let resp = bench.exec("SYST:ERR?").unwrap();
    assert!(resp.as_str().contains("-100"));
}
