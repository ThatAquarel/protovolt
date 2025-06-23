use embassy_time::Duration;

use defmt::*;

use crate::lib::event::{
    AppEvent, AppTask, AppTaskBuilder, Change, Channel, ChannelFocus, DisplayTask, FunctionButton,
    HardwareEvent, HardwareTask, InterfaceEvent, Limits, PowerType, Readout, SetState,
};

#[derive(Default)]
pub struct App {
    pub ch_a: ChannelState,
    pub ch_b: ChannelState,

    pub power_type: PowerType,

    pub set_modify: bool,
    pub set_state: SetState,

    pub interface_state: InterfaceState,
    pub hardware_state: HardwareState,
}

#[derive(Default)]
pub enum SetSelect {
    #[default]
    Voltage,
    Current,
}

struct ChannelState {
    pub enable: bool,

    pub set: Limits,
    pub set_select: SetSelect,
    pub limits: Limits,

    pub readout: Option<Readout>,
}

impl Default for ChannelState {
    fn default() -> Self {
        Self {
            enable: false,
            set: Limits {
                voltage: 5.000,
                current: 1.000,
            },
            set_select: Default::default(),
            limits: Limits {
                voltage: 20.000,
                current: 5.000,
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

                AppTaskBuilder::new()
                    .hardware(HardwareTask::EnablePowerDelivery)
                    .display(DisplayTask::SetupSplash)
                    .build()
            }
            (
                HardwareState::WaitingForPowerDelivery,
                HardwareEvent::PowerDeliveryReady(power_type),
            ) => {
                self.hardware_state = HardwareState::WaitingForSense;
                self.power_type = power_type;

                AppTaskBuilder::new()
                    .hardware(HardwareTask::EnableSense)
                    .display(DisplayTask::ConfirmPowerDelivery(power_type))
                    .build()
            }
            (HardwareState::WaitingForSense, HardwareEvent::SenseReady(result)) => {
                self.hardware_state = HardwareState::WaitingForConverter;

                AppTaskBuilder::new()
                    .hardware(HardwareTask::EnableConverter)
                    .display(DisplayTask::ConfirmSense(result))
                    .build()
            }
            (HardwareState::WaitingForConverter, HardwareEvent::ConverterReady(result)) => {
                self.hardware_state = HardwareState::WaitingMainUi;

                AppTaskBuilder::new()
                    .hardware(HardwareTask::DelayedHardwareEvent(
                        Duration::from_millis(500),
                        HardwareEvent::StartMainInterface,
                    ))
                    .display(DisplayTask::ConfirmConverter(result))
                    .build()
            }
            (HardwareState::WaitingMainUi, HardwareEvent::StartMainInterface) => {
                self.hardware_state = HardwareState::Standby;
                self.interface_state.screen = Screen::Main;

                let (ch_a_limit, ch_b_limit) = self.get_current_set();

                AppTaskBuilder::new()
                    .hardware(HardwareTask::EnableReadoutLoop)
                    .display(DisplayTask::SetupMain(
                        self.power_type,
                        ch_a_limit,
                        ch_b_limit,
                    ))
                    .build()
            }
            (HardwareState::Standby, HardwareEvent::ReadoutAcquired(channel, readout)) => {
                let current_readout = match channel {
                    Channel::A => &mut self.ch_a.readout,
                    Channel::B => &mut self.ch_b.readout,
                };
                *current_readout = Some(readout);

                AppTaskBuilder::display_task(DisplayTask::UpdateReadout(channel, readout))
            }
            _ => None,
        }
    }

    fn handle_interface_event(&mut self, event: InterfaceEvent) -> Option<AppTask> {
        match event {
            InterfaceEvent::ButtonSettings(change) => match change {
                Change::Pressed => AppTaskBuilder::new()
                    .display(DisplayTask::UpdateButton(Some(FunctionButton::Settings)))
                    .build(),
                Change::Released => AppTaskBuilder::new()
                    .display(DisplayTask::UpdateButton(None))
                    .build(),
            },
            InterfaceEvent::ButtonSwitch(change) => match change {
                Change::Pressed => {
                    self.set_state = match self.set_state {
                        SetState::Set => SetState::Limits,
                        SetState::Limits => SetState::Set,
                    };

                    AppTaskBuilder::new()
                        .display(DisplayTask::UpdateButton(Some(FunctionButton::Switch)))
                        .build()
                }
                Change::Released => AppTaskBuilder::new()
                    .display(DisplayTask::UpdateButton(None))
                    .build(),
            },
            InterfaceEvent::ButtonEnter(change) => match change {
                Change::Pressed => AppTaskBuilder::new()
                    .display(DisplayTask::UpdateButton(Some(FunctionButton::Enter)))
                    .build(),
                Change::Released => AppTaskBuilder::new()
                    .display(DisplayTask::UpdateButton(None))
                    .build(),
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

                AppTaskBuilder::new()
                    .display(DisplayTask::UpdateChannelFocus(focus_a, focus_b))
                    .build()
            }
        }
    }

    pub fn get_current_set(&mut self) -> (Limits, Limits) {
        match self.set_state {
            SetState::Set => (self.ch_a.set, self.ch_b.set),
            SetState::Limits => (self.ch_a.limits, self.ch_b.limits),
        }
    }
}
