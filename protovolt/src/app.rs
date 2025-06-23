use embassy_time::Duration;

use defmt::*;

use crate::lib::event::{
    AppEvent, AppTask, Change, Channel, ChannelFocus, DisplayTask, FunctionButton, HardwareEvent,
    HardwareTask, InterfaceEvent, Limits, PowerType, Readout, SetState,
};

#[derive(Default)]
pub struct App {
    pub ch_a: ChannelState,
    pub ch_b: ChannelState,

    pub power_type: PowerType,

    pub set_state: SetState,

    pub interface_state: InterfaceState,
    pub hardware_state: HardwareState,
}

struct ChannelState {
    pub enable: bool,
    pub limits: Limits,

    pub readout: Option<Readout>,
}

impl Default for ChannelState {
    fn default() -> Self {
        Self {
            enable: false,
            limits: Limits {
                voltage: 5.000,
                current: 1.000,
            },
            readout: None,
        }
    }
}

// pub enum LimitSetting {
//     Voltage,
//     Current,
// }

// pub enum

struct InterfaceState {
    pub screen: Screen,
    pub selected_channel: Option<Channel>,
    // pub
}

impl Default for InterfaceState {
    fn default() -> Self {
        Self {
            screen: Screen::Boot,
            selected_channel: None,
        }
    }
}

enum HardwareState {
    PowerOn,

    WaitingForPowerDelivery,
    WaitingForSense,
    WaitingForConverter,

    WaitingMainUi,
    Standby,

    Error,
}

impl Default for HardwareState {
    fn default() -> Self {
        Self::PowerOn
    }
}

enum Screen {
    Boot,
    Main,
    Settings,
}

impl App {
    pub fn handle_event(&mut self, event: AppEvent) -> Option<AppTask> {
        match event {
            AppEvent::Hardware(hw) => self.handle_hardware_event(hw),
            AppEvent::Interface(ui) => self.handle_interface_event(ui),
        }
    }

    fn handle_hardware_event(&mut self, event: HardwareEvent) -> Option<AppTask> {
        match (&self.hardware_state, event) {
            (HardwareState::PowerOn, HardwareEvent::PowerOn) => {
                self.hardware_state = HardwareState::WaitingForPowerDelivery;

                Some(AppTask {
                    hardware: Some(HardwareTask::EnablePowerDelivery),
                    display: Some(DisplayTask::SetupSplash),
                })
            }
            (
                HardwareState::WaitingForPowerDelivery,
                HardwareEvent::PowerDeliveryReady(power_type),
            ) => {
                self.hardware_state = HardwareState::WaitingForSense;
                self.power_type = power_type;

                Some(AppTask {
                    hardware: Some(HardwareTask::EnableSense),
                    display: Some(DisplayTask::ConfirmPowerDelivery(power_type)),
                })
            }
            (HardwareState::WaitingForSense, HardwareEvent::SenseReady(result)) => {
                self.hardware_state = HardwareState::WaitingForConverter;

                Some(AppTask {
                    hardware: Some(HardwareTask::EnableConverter),
                    display: Some(DisplayTask::ConfirmSense(result)),
                })
            }
            (HardwareState::WaitingForConverter, HardwareEvent::ConverterReady(result)) => {
                self.hardware_state = HardwareState::WaitingMainUi;

                Some(AppTask {
                    hardware: Some(HardwareTask::DelayedHardwareEvent(
                        Duration::from_millis(500),
                        HardwareEvent::StartMainInterface,
                    )),
                    display: Some(DisplayTask::ConfirmConverter(result)),
                })
            }
            (HardwareState::WaitingMainUi, HardwareEvent::StartMainInterface) => {
                self.hardware_state = HardwareState::Standby;
                self.interface_state.screen = Screen::Main;

                Some(AppTask {
                    hardware: Some(HardwareTask::EnableReadoutLoop),
                    display: Some(DisplayTask::SetupMain(
                        self.power_type,
                        self.ch_a.limits,
                        self.ch_b.limits,
                    )),
                })
            }
            (HardwareState::Standby, HardwareEvent::ReadoutAcquired(channel, readout)) => {
                let current_readout = match channel {
                    Channel::A => &mut self.ch_a.readout,
                    Channel::B => &mut self.ch_b.readout,
                };
                *current_readout = Some(readout);

                Some(AppTask {
                    hardware: None,
                    display: Some(DisplayTask::UpdateReadout(channel, readout)),
                })
            }
            _ => None,
        }
    }

    fn handle_interface_event(&mut self, event: InterfaceEvent) -> Option<AppTask> {
        match event {
            InterfaceEvent::ButtonSettings(change) => match change {
                Change::Pressed => Some(AppTask {
                    hardware: None,
                    display: Some(DisplayTask::UpdateButton(Some(FunctionButton::Settings))),
                }),
                Change::Released => Some(AppTask {
                    hardware: None,
                    display: Some(DisplayTask::UpdateButton(None)),
                }),
            },
            InterfaceEvent::ButtonSwitch(change) => match change {
                Change::Pressed => Some(AppTask {
                    hardware: None,
                    display: Some(DisplayTask::UpdateButton(Some(FunctionButton::Switch))),
                }),
                Change::Released => Some(AppTask {
                    hardware: None,
                    display: Some(DisplayTask::UpdateButton(None)),
                }),
            },
            InterfaceEvent::ButtonEnter(change) => match change {
                Change::Pressed => Some(AppTask {
                    hardware: None,
                    display: Some(DisplayTask::UpdateButton(Some(FunctionButton::Enter))),
                }),
                Change::Released => Some(AppTask {
                    hardware: None,
                    display: Some(DisplayTask::UpdateButton(None)),
                }),
            },
            InterfaceEvent::ButtonRight => None,
            InterfaceEvent::ButtonUp => None,
            InterfaceEvent::ButtonDown => None,
            InterfaceEvent::ButtonLeft => None,
            InterfaceEvent::ButtonChannel(event_channel) => {
                let (current_state, other_state) = match event_channel {
                    Channel::A => (&mut self.ch_a, &self.ch_b),
                    Channel::B => (&mut self.ch_b, &self.ch_a),
                };

                let selected_channel = &mut self.interface_state.selected_channel;

                if selected_channel.as_ref() == Some(&event_channel) {
                    current_state.enable = !current_state.enable;
                };
                *selected_channel = Some(event_channel);

                let other_focus = if other_state.enable {
                    ChannelFocus::UnselectedActive
                } else {
                    ChannelFocus::UnselectedInactive
                };
                let current_focus = if current_state.enable {
                    ChannelFocus::SelectedActive
                } else {
                    ChannelFocus::SelectedInactive
                };

                let (focus_a, focus_b) = match event_channel {
                    Channel::A => (current_focus, other_focus),
                    Channel::B => (other_focus, current_focus),
                };

                Some(AppTask {
                    hardware: None,
                    display: Some(DisplayTask::UpdateChannelFocus(focus_a, focus_b)),
                })
            }
        }
    }
}
