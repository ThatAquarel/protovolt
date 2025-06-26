use defmt::*;

use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Sender;
use embassy_time::{Duration, Ticker, Timer};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::DrawTarget;

use crate::lib::event::{
    Channel, ChannelFocus, ConfirmState, DisplayTask, HardwareEvent, HardwareTask, InterfaceEvent, Limits, PowerType, SetState
};
use crate::ui::{Ui, color_scheme, labels};
use crate::{HARDWARE_CHANNEL, poll_readout};

pub async fn handle_hardware_task(
    hardware_task: HardwareTask,
    spawner: Spawner,
    hw_sender: &Sender<'_, ThreadModeRawMutex, HardwareEvent, 32>,
    int_sender: &Sender<'_, ThreadModeRawMutex, InterfaceEvent, 32>,
) {
    match hardware_task {
        HardwareTask::EnablePowerDelivery => {
            Timer::after_millis(10).await;

            hw_sender
                .send(HardwareEvent::PowerDeliveryReady(PowerType::PowerDelivery(
                    Limits {
                        voltage: 12.345,
                        current: 6.789,
                    },
                )))
                .await;
        }
        HardwareTask::EnableSense => {
            Timer::after_millis(10).await;

            hw_sender.send(HardwareEvent::SenseReady(Ok(()))).await;
        }
        HardwareTask::EnableConverter => {
            Timer::after_millis(10).await;

            hw_sender.send(HardwareEvent::ConverterReady(Ok(()))).await;
        }
        HardwareTask::EnableReadoutLoop => {
            info!("enable readout loop");
            unwrap!(spawner.spawn(poll_readout(HARDWARE_CHANNEL.sender())));
        }

        HardwareTask::DelayedInterfaceEvent(duration, event) => {
            Timer::after(duration).await;
            int_sender.send(event).await;
        }
        HardwareTask::DelayedHardwareEvent(duration, event) => {
            Timer::after(duration).await;
            hw_sender.send(event).await;
        }
    }
}

pub async fn handle_display_task<D>(
    display_task: DisplayTask,
    ui: &mut Ui<'_, D>,
    hw_sender: &Sender<'_, ThreadModeRawMutex, HardwareEvent, 32>,
    int_sender: &Sender<'_, ThreadModeRawMutex, InterfaceEvent, 32>,
) where
    D: DrawTarget<Color = Rgb565>,
{
    match display_task {
        DisplayTask::SetupSplash => {
            ui.clear();
            ui.boot_splash_screen();
        }
        DisplayTask::ConfirmPowerDelivery(power_type) => {
            let (usb_type, valid) = match power_type {
                PowerType::PowerDelivery(_) => (labels::PD, true),
                PowerType::Standard(_) => (labels::STD, false),
            };

            ui.boot_splash_text(0, labels::INPUT, usb_type, valid);
        }
        DisplayTask::ConfirmSense(result) => {
            let (res, valid) = match result {
                Ok(()) => (labels::PASS, true),
                Err(()) => (labels::FAIL, false),
            };

            ui.boot_splash_text(1, labels::SENSE, res, valid);
        }
        DisplayTask::ConfirmConverter(result) => {
            let (res, valid) = match result {
                Ok(()) => (labels::PASS, true),
                Err(()) => (labels::FAIL, false),
            };

            ui.boot_splash_text(2, labels::CONVERTER, res, valid);
        }
        DisplayTask::SetupMain(power_type, ch_a_limits, ch_b_limits) => {
            ui.clear();

            ui.nav_power_info(power_type);
            ui.nav_buttons(ConfirmState::AwaitModify, None);

            let channels = [Channel::A, Channel::B];
            for channel in channels.iter() {
                let limits = match channel {
                    Channel::A => ch_a_limits,
                    Channel::B => ch_b_limits,
                };

                ui.controls_channel_box(*channel, ChannelFocus::UnselectedInactive);
                ui.controls_channel_units(*channel);

                ui.controls_submeasurement(*channel, None, limits);
                ui.controls_submeasurement_tag(*channel, SetState::Set, None);
            }
        }
        DisplayTask::UpdateReadout(channel, readout) => {
            ui.controls_measurement(channel, readout);
        }
        DisplayTask::UpdateSetpoint(channel,  limits, set_select) => {
            ui.controls_submeasurement(channel, set_select, limits);
        }
        DisplayTask::UpdateChannelFocus(focus_a, focus_b) => {
            let focuses = [focus_a, focus_b];
            for (i, focus) in focuses.iter().enumerate() {
                let channel = match i {
                    0 => Channel::A,
                    _ => Channel::B,
                };
                ui.controls_channel_box(channel, *focus);
            }
        }
        DisplayTask::UpdateButton(confirm_state, function_button_state) => {
            ui.nav_buttons(confirm_state, function_button_state);
        }
        DisplayTask::UpdateSetState(channel, set_state, set_select) => {
            ui.controls_submeasurement_tag(channel, set_state, set_select);
        }
        // DisplayTask::
        _ => {}
    }
}
