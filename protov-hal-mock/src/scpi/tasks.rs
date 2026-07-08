//! Execute AppCore hardware tasks in-process (mock DFU backend).

use protov_core::app::AppCore;
use protov_core::model::{AppEvent, AppTask, HardwareTask, Task};
use protov_core::scpi::ScpiResponse;
use protov_core::scpi::state::ScpiState;

use super::dfu::MockFirmwareStore;

pub enum DfuOutcome {
    Progress(Option<AppTask>),
    VerifySucceeded(Option<AppTask>),
    VerifyFailed(Option<AppTask>),
}

fn is_dfu_hardware_task(task: &HardwareTask) -> bool {
    matches!(
        task,
        HardwareTask::DfuPrepare
            | HardwareTask::DfuWriteBlock { .. }
            | HardwareTask::DfuVerifyApply { .. }
    )
}

fn handle_mock_dfu(
    task: HardwareTask,
    app: &mut AppCore,
    scpi: &mut ScpiState,
    firmware: &mut MockFirmwareStore,
) -> DfuOutcome {
    let Some(event) = MockFirmwareStore::dfu_hardware_event(task, firmware) else {
        return DfuOutcome::Progress(None);
    };

    match event {
        protov_core::model::DfuEvent::VerifyApplyComplete => {
            DfuOutcome::VerifySucceeded(app.handle_event(AppEvent::Dfu(event), scpi))
        }
        protov_core::model::DfuEvent::VerifyApplyFailed => {
            DfuOutcome::VerifyFailed(app.handle_event(AppEvent::Dfu(event), scpi))
        }
        other => DfuOutcome::Progress(app.handle_event(AppEvent::Dfu(other), scpi)),
    }
}

pub fn run_tasks(
    app: &mut AppCore,
    scpi: &mut ScpiState,
    ctx: &protov_core::scpi::ScpiContext,
    firmware: &mut MockFirmwareStore,
    tasks: AppTask,
    mut response: ScpiResponse,
) -> (ScpiResponse, bool) {
    let mut fwup_appl_outcome = None;

    for task in tasks {
        match task {
            Task::Display(_) => {}
            Task::Hardware(hw_task) if is_dfu_hardware_task(&hw_task) => {
                match handle_mock_dfu(hw_task, app, scpi, firmware) {
                    DfuOutcome::VerifySucceeded(extra) => {
                        fwup_appl_outcome = Some(true);
                        if let Some(extra) = extra {
                            let (next_response, _) =
                                run_tasks(app, scpi, ctx, firmware, extra, response);
                            response = next_response;
                        }
                    }
                    DfuOutcome::VerifyFailed(extra) => {
                        fwup_appl_outcome = Some(false);
                        firmware.abort();
                        if let Some(extra) = extra {
                            let (next_response, _) =
                                run_tasks(app, scpi, ctx, firmware, extra, response);
                            response = next_response;
                        }
                    }
                    DfuOutcome::Progress(extra) => {
                        if let Some(extra) = extra {
                            let (next_response, _) =
                                run_tasks(app, scpi, ctx, firmware, extra, response);
                            response = next_response;
                        }
                    }
                }
            }
            Task::Hardware(_) => {}
        }
    }

    if let Some(success) = fwup_appl_outcome {
        response = app.fwup_appl_response(success, scpi);
        return (response, success);
    }

    (response, false)
}
