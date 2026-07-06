//! SCPI bus response contract (v1.3.3): queries emit a line, sets stay silent.

use crate::config::{
    CH1_FACTORY, CH2_FACTORY, DEFAULT_LCD_BRIGHTNESS, DEFAULT_LED_BRIGHTNESS, format_idn,
};
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
}

/// Line emitted by protov-hal USB when `ScpiResponse::text` is `Some`.
fn usb_line(text: Option<heapless::String<RESPONSE_BUF>>) -> Option<String> {
    text.map(|s| s.to_string())
}

fn assert_silent(bench: &mut TestBench, cmd: &str) {
    assert!(
        usb_line(bench.exec(cmd)).is_none(),
        "expected no USB line for set command {cmd:?}"
    );
}

fn assert_line(bench: &mut TestBench, cmd: &str, expected: &str) {
    assert_eq!(
        usb_line(bench.exec(cmd)).as_deref(),
        Some(expected),
        "unexpected USB line for query {cmd:?}"
    );
}

fn assert_line_nonempty(bench: &mut TestBench, cmd: &str) {
    let line = usb_line(bench.exec(cmd));
    assert!(
        line.as_ref().is_some_and(|s| !s.is_empty()),
        "query {cmd:?}"
    );
}

#[test]
fn query_commands_emit_bus_line() {
    let mut bench = TestBench::standby();
    for cmd in [
        "*IDN?",
        "SYST:VERS?",
        "SYST:ERR?",
        "TELEM?",
        "INP?",
        "DIAG?",
        "TEMP? CHA",
        "TEMP? CHB",
        "TEMP? MCU",
        "LCD:BRIG?",
        "LED:BRIG?",
        "CH1:VOLT?",
        "CH1:CURR?",
        "CH1:OVP?",
        "CH1:OCP?",
        "CH1:MODE?",
        "CH1:COLR?",
        "CH2:VOLT?",
        "CH2:MODE?",
        "OUTP? CH1",
        "OUTP? CH2",
        "MEAS:VOLT? CH1",
        "MEAS:CURR? CH2",
        "MEAS:POW? CH1",
        "SYST:FWUP:STAT?",
    ] {
        assert_line_nonempty(&mut bench, cmd);
    }
}

#[test]
fn mutation_commands_are_silent_on_bus() {
    let mut bench = TestBench::standby();
    for cmd in [
        "*RST",
        "*SAV 1",
        "*RCL 1",
        "*DEL 1",
        "SYST:LOC",
        "SYST:REM",
        "CH1:VOLT 12.0",
        "CH1:CURR 2.5",
        "CH1:OVP 18.0",
        "CH1:OCP 3.0",
        "CH1:COLR 1,2,3",
        "CH2:VOLT 3.3",
        "OUTP CH1,ON",
        "OUTP CH1,OFF",
        "OUTP CH2,ON",
        "OUTP:RESET:PROT CH1",
        "OUTP:RESET:PROT",
        "LCD:BRIG 128",
        "LED:BRIG 64",
    ] {
        assert_silent(&mut bench, cmd);
    }
}

#[test]
fn channel_queries_return_factory_defaults() {
    let mut bench = TestBench::standby();
    assert_line(
        &mut bench,
        "CH1:VOLT?",
        &format!("{:.3}", CH1_FACTORY.voltage_set),
    );
    assert_line(
        &mut bench,
        "CH1:CURR?",
        &format!("{:.3}", CH1_FACTORY.current_set),
    );
    assert_line(&mut bench, "CH1:OVP?", &format!("{:.3}", CH1_FACTORY.ovp));
    assert_line(&mut bench, "CH1:OCP?", &format!("{:.3}", CH1_FACTORY.ocp));
    assert_line(&mut bench, "CH1:MODE?", "OFF");
    assert_line(
        &mut bench,
        "CH2:VOLT?",
        &format!("{:.3}", CH2_FACTORY.voltage_set),
    );
    assert_line(
        &mut bench,
        "CH2:CURR?",
        &format!("{:.3}", CH2_FACTORY.current_set),
    );
    assert_line(&mut bench, "CH2:MODE?", "OFF");
}

#[test]
fn brightness_query_after_set() {
    let mut bench = TestBench::standby();
    assert_line(&mut bench, "LCD:BRIG?", &DEFAULT_LCD_BRIGHTNESS.to_string());
    assert_line(&mut bench, "LED:BRIG?", &DEFAULT_LED_BRIGHTNESS.to_string());
    assert_silent(&mut bench, "LCD:BRIG 128");
    assert_line(&mut bench, "LCD:BRIG?", "128");
    assert_silent(&mut bench, "LED:BRIG 64");
    assert_line(&mut bench, "LED:BRIG?", "64");
}

#[test]
fn interleaved_sets_and_queries_stay_aligned() {
    let mut bench = TestBench::standby();
    assert_silent(&mut bench, "OUTP CH1,OFF");
    assert_line(&mut bench, "CH1:MODE?", "OFF");
    assert_silent(&mut bench, "CH1:VOLT 7.5");
    assert_line(&mut bench, "CH1:VOLT?", "7.500");
    assert_line(
        &mut bench,
        "CH2:VOLT?",
        &format!("{:.3}", CH2_FACTORY.voltage_set),
    );
    assert_silent(&mut bench, "CH2:CURR 0.5");
    assert_line(&mut bench, "CH1:MODE?", "OFF");
    assert_line(&mut bench, "CH2:CURR?", "0.500");
}

#[test]
fn error_commands_are_silent_use_syst_err() {
    let mut bench = TestBench::standby();
    assert_silent(&mut bench, "GARBAGE");
    assert_line(
        &mut bench,
        "SYST:ERR?",
        r#"-113,"Unknown command: GARBAGE""#,
    );
    assert_line(&mut bench, "SYST:ERR?", r#"0,"No error""#);

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
fn fwup_ack_commands_emit_bus_line() {
    let mut bench = TestBench::standby();
    assert_line(&mut bench, "SYST:FWUP:STAT?", "IDLE");
    assert_line(&mut bench, "SYST:FWUP:ABOR", "OK");
}

#[test]
fn idn_query_format() {
    let mut bench = TestBench::standby();
    let mut expected = heapless::String::<128>::new();
    format_idn(&mut expected);
    assert_line(&mut bench, "*IDN?", expected.as_str());
}

#[test]
fn set_does_not_shadow_following_query() {
    let mut bench = TestBench::standby();
    assert_silent(&mut bench, "CH1:VOLT 9.0");
    assert_silent(&mut bench, "CH1:CURR 1.5");
    assert_silent(&mut bench, "OUTP CH1,ON");
    assert_line(&mut bench, "CH1:VOLT?", "9.000");
    assert_line(&mut bench, "CH1:CURR?", "1.500");
    assert_line(&mut bench, "OUTP? CH1", "ON");
}
