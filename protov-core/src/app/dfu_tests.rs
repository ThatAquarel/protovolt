use crate::dfu::FWUP_MAX_BLOCK_LEN;
use crate::model::{
    AppEvent, AppTask, DfuEvent, DisplayTask, HardwareEvent, HardwareTask, InterfaceEvent, Task,
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

    fn exec_parsed(&mut self, cmd: crate::scpi::ScpiCommand) -> crate::scpi::ScpiHandleResult {
        self.app.handle_scpi(cmd, &mut self.scpi, &self.ctx)
    }

    fn fwup_data(&mut self, len: u32) -> Option<heapless::String<RESPONSE_BUF>> {
        self.app.handle_fwup_data(len, &mut self.scpi).response.text
    }
}

fn iter_tasks(task: &AppTask) -> impl Iterator<Item = &Task> {
    task.tasks.iter().filter_map(|t| t.as_ref())
}

fn has_hw(task: &AppTask, expected: HardwareTask) -> bool {
    iter_tasks(task).any(|t| match (t, &expected) {
        (Task::Hardware(HardwareTask::DfuPrepare), HardwareTask::DfuPrepare) => true,
        (
            Task::Hardware(HardwareTask::UpdateConverterState(ch, on)),
            HardwareTask::UpdateConverterState(expected_ch, expected_on),
        ) => ch == expected_ch && on == expected_on,
        (
            Task::Hardware(HardwareTask::DfuWriteBlock { offset: a, len: b }),
            HardwareTask::DfuWriteBlock { offset: c, len: d },
        ) => a == c && b == d,
        (
            Task::Hardware(HardwareTask::DfuVerifyApply {
                len: a,
                signature: sa,
            }),
            HardwareTask::DfuVerifyApply {
                len: b,
                signature: sb,
            },
        ) => a == b && sa == sb,
        _ => false,
    })
}

fn assert_both_channels_disabled(app: &AppCore) {
    use crate::model::{Channel, ChannelHardwareState};
    for ch in [Channel::A, Channel::B] {
        assert!(!app.channel_enable(ch));
        assert_eq!(app.hw_state(ch), ChannelHardwareState::Off);
    }
}

#[test]
fn fwup_star_prepare_and_stat() {
    let mut bench = TestBench::standby();
    assert_eq!(bench.exec("SYST:FWUP:STAT?").unwrap().as_str(), "IDLE");

    let result = bench.exec_parsed(parse_command("SYST:FWUP:STAR 8192").unwrap());
    assert_eq!(result.response.text.unwrap().as_str(), "OK");
    assert!(bench.app.is_update_mode());
    assert_both_channels_disabled(&bench.app);
    let tasks = result.tasks.expect("prepare tasks");
    assert!(has_hw(
        &tasks,
        HardwareTask::UpdateConverterState(crate::model::Channel::A, false)
    ));
    assert!(has_hw(
        &tasks,
        HardwareTask::UpdateConverterState(crate::model::Channel::B, false)
    ));
    assert!(has_hw(&tasks, HardwareTask::DfuPrepare));

    bench
        .app
        .handle_event(AppEvent::Dfu(DfuEvent::PrepareComplete), &mut bench.scpi);
    assert_eq!(
        bench.exec("SYST:FWUP:STAT?").unwrap().as_str(),
        "RECV,0/8192"
    );
}

#[test]
fn fwup_star_disables_active_channels() {
    let mut bench = TestBench::standby();
    bench.exec("OUTP CH1,ON");
    bench.exec("OUTP CH2,ON");
    assert!(bench.app.channel_enable(crate::model::Channel::A));
    assert!(bench.app.channel_enable(crate::model::Channel::B));

    let result = bench.exec_parsed(parse_command("SYST:FWUP:STAR 4096").unwrap());
    assert_eq!(result.response.text.unwrap().as_str(), "OK");
    assert_both_channels_disabled(&bench.app);

    let tasks = result.tasks.expect("prepare tasks");
    assert!(has_hw(
        &tasks,
        HardwareTask::UpdateConverterState(crate::model::Channel::A, false)
    ));
    assert!(has_hw(
        &tasks,
        HardwareTask::UpdateConverterState(crate::model::Channel::B, false)
    ));
}

#[test]
fn fwup_data_block_and_progress() {
    let mut bench = TestBench::standby();
    bench.exec("SYST:FWUP:STAR 8192");
    bench
        .app
        .handle_event(AppEvent::Dfu(DfuEvent::PrepareComplete), &mut bench.scpi);

    let result = bench
        .app
        .handle_fwup_data(FWUP_MAX_BLOCK_LEN as u32, &mut bench.scpi);
    assert_eq!(result.response.text.unwrap().as_str(), "OK");
    let tasks = result.tasks.expect("write task");
    assert!(has_hw(
        &tasks,
        HardwareTask::DfuWriteBlock {
            offset: 0,
            len: FWUP_MAX_BLOCK_LEN as u32,
        }
    ));

    bench.app.handle_event(
        AppEvent::Dfu(DfuEvent::BlockWriteComplete {
            offset: 0,
            len: FWUP_MAX_BLOCK_LEN as u32,
        }),
        &mut bench.scpi,
    );
    assert_eq!(
        bench.exec("SYST:FWUP:STAT?").unwrap().as_str(),
        "RECV,4096/8192"
    );
}

#[test]
fn fwup_abor_restores_standby() {
    let mut bench = TestBench::standby();
    bench.exec("SYST:FWUP:STAR 4096");
    assert!(bench.app.is_update_mode());
    assert_eq!(bench.exec("SYST:FWUP:ABOR").unwrap().as_str(), "OK");
    assert!(!bench.app.is_update_mode());
    assert!(bench.app.is_standby());
    assert_eq!(bench.exec("SYST:FWUP:STAT?").unwrap().as_str(), "IDLE");
}

#[test]
fn update_mode_rejects_channel_mutations() {
    let mut bench = TestBench::standby();
    bench.exec("SYST:FWUP:STAR 4096");
    bench
        .app
        .handle_event(AppEvent::Dfu(DfuEvent::PrepareComplete), &mut bench.scpi);

    let result = bench.exec_parsed(parse_command("OUTP CH1,ON").unwrap());
    assert_eq!(result.response.text.unwrap().as_str(), "ERR");
}

#[test]
fn update_mode_silences_hardware_events() {
    let mut bench = TestBench::standby();
    bench.exec("SYST:FWUP:STAR 4096");
    bench.app.set_channel_readout(
        crate::model::Channel::A,
        crate::model::Readout {
            voltage: 1.0,
            current: 0.1,
            power: 0.1,
        },
    );
    let task = bench.app.handle_event(
        AppEvent::Hardware(HardwareEvent::ReadoutAcquired(
            crate::model::Channel::A,
            crate::model::Readout {
                voltage: 2.0,
                current: 0.2,
                power: 0.4,
            },
        )),
        &mut bench.scpi,
    );
    assert!(task.is_none());
}

#[test]
fn update_mode_silences_interface_events() {
    let mut bench = TestBench::standby();
    bench.exec("SYST:FWUP:STAR 4096");
    let task = bench.app.handle_event(
        AppEvent::Interface(InterfaceEvent::ButtonUp),
        &mut bench.scpi,
    );
    assert!(task.is_none());
}

#[test]
fn fwup_data_before_star_rejected() {
    let mut bench = TestBench::standby();
    let resp = bench.fwup_data(4096).unwrap();
    assert_eq!(resp.as_str(), "ERR");
}

#[test]
fn fwup_star_requires_standby() {
    let mut app = AppCore::default();
    let mut scpi = ScpiState::default();
    let ctx = ScpiContext::default();
    let cmd = parse_command("SYST:FWUP:STAR 4096").unwrap();
    let result = app.handle_scpi(cmd, &mut scpi, &ctx);
    assert!(result.response.text.is_none());
}

#[test]
fn dfu_status_display_task_on_star() {
    let mut bench = TestBench::standby();
    let result = bench.exec_parsed(parse_command("SYST:FWUP:STAR 4096").unwrap());
    let tasks = result.tasks.unwrap();
    assert!(iter_tasks(&tasks).any(|t| {
        matches!(
            t,
            Task::Display(DisplayTask::DfuStatus(crate::model::DfuStatus::Preparing {
                total: 4096
            }))
        )
    }));
}

#[test]
fn fwup_appl_when_ready_defers_response() {
    let mut bench = TestBench::standby();
    bench.exec("SYST:FWUP:STAR 4096");
    bench
        .app
        .handle_event(AppEvent::Dfu(DfuEvent::PrepareComplete), &mut bench.scpi);
    bench.fwup_data(4096);
    bench.app.handle_event(
        AppEvent::Dfu(DfuEvent::BlockWriteComplete {
            offset: 0,
            len: 4096,
        }),
        &mut bench.scpi,
    );
    assert_eq!(
        bench.exec("SYST:FWUP:STAT?").unwrap().as_str(),
        "READY,4096"
    );

    let sig_hex = "#H".to_string() + &"cd".repeat(64);
    let result = bench.exec_parsed(parse_command(&format!("SYST:FWUP:APPL {sig_hex}")).unwrap());
    assert!(result.response.text.is_none());
    let tasks = result.tasks.expect("verify task");
    assert!(has_hw(
        &tasks,
        HardwareTask::DfuVerifyApply {
            len: 4096,
            signature: [0xcd; 64],
        }
    ));
}

#[test]
fn fwup_appl_response_ok_after_verify() {
    let mut bench = TestBench::standby();
    bench.exec("SYST:FWUP:STAR 4096");
    bench
        .app
        .handle_event(AppEvent::Dfu(DfuEvent::PrepareComplete), &mut bench.scpi);
    bench.fwup_data(4096);
    bench.app.handle_event(
        AppEvent::Dfu(DfuEvent::BlockWriteComplete {
            offset: 0,
            len: 4096,
        }),
        &mut bench.scpi,
    );
    let sig_hex = "#H".to_string() + &"cd".repeat(64);
    bench.exec(&format!("SYST:FWUP:APPL {sig_hex}"));

    let resp = bench.app.fwup_appl_response(true, &mut bench.scpi);
    assert_eq!(resp.text.unwrap().as_str(), "OK");
}

#[test]
fn fwup_appl_response_err_on_verify_failure() {
    let mut bench = TestBench::standby();
    bench.exec("SYST:FWUP:STAR 4096");
    bench
        .app
        .handle_event(AppEvent::Dfu(DfuEvent::PrepareComplete), &mut bench.scpi);
    bench.fwup_data(4096);
    bench.app.handle_event(
        AppEvent::Dfu(DfuEvent::BlockWriteComplete {
            offset: 0,
            len: 4096,
        }),
        &mut bench.scpi,
    );
    let sig_hex = "#H".to_string() + &"cd".repeat(64);
    bench.exec(&format!("SYST:FWUP:APPL {sig_hex}"));

    let resp = bench.app.fwup_appl_response(false, &mut bench.scpi);
    assert_eq!(resp.text.unwrap().as_str(), "ERR");
    assert_eq!(
        bench.exec("SYST:ERR?").unwrap().as_str(),
        "-200,\"Firmware signature verification failed\""
    );
}
