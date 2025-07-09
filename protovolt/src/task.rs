use defmt::*;

use embassy_rp::pio::Instance;
use embassy_sync::blocking_mutex::raw::{RawMutex, ThreadModeRawMutex};
use embassy_sync::channel::Sender;
use embassy_time::Timer;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::DrawTarget;
use embedded_hal::i2c::I2c;

use crate::hal::event::{
    Channel, ChannelFocus, ConfirmState, DisplayTask, HardwareEvent, HardwareTask, InterfaceEvent, Limits, PowerType, SetState
};
use crate::hal::Hal;
use crate::ui::{Ui, labels};

pub async fn handle_hardware_task<M, BUS>(
    hardware_task: HardwareTask,
    hal: &mut Hal<'_, M, BUS>,
    hw_sender: &Sender<'_, ThreadModeRawMutex, HardwareEvent, 32>,
    _int_sender: &Sender<'_, ThreadModeRawMutex, InterfaceEvent, 32>,
) 
where
    M: RawMutex,
    BUS: I2c,
{
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
            hal.enable_sense().await;
            info!("enable sense");
            Timer::after_millis(10).await;
        }
        HardwareTask::EnableConverter => {
            let res = hal.enable_converter().await;
            Timer::after_millis(10).await;
            hw_sender.send(HardwareEvent::ConverterReady(res)).await;
        }
        HardwareTask::EnableReadoutLoop => {
            hal.enable_readout_loop().await;
            info!("enable readout loop");
        }
        // HardwareTask::DelayedInterfaceEvent(duration, event) => {
        //     Timer::after(duration).await;
        //     int_sender.send(event).await;
        // }
        HardwareTask::DelayedHardwareEvent(duration, event) => {
            Timer::after(duration).await;
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
    _hw_sender: &Sender<'_, ThreadModeRawMutex, HardwareEvent, 32>,
    _int_sender: &Sender<'_, ThreadModeRawMutex, InterfaceEvent, 32>,
) where
    D: DrawTarget<Color = Rgb565>,
    PIO: Instance,
{
    match display_task {
        DisplayTask::SetupSplash => {
            ui.clear().unwrap();
            ui.boot_splash_screen().unwrap();
        }
        DisplayTask::ConfirmPowerDelivery(power_type) => {
            let (usb_type, valid) = match power_type {
                PowerType::PowerDelivery(_) => (labels::PD, true),
                PowerType::Standard(_) => (labels::STD, false),
            };

            ui.boot_splash_text(0, labels::INPUT, usb_type, valid).unwrap();
        }
        DisplayTask::ConfirmSense(result) => {
            let (res, valid) = match result {
                Ok(()) => (labels::PASS, true),
                Err(()) => (labels::FAIL, false),
            };

            ui.boot_splash_text(1, labels::SENSE, res, valid).unwrap();
        }
        DisplayTask::ConfirmConverter(result) => {
            let (res, valid) = match result {
                Ok(()) => (labels::PASS, true),
                Err(()) => (labels::FAIL, false),
            };

            ui.boot_splash_text(2, labels::CONVERTER, res, valid).unwrap();
        }
        DisplayTask::SetupMain(power_type, ch_a_limits, ch_b_limits) => {
            ui.clear().unwrap();

            ui.nav_power_info(power_type).unwrap();
            ui.nav_buttons(ConfirmState::AwaitModify, None).await.unwrap();

            let channels = [Channel::A, Channel::B];
            for channel in channels.iter() {
                let limits = match channel {
                    Channel::A => ch_a_limits,
                    Channel::B => ch_b_limits,
                };

                ui.controls_channel_box(*channel, ChannelFocus::UnselectedInactive).await.unwrap();
                ui.controls_channel_units(*channel).unwrap();

                ui.controls_submeasurement(*channel, None, limits, ConfirmState::AwaitModify, None).unwrap();
                ui.controls_submeasurement_tag(*channel, SetState::Set, None, ConfirmState::AwaitModify).unwrap();
            }
        }
        DisplayTask::UpdateReadout(channel, readout) => {
            ui.controls_measurement(channel, readout).unwrap();
        }
        DisplayTask::UpdateSetpoint(channel,  limits, set_select, confirm_state, precision) => {
            ui.controls_submeasurement(channel, set_select, limits, confirm_state, precision).unwrap();
        }
        DisplayTask::UpdateChannelFocus(focus_a, focus_b) => {
            let focuses = [focus_a, focus_b];
            for (i, focus) in focuses.iter().enumerate() {
                let channel = match i {
                    0 => Channel::A,
                    _ => Channel::B,
                };
                ui.controls_channel_box(channel, *focus).await.unwrap();
            }
        }
        DisplayTask::UpdateButton(confirm_state, function_button_state) => {
            ui.nav_buttons(confirm_state, function_button_state).await.unwrap();
        }
        DisplayTask::UpdateSetState(channel, set_state, set_select, confirm_state) => {
            ui.controls_submeasurement_tag(channel, set_state, set_select, confirm_state).unwrap();
        }
    }
}
