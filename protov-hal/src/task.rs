use defmt::*;

use embassy_rp::pio::Instance;
use embassy_sync::blocking_mutex::raw::{RawMutex, ThreadModeRawMutex};
use embassy_sync::channel::Sender;
use embassy_time::{Duration, Timer};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::DrawTarget;
use embedded_hal::i2c::I2c;

use crate::hal::Hal;
use crate::hal::event::{
    Channel, ChannelFocus, ChannelHardwareState, ConfirmState, DisplayTask, HardwareEvent,
    HardwareTask, InterfaceEvent, PowerType, SetState,
};
use crate::scpi::state::ScpiState;
use crate::ui::{SCREEN_HOLD_TIME, Ui, labels};

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
    }
}

pub async fn handle_display_task<D, PIO>(
    display_task: DisplayTask,
    ui: &mut Ui<'_, D, PIO>,
    scpi: &ScpiState,
    _hw_sender: &Sender<'_, ThreadModeRawMutex, HardwareEvent, 32>,
    _int_sender: &Sender<'_, ThreadModeRawMutex, InterfaceEvent, 32>,
) where
    D: DrawTarget<Color = Rgb565>,
    PIO: Instance,
{
    let skip_settings_redraw = matches!(
        display_task,
        DisplayTask::UpdateSettings(_)
            | DisplayTask::UpdateChannelUnits(_)
            | DisplayTask::SetupSplash
            | DisplayTask::ConfirmPowerDelivery(_)
            | DisplayTask::ConfirmSense(_)
            | DisplayTask::ConfirmConverter(_)
            | DisplayTask::SetupMain(_, _, _)
    );

    match display_task {
        DisplayTask::SetupSplash => {
            ui.clear().unwrap();

            #[cfg(feature = "demo")]
            {
                ui.boot_demo_mode().unwrap();
                Timer::after_millis(SCREEN_HOLD_TIME).await;
            }

            ui.boot_splash_screen().unwrap();
            Timer::after_millis(SCREEN_HOLD_TIME).await;
        }
        DisplayTask::ConfirmPowerDelivery(power_type) => {
            let (usb_type, valid) = match power_type {
                PowerType::PowerDelivery(_) => (labels::PD, true),
                PowerType::Standard(_) => (labels::STD, false),
            };

            ui.boot_splash_text(0, labels::INPUT, usb_type, valid)
                .unwrap();
            Timer::after_millis(SCREEN_HOLD_TIME).await;
        }
        DisplayTask::ConfirmSense(result) => {
            let (res, valid) = match result {
                Ok(()) => (labels::PASS, true),
                Err(()) => (labels::FAIL, false),
            };

            ui.boot_splash_text(1, labels::SENSE, res, valid).unwrap();
            Timer::after_millis(SCREEN_HOLD_TIME).await;
        }
        DisplayTask::ConfirmConverter(result) => {
            let (res, valid) = match result {
                Ok(()) => (labels::PASS, true),
                Err(()) => (labels::FAIL, false),
            };

            ui.boot_splash_text(2, labels::CONVERTER, res, valid)
                .unwrap();
            Timer::after_millis(SCREEN_HOLD_TIME).await;
        }
        DisplayTask::SetupMain(power_type, ch_a_limits, ch_b_limits) => {
            ui.clear().unwrap();

            ui.nav_power_info(power_type).unwrap();
            ui.nav_buttons(scpi, ConfirmState::AwaitModify, None)
                .await
                .unwrap();

            let channels = [Channel::A, Channel::B];
            for channel in channels.iter() {
                let limits = match channel {
                    Channel::A => ch_a_limits,
                    Channel::B => ch_b_limits,
                };

                let initial_focus = ChannelFocus::UnselectedInactive;
                ui.controls_channel_box(scpi, *channel, initial_focus)
                    .await
                    .unwrap();
                ui.controls_header_chip(scpi, *channel, initial_focus, ChannelHardwareState::Off)
                    .unwrap();
                ui.controls_channel_units(*channel).unwrap();

                ui.controls_submeasurement(
                    scpi,
                    *channel,
                    None,
                    limits,
                    ConfirmState::AwaitModify,
                    None,
                )
                .unwrap();
                ui.controls_submeasurement_tag(
                    scpi,
                    *channel,
                    SetState::Set,
                    None,
                    ConfirmState::AwaitModify,
                )
                .unwrap();
            }
        }
        DisplayTask::UpdatePowerInfo(power_type) => {
            ui.nav_power_info(power_type).unwrap();
        }
        DisplayTask::UpdateReadout(channel, readout) => {
            ui.controls_measurement(channel, readout).unwrap();
        }
        DisplayTask::UpdateSetpoint(channel, limits, set_select, confirm_state, precision) => {
            ui.controls_submeasurement(scpi, channel, set_select, limits, confirm_state, precision)
                .unwrap();
        }
        DisplayTask::UpdateChannelFocus(focus_a, focus_b, hw_state_a, hw_state_b) => {
            let focuses = [focus_a, focus_b];
            for (i, focus) in focuses.iter().enumerate() {
                let (channel, hw_state) = match i {
                    0 => (Channel::A, hw_state_a),
                    _ => (Channel::B, hw_state_b),
                };
                ui.controls_channel_box(scpi, channel, *focus)
                    .await
                    .unwrap();
                ui.controls_header_chip(scpi, channel, *focus, hw_state)
                    .unwrap();
            }
        }
        DisplayTask::UpdateButton(confirm_state, function_button_state) => {
            ui.nav_buttons(scpi, confirm_state, function_button_state)
                .await
                .unwrap();
        }
        DisplayTask::UpdateSetState(channel, set_state, set_select, confirm_state) => {
            ui.controls_submeasurement_tag(scpi, channel, set_state, set_select, confirm_state)
                .unwrap();
        }
        DisplayTask::UpdateChannelHardwareState(channel, focus, hw_state) => {
            ui.controls_header_chip(scpi, channel, focus, hw_state)
                .unwrap();
        }
        DisplayTask::UpdateSettings(visible) => {
            ui.set_settings_visible(visible);
            if visible {
                ui.draw_settings_overlay().unwrap();
            } else {
                ui.clear_settings_section().unwrap();
            }
        }
        DisplayTask::UpdateChannelUnits(channel) => {
            if !ui.settings_visible() {
                ui.controls_channel_units(channel).unwrap();
            }
        }
    }

    if !skip_settings_redraw {
        ui.redraw_settings_overlay_if_visible().ok();
    }
}
