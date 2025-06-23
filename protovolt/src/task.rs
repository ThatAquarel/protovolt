use defmt::*;

use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Sender;
use embassy_time::{Duration, Ticker, Timer};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::DrawTarget;

use crate::lib::event::{
    Channel, ChannelFocus, DisplayTask, HardwareEvent, HardwareTask, InterfaceEvent, Limits, PowerType, SetState
};
use crate::ui::{Ui, color_scheme};
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

            hw_sender.send(HardwareEvent::PowerDeliveryReady).await;
        }
        HardwareTask::EnableSense => {
            Timer::after_millis(10).await;

            hw_sender.send(HardwareEvent::SenseReady).await;
        }
        HardwareTask::EnableConverter => {
            Timer::after_millis(10).await;

            hw_sender.send(HardwareEvent::ConverterReady).await;
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
        DisplayTask::ConfirmPowerDelivery => {
            ui.boot_splash_text(0, "INPUT", "PD 20V 5A", true);
        }
        DisplayTask::ConfirmSense => {
            ui.boot_splash_text(1, "SENSE", "CH A ERR", false);
        }
        DisplayTask::ConfirmConverter => {
            ui.boot_splash_text(2, "CONVERTER", "CH A, CH B", true);
        }
        DisplayTask::SetupMain => {
            ui.clear();

            let channels = [Channel::A, Channel::B];

            for channel in channels.iter() {
                ui.controls_channel_box(*channel, ChannelFocus::UnselectedInactive);
                ui.controls_channel_units(*channel);

                ui.controls_submeasurement(
                    *channel,
                    Limits {
                        voltage: 12.024,
                        current: 59.014,
                    },
                );
                ui.controls_submeasurement_tag(*channel, SetState::SetLimits);

                ui.nav_power_info(PowerType::PowerDelivery(Limits {
                    voltage: 20.1,
                    current: 4.56,
                }));
                ui.nav_buttons(None);
            }
        }
        DisplayTask::UpdateReadout(channel, readout) => {
            ui.controls_measurement(channel, readout);
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
        DisplayTask::UpdateButton(function_button_state) => {
            ui.nav_buttons(function_button_state);
        }
        // DisplayTask::
        _ => {}
    }
}
