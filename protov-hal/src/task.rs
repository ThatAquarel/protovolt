use defmt::*;

use embassy_rp::pio::Instance;
use embassy_sync::blocking_mutex::raw::{RawMutex, ThreadModeRawMutex};
use embassy_sync::channel::Sender;
use embassy_time::{Duration, Timer};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::DrawTarget;
use embedded_hal::i2c::I2c;

use crate::app::App;
use crate::hal::Hal;
use crate::hal::event::{
    AppEvent, AppTask, DisplayTask, HardwareEvent, HardwareTask, InterfaceEvent, Task,
};
use crate::hal::firmware;
use crate::hal::firmware::BoardFirmwareCtx;
use crate::scpi::state::ScpiState;
use crate::ui_platform::{HalPlatform, channel_leds_need_refresh};
use protov_ui::{UiRenderer, dispatch_display_task};

#[cfg(feature = "demo")]
const SCREEN_HOLD_TIME: u64 = 2000;
#[cfg(not(feature = "demo"))]
const SCREEN_HOLD_TIME: u64 = 10;

pub enum DfuOutcome {
    Progress(Option<AppTask>),
    VerifySucceeded(Option<AppTask>),
    VerifyFailed(Option<AppTask>),
}

pub fn is_dfu_hardware_task(task: &HardwareTask) -> bool {
    matches!(
        task,
        HardwareTask::DfuPrepare
            | HardwareTask::DfuWriteBlock { .. }
            | HardwareTask::DfuVerifyApply { .. }
    )
}

pub async fn run_followup_tasks<D, PIO>(
    followup: AppTask,
    ui: &mut UiRenderer<'_, D>,
    platform: &mut HalPlatform<'_, '_, PIO>,
    scpi: &ScpiState,
    hw_sender: &Sender<'_, ThreadModeRawMutex, HardwareEvent, 32>,
    int_sender: &Sender<'_, ThreadModeRawMutex, InterfaceEvent, 32>,
) where
    D: DrawTarget<Color = Rgb565>,
    PIO: Instance,
{
    for task in followup {
        match task {
            Task::Hardware(_) => {}
            Task::Display(disp) => {
                handle_display_task(disp, ui, platform, scpi, hw_sender, int_sender).await;
            }
        }
    }
}

pub fn handle_dfu_task(
    task: HardwareTask,
    fw: &mut BoardFirmwareCtx,
    app: &mut App,
    scpi: &mut ScpiState,
    payload: &[u8],
) -> DfuOutcome {
    let Some(event) = firmware::dfu_hardware_event(task, fw, payload) else {
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

pub async fn handle_hardware_task<M, PowerBus, ConverterBus>(
    hardware_task: HardwareTask,
    hal: &mut Hal<'_, M, PowerBus, ConverterBus>,
    hw_sender: &Sender<'_, ThreadModeRawMutex, HardwareEvent, 32>,
    _int_sender: &Sender<'_, ThreadModeRawMutex, InterfaceEvent, 32>,
) where
    M: RawMutex,
    PowerBus: I2c,
    ConverterBus: I2c,
{
    match hardware_task {
        HardwareTask::EnablePowerDelivery => {
            let power_type = hal.init_power_delivery().await;
            hw_sender
                .send(HardwareEvent::PowerDeliveryReady(power_type))
                .await;
        }
        HardwareTask::EnableSense => {
            hal.enable_sense().await;
            info!("enable sense");
        }
        HardwareTask::EnableConverter => {
            let res = hal.enable_converter().await;
            hw_sender.send(HardwareEvent::ConverterReady(res)).await;
        }
        HardwareTask::EnableReadoutLoop => {
            hal.enable_readout_loop().await;
            info!("enable readout loop");
        }
        HardwareTask::PollConverterStatus(channel) => match hal.poll_converter_status(channel) {
            Ok(flags) => {
                hw_sender
                    .send(HardwareEvent::ConverterStatusAcquired(channel, flags))
                    .await;
            }
            Err(()) => {
                warn!("poll converter status failed");
            }
        },
        HardwareTask::DelayedHardwareEvent(ms, event) => {
            Timer::after(Duration::from_millis(ms)).await;
            hw_sender.send(event).await;
        }
        HardwareTask::UpdateConverterVoltage(channel, value) => {
            hal.update_converter_voltage(channel, value).await.unwrap();
        }
        HardwareTask::UpdateConverterCurrent(channel, value) => {
            hal.update_converter_current(channel, value).await.unwrap();
        }
        HardwareTask::UpdateConverterState(channel, state) => {
            hal.update_converter_state(channel, state).await.unwrap();
        }
        _ => {}
    }
}

pub async fn handle_display_task<D, PIO>(
    display_task: DisplayTask,
    ui: &mut UiRenderer<'_, D>,
    platform: &mut HalPlatform<'_, '_, PIO>,
    scpi: &ScpiState,
    _hw_sender: &Sender<'_, ThreadModeRawMutex, HardwareEvent, 32>,
    _int_sender: &Sender<'_, ThreadModeRawMutex, InterfaceEvent, 32>,
) where
    D: DrawTarget<Color = Rgb565>,
    PIO: Instance,
{
    if ui.dfu_screen_active() && !matches!(display_task, DisplayTask::DfuStatus(_)) {
        return;
    }

    match display_task {
        DisplayTask::SetupSplash => {
            ui.clear(platform).ok();
            #[cfg(feature = "demo")]
            {
                ui.boot_demo_mode().ok();
                Timer::after_millis(SCREEN_HOLD_TIME).await;
            }
            ui.boot_splash_screen().ok();
            Timer::after_millis(SCREEN_HOLD_TIME).await;
        }
        DisplayTask::ConfirmPowerDelivery(_)
        | DisplayTask::ConfirmSense(_)
        | DisplayTask::ConfirmConverter(_) => {
            dispatch_display_task(display_task, ui, scpi, platform).ok();
            Timer::after_millis(SCREEN_HOLD_TIME).await;
        }
        _ => {
            dispatch_display_task(display_task, ui, scpi, platform).ok();
            if channel_leds_need_refresh(display_task) {
                platform.refresh_leds().await;
            }
        }
    }
}
