use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::DrawTarget;
use protov_core::model::{
    Channel, ChannelFocus, ChannelHardwareState, ConfirmState, DisplayTask, PowerType, SetState,
};

use crate::{
    appearance::ChannelAppearance,
    labels,
    platform::UiPlatform,
    renderer::UiRenderer,
};

pub fn dispatch_display_task<D, A, P>(
    display_task: DisplayTask,
    ui: &mut UiRenderer<D>,
    appearance: &A,
    platform: &mut P,
) -> Result<(), ()>
where
    D: DrawTarget<Color = Rgb565>,
    A: ChannelAppearance,
    P: UiPlatform,
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
            | DisplayTask::DfuStatus(_)
    );

    if ui.dfu_screen_active() && !matches!(display_task, DisplayTask::DfuStatus(_)) {
        return Ok(());
    }

    match display_task {
        DisplayTask::SetupSplash => {
            ui.clear(platform)?;
            #[cfg(feature = "demo")]
            ui.boot_demo_mode()?;
            ui.boot_splash_screen()?;
        }
        DisplayTask::ConfirmPowerDelivery(power_type) => {
            let (usb_type, valid) = match power_type {
                PowerType::PowerDelivery(_) => (labels::PD, true),
                PowerType::Standard(_) => (labels::STD, false),
            };
            ui.boot_splash_text(0, labels::INPUT, usb_type, valid)?;
        }
        DisplayTask::ConfirmSense(result) => {
            let (res, valid) = match result {
                Ok(()) => (labels::PASS, true),
                Err(()) => (labels::FAIL, false),
            };
            ui.boot_splash_text(1, labels::SENSE, res, valid)?;
        }
        DisplayTask::ConfirmConverter(result) => {
            let (res, valid) = match result {
                Ok(()) => (labels::PASS, true),
                Err(()) => (labels::FAIL, false),
            };
            ui.boot_splash_text(2, labels::CONVERTER, res, valid)?;
        }
        DisplayTask::SetupMain(power_type, ch_a_limits, ch_b_limits) => {
            ui.clear(platform)?;
            ui.nav_power_info(power_type, platform)?;
            ui.nav_buttons(appearance, platform, ConfirmState::AwaitModify, None)?;

            let channels = [Channel::A, Channel::B];
            for channel in channels.iter() {
                let limits = match channel {
                    Channel::A => ch_a_limits,
                    Channel::B => ch_b_limits,
                };

                let initial_focus = ChannelFocus::UnselectedInactive;
                ui.controls_channel_box(appearance, platform, *channel, initial_focus)?;
                ui.controls_header_chip(
                    appearance,
                    *channel,
                    initial_focus,
                    ChannelHardwareState::Off,
                )?;
                ui.controls_channel_units(*channel)?;

                ui.controls_submeasurement(
                    appearance,
                    *channel,
                    None,
                    limits,
                    ConfirmState::AwaitModify,
                    None,
                )?;
                ui.controls_submeasurement_tag(
                    appearance,
                    *channel,
                    SetState::Set,
                    None,
                    ConfirmState::AwaitModify,
                )?;
            }
        }
        DisplayTask::UpdatePowerInfo(power_type) => {
            ui.nav_power_info(power_type, platform)?;
        }
        DisplayTask::UpdateReadout(channel, readout) => {
            ui.controls_measurement(channel, readout)?;
        }
        DisplayTask::UpdateSetpoint(channel, limits, set_select, confirm_state, precision) => {
            ui.controls_submeasurement(
                appearance,
                channel,
                set_select,
                limits,
                confirm_state,
                precision,
            )?;
        }
        DisplayTask::UpdateChannelFocus(focus_a, focus_b, hw_state_a, hw_state_b) => {
            let focuses = [focus_a, focus_b];
            for (i, focus) in focuses.iter().enumerate() {
                let (channel, hw_state) = match i {
                    0 => (Channel::A, hw_state_a),
                    _ => (Channel::B, hw_state_b),
                };
                ui.controls_channel_box(appearance, platform, channel, *focus)?;
                ui.controls_header_chip(appearance, channel, *focus, hw_state)?;
            }
        }
        DisplayTask::UpdateButton(confirm_state, function_button_state) => {
            ui.nav_buttons(appearance, platform, confirm_state, function_button_state)?;
        }
        DisplayTask::UpdateSetState(channel, set_state, set_select, confirm_state) => {
            ui.controls_submeasurement_tag(
                appearance,
                channel,
                set_state,
                set_select,
                confirm_state,
            )?;
        }
        DisplayTask::UpdateChannelHardwareState(channel, focus, hw_state) => {
            ui.controls_header_chip(appearance, channel, focus, hw_state)?;
        }
        DisplayTask::UpdateSettings(visible) => {
            ui.set_settings_visible(visible);
            if visible {
                ui.draw_settings_overlay()?;
            } else {
                ui.clear_settings_section()?;
            }
        }
        DisplayTask::UpdateChannelUnits(channel) => {
            if !ui.settings_visible() {
                ui.controls_channel_units(channel)?;
            }
        }
        DisplayTask::DfuStatus(status) => {
            ui.draw_dfu_status(status, platform)?;
        }
    }

    if !skip_settings_redraw {
        ui.redraw_settings_overlay_if_visible()?;
    }

    Ok(())
}
