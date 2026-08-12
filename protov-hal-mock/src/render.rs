//! Headless UI rendering for documentation PNG export.

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::Size;
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay};
use protov_core::model::{DfuStatus, DisplayTask, Limits, PowerType};
use protov_ui::{DisplayGeometry, NullPlatform, UiRenderer, dispatch_display_task};

use crate::device::{MockDevice, MockIdentity};
use crate::pool::SLOT_PROFILES;
use crate::pool::identity_from_profile;
use crate::state::{
    StateSnapshot, UiMode, apply_snapshot, nav_button_from_snapshot, serial_connected_from_snapshot,
};

pub fn render_snapshot(snapshot: &StateSnapshot) -> SimulatorDisplay<Rgb565> {
    let mut device = MockDevice::with_identity(MockIdentity::new(
        snapshot.idn.serial.as_str(),
        snapshot.idn.fw_version.as_str(),
        snapshot.idn.hw_version.as_str(),
        identity_from_profile(SLOT_PROFILES[0]).manufacturing_date,
        identity_from_profile(SLOT_PROFILES[0]).flash_unique_id,
        identity_from_profile(SLOT_PROFILES[0]).serial_signature,
    ));
    apply_snapshot(&mut device, snapshot);

    let mut display = SimulatorDisplay::new(Size::new(320, 240));
    let mut platform = NullPlatform::with_serial_connected(serial_connected_from_snapshot(
        snapshot.power.as_ref(),
    ));
    let mut ui = UiRenderer::new(&mut display, DisplayGeometry::default());

    for task in display_tasks_for_snapshot(snapshot, &mut device) {
        dispatch_display_task(task, &mut ui, &device.scpi, &mut platform).ok();
    }

    display
}

pub fn save_snapshot_png(
    snapshot: &StateSnapshot,
    path: &std::path::Path,
) -> Result<(), RenderError> {
    let display = render_snapshot(snapshot);
    let settings = OutputSettingsBuilder::new().scale(2).build();
    display
        .to_rgb_output_image(&settings)
        .save_png(path)
        .map_err(|source| RenderError::Image { source })
}

fn display_tasks_for_snapshot(
    snapshot: &StateSnapshot,
    device: &mut MockDevice,
) -> heapless::Vec<DisplayTask, 16> {
    let ui = snapshot.ui.unwrap_or(UiMode::Main);
    let nav_button = nav_button_from_snapshot(snapshot.interface.as_ref());

    match ui {
        UiMode::BootSplash => tasks_from([DisplayTask::SetupSplash]),
        UiMode::BootInputPd => boot_tasks([
            DisplayTask::SetupSplash,
            DisplayTask::ConfirmPowerDelivery(PowerType::PowerDelivery(limits_from_power(
                snapshot.power.as_ref(),
            ))),
        ]),
        UiMode::BootInputStd => boot_tasks([
            DisplayTask::SetupSplash,
            DisplayTask::ConfirmPowerDelivery(PowerType::Standard(limits_from_power(
                snapshot.power.as_ref(),
            ))),
        ]),
        UiMode::BootSensePass => boot_tasks([
            DisplayTask::SetupSplash,
            DisplayTask::ConfirmPowerDelivery(default_boot_power(snapshot)),
            DisplayTask::ConfirmSense(Ok(())),
        ]),
        UiMode::BootSenseFail => boot_tasks([
            DisplayTask::SetupSplash,
            DisplayTask::ConfirmPowerDelivery(default_boot_power(snapshot)),
            DisplayTask::ConfirmSense(Err(())),
        ]),
        UiMode::BootConverterPass => boot_tasks([
            DisplayTask::SetupSplash,
            DisplayTask::ConfirmPowerDelivery(default_boot_power(snapshot)),
            DisplayTask::ConfirmSense(Ok(())),
            DisplayTask::ConfirmConverter(Ok(())),
        ]),
        UiMode::BootConverterFail => boot_tasks([
            DisplayTask::SetupSplash,
            DisplayTask::ConfirmPowerDelivery(default_boot_power(snapshot)),
            DisplayTask::ConfirmSense(Ok(())),
            DisplayTask::ConfirmConverter(Err(())),
        ]),
        UiMode::Settings => {
            append_main_tasks(device, nav_button, &[DisplayTask::UpdateSettings(true)])
        }
        UiMode::DfuPreparing => dfu_tasks(DfuStatus::Preparing { total: 65_536 }),
        UiMode::DfuTransferring => dfu_tasks(DfuStatus::Receiving {
            received: 32_768,
            total: 65_536,
        }),
        UiMode::DfuVerified => dfu_tasks(DfuStatus::Verified),
        UiMode::DfuFlashing => dfu_tasks(DfuStatus::Flashing),
        UiMode::DfuFailed => dfu_tasks(DfuStatus::Error),
        UiMode::Main => device.app.main_screen_display_tasks(nav_button),
    }
}

fn append_main_tasks(
    device: &mut MockDevice,
    nav_button: Option<protov_core::model::FunctionButton>,
    extra: &[DisplayTask],
) -> heapless::Vec<DisplayTask, 16> {
    let mut tasks = device.app.main_screen_display_tasks(nav_button);
    for task in extra {
        let _ = tasks.push(*task);
    }
    tasks
}

fn default_boot_power(snapshot: &StateSnapshot) -> PowerType {
    match snapshot.power.as_ref().and_then(|p| p.kind.as_deref()) {
        Some("power_delivery") | Some("PowerDelivery") | Some("PD") => {
            PowerType::PowerDelivery(limits_from_power(snapshot.power.as_ref()))
        }
        _ => PowerType::Standard(limits_from_power(snapshot.power.as_ref())),
    }
}

fn limits_from_power(power: Option<&crate::state::PowerSnapshot>) -> Limits {
    Limits {
        voltage: power.and_then(|p| p.voltage).unwrap_or(5.0),
        current: power.and_then(|p| p.current).unwrap_or(0.5),
    }
}

fn tasks_from<const N: usize>(items: [DisplayTask; N]) -> heapless::Vec<DisplayTask, 16> {
    let mut tasks = heapless::Vec::new();
    for item in items {
        let _ = tasks.push(item);
    }
    tasks
}

fn boot_tasks<const N: usize>(items: [DisplayTask; N]) -> heapless::Vec<DisplayTask, 16> {
    tasks_from(items)
}

/// DFU status updates draw text over the splash screen established by `Preparing`.
fn dfu_tasks(status: DfuStatus) -> heapless::Vec<DisplayTask, 16> {
    let mut tasks = heapless::Vec::new();
    let _ = tasks.push(DisplayTask::DfuStatus(DfuStatus::Preparing {
        total: 65_536,
    }));
    let _ = tasks.push(DisplayTask::DfuStatus(status));
    tasks
}

#[derive(Debug)]
pub enum RenderError {
    Image { source: image::ImageError },
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Image { source } => write!(f, "failed to save png: {source}"),
        }
    }
}

impl std::error::Error for RenderError {}
