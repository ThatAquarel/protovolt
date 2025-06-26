use defmt::*;
use embassy_rp::pac::usb_dpram::regs::SetupPacketHigh;
use embassy_time::Duration;
use micromath::F32Ext;

use crate::lib::event::{
    AppEvent, AppTask, AppTaskBuilder, Change, Channel, ChannelFocus, ConfirmState, DisplayTask,
    FunctionButton, HardwareEvent, HardwareTask, InterfaceEvent, Limits, PowerType, Readout,
    SetState,
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

#[derive(Debug, Clone, Copy)]
pub struct SetPrecision {
    pub exponent: f32,
}

impl Default for SetPrecision {
    fn default() -> Self {
        Self { exponent: 0.0 }
    }
}

impl SetPrecision {
    fn set_exponent(&mut self, exp: f32) {
        self.exponent = exp.clamp(-2.0, 1.0);
    }

    pub fn cursor_right(&mut self) {
        self.set_exponent(self.exponent - 1.0);
    }

    pub fn cursor_left(&mut self) {
        self.set_exponent(self.exponent + 1.0);
    }
}

pub trait HasPrecisionSetter {
    fn inner(&self) -> &SetPrecision;
    fn inner_mut(&mut self) -> &mut SetPrecision;

    fn exponent(&self) -> f32 {
        self.inner().exponent
    }

    fn cursor_right(&mut self) {
        self.inner_mut().cursor_right();
    }

    fn cursor_left(&mut self) {
        self.inner_mut().cursor_left();
    }

    fn multiplier(&self) -> f32;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SetPrecisionVoltage(pub SetPrecision);

impl HasPrecisionSetter for SetPrecisionVoltage {
    fn inner(&self) -> &SetPrecision {
        &self.0
    }

    fn inner_mut(&mut self) -> &mut SetPrecision {
        &mut self.0
    }

    fn multiplier(&self) -> f32 {
        10f32.powf(self.0.exponent)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SetPrecisionCurrent(pub SetPrecision);

impl HasPrecisionSetter for SetPrecisionCurrent {
    fn inner(&self) -> &SetPrecision {
        &self.0
    }

    fn inner_mut(&mut self) -> &mut SetPrecision {
        &mut self.0
    }

    fn multiplier(&self) -> f32 {
        10f32.powf(self.0.exponent).max(0.05f32)
    }
}

pub struct WithPrecision<T, P: HasPrecisionSetter> {
    pub value: T,
    pub precision: P,
}

pub struct VoltageCurrentWithSetter {
    pub voltage: WithPrecision<f32, SetPrecisionVoltage>,
    pub current: WithPrecision<f32, SetPrecisionCurrent>,
}

impl VoltageCurrentWithSetter {
    fn new(limits: Limits) -> Self {
        let (v, i) = (limits.voltage, limits.current);
        Self {
            voltage: WithPrecision {
                value: v,
                precision: Default::default(),
            },
            current: WithPrecision {
                value: i,
                precision: Default::default(),
            },
        }
    }

    fn get_limits(&self) -> Limits {
        Limits {
            voltage: self.voltage.value,
            current: self.current.value,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub enum SetSelect {
    #[default]
    Voltage,
    Current,
}

struct ChannelState {
    pub enable: bool,

    pub set_select: SetSelect,

    pub target: VoltageCurrentWithSetter,
    pub limits: VoltageCurrentWithSetter,

    pub readout: Option<Readout>,
}

impl Default for ChannelState {
    fn default() -> Self {
        Self {
            enable: false,
            target: VoltageCurrentWithSetter::new(Limits {
                voltage: 5.000,
                current: 1.000,
            }),
            set_select: Default::default(),
            limits: VoltageCurrentWithSetter::new(Limits {
                voltage: 20.000,
                current: 5.000,
            }),
            readout: None,
        }
    }
}

#[derive(Default)]
pub enum ArrowsFunction {
    #[default]
    Navigation,
    SetpointEdit,
}

#[derive(Default)]
struct InterfaceState {
    pub screen: Screen,
    pub selected_channel: Option<Channel>,

    pub arrows_function: ArrowsFunction,
}

#[derive(Default)]
enum HardwareState {
    #[default]
    PowerOn,

    WaitingForPowerDelivery,
    WaitingForSense,
    WaitingForConverter,

    WaitingMainUi,
    Standby,

    Error,
}

#[derive(Default)]
enum Screen {
    #[default]
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

                let power_type = self.power_type;
                let (ch_a_limit, ch_b_limit) = self.get_current_set();

                AppTaskBuilder::new()
                    .hardware(HardwareTask::EnableReadoutLoop)
                    .display(DisplayTask::SetupMain(power_type, ch_a_limit, ch_b_limit))
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
                Change::Pressed => self
                    .current_confirm_state_button_task(Some(FunctionButton::Settings))
                    .build(),
                Change::Released => self.current_confirm_state_button_task(None).build(),
            },
            InterfaceEvent::ButtonSwitch(change) => match change {
                Change::Pressed => {
                    self.set_state = match self.set_state {
                        SetState::Set => SetState::Limits,
                        SetState::Limits => SetState::Set,
                    };
                    // self.current_confirm_state_button_task(None).

                    self.setpoints_task()
                        .extend(
                            self.current_confirm_state_button_task(Some(FunctionButton::Switch)),
                        )
                        .build()
                }
                Change::Released => self.current_confirm_state_button_task(None).build(),
            },
            InterfaceEvent::ButtonEnter(change) => match change {
                Change::Pressed => {
                    if self.interface_state.selected_channel == None {
                        return self
                            .current_confirm_state_button_task(Some(FunctionButton::Enter(
                                ConfirmState::AwaitModify,
                            )))
                            .build();
                    }

                    let (arrow_function, enter_confirm_state) =
                        match self.interface_state.arrows_function {
                            ArrowsFunction::Navigation => (
                                ArrowsFunction::SetpointEdit,
                                ConfirmState::AwaitConfirmModify,
                            ),
                            ArrowsFunction::SetpointEdit => {
                                (ArrowsFunction::Navigation, ConfirmState::AwaitModify)
                            }
                        };
                    self.interface_state.arrows_function = arrow_function;

                    self.current_confirm_state_button_task(Some(FunctionButton::Enter(
                        enter_confirm_state,
                    )))
                    .build()
                }
                Change::Released => {
                    let function_button = match self.interface_state.arrows_function {
                        ArrowsFunction::Navigation => None,
                        ArrowsFunction::SetpointEdit => {
                            Some(FunctionButton::Enter(ConfirmState::AwaitConfirmModify))
                        }
                    };
                    self.current_confirm_state_button_task(function_button)
                        .build()
                }
            },
            InterfaceEvent::ButtonUp => match self.interface_state.arrows_function {
                ArrowsFunction::Navigation => {
                    self.set_current_select_set(SetSelect::Voltage);
                    self.setpoints_task().build()
                }
                _ => None,
            },
            InterfaceEvent::ButtonDown => match self.interface_state.arrows_function {
                ArrowsFunction::Navigation => {
                    self.set_current_select_set(SetSelect::Current);
                    self.setpoints_task().build()
                }
                _ => None,
            },
            InterfaceEvent::ButtonRight => match self.interface_state.arrows_function {
                ArrowsFunction::Navigation => {
                    info!("right B");
                    self.navigation_channel_focus(Channel::B)
                }
                _ => None,
            },
            InterfaceEvent::ButtonLeft => match self.interface_state.arrows_function {
                ArrowsFunction::Navigation => {
                    info!("left A");
                    self.navigation_channel_focus(Channel::A)
                }
                _ => None,
            },
            InterfaceEvent::ButtonChannel(event_channel) => {
                let current_state = match event_channel {
                    Channel::A => &mut self.ch_a,
                    Channel::B => &mut self.ch_b,
                };

                let selected_channel = &mut self.interface_state.selected_channel;

                if selected_channel.as_ref() == Some(&event_channel) {
                    current_state.enable = !current_state.enable;
                };
                *selected_channel = Some(event_channel);

                self.shift_channel_focus_task(event_channel).build()
            }
        }
    }

    pub fn get_current_set_mut(
        &mut self,
    ) -> (&mut VoltageCurrentWithSetter, &mut VoltageCurrentWithSetter) {
        match self.set_state {
            SetState::Set => (&mut self.ch_a.target, &mut self.ch_b.target),
            SetState::Limits => (&mut self.ch_a.limits, &mut self.ch_b.limits),
        }
    }

    pub fn get_current_set(&mut self) -> (Limits, Limits) {
        match self.set_state {
            SetState::Set => (self.ch_a.target.get_limits(), self.ch_b.target.get_limits()),
            SetState::Limits => (self.ch_a.limits.get_limits(), self.ch_b.limits.get_limits()),
        }
    }

    pub fn get_current_select_set(&mut self) -> (Option<SetSelect>, Option<SetSelect>) {
        match self.interface_state.selected_channel {
            Some(Channel::A) => (Some(self.ch_a.set_select), None),
            Some(Channel::B) => (None, Some(self.ch_b.set_select)),
            _ => (None, None),
        }
    }

    pub fn set_current_select_set(&mut self, set_select: SetSelect) {
        match self.interface_state.selected_channel {
            Some(Channel::A) => {
                self.ch_a.set_select = set_select;
            }
            Some(Channel::B) => {
                self.ch_b.set_select = set_select;
            }
            _ => {}
        };
    }

    pub fn setpoints_task(&mut self) -> AppTaskBuilder {
        let (ch_a_set, ch_b_set) = self.get_current_set();
        let (ch_a_select, ch_b_select) = self.get_current_select_set();

        AppTaskBuilder::new()
            .display(DisplayTask::UpdateSetState(
                Channel::A,
                self.set_state,
                ch_a_select,
            ))
            .display(DisplayTask::UpdateSetState(
                Channel::B,
                self.set_state,
                ch_b_select,
            ))
            .display(DisplayTask::UpdateSetpoint(
                Channel::A,
                ch_a_set,
                ch_a_select,
            ))
            .display(DisplayTask::UpdateSetpoint(
                Channel::B,
                ch_b_set,
                ch_b_select,
            ))
    }

    pub fn shift_channel_focus_task(&mut self, channel: Channel) -> AppTaskBuilder {
        let (current_state, other_state) = match channel {
            Channel::A => (&self.ch_a, &self.ch_b),
            Channel::B => (&self.ch_b, &self.ch_a),
        };

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

        let (focus_a, focus_b) = match channel {
            Channel::A => (current_focus, other_focus),
            Channel::B => (other_focus, current_focus),
        };

        self.setpoints_task()
            .display(DisplayTask::UpdateChannelFocus(focus_a, focus_b))
    }

    pub fn current_confirm_state_button_task(
        &mut self,
        function_button: Option<FunctionButton>,
    ) -> AppTaskBuilder {
        let confirm_state = match self.interface_state.arrows_function {
            ArrowsFunction::Navigation => ConfirmState::AwaitModify,
            ArrowsFunction::SetpointEdit => ConfirmState::AwaitConfirmModify,
        };

        AppTaskBuilder::new().display(DisplayTask::UpdateButton(confirm_state, function_button))
    }

    pub fn navigation_channel_focus(&mut self, resulting_channel: Channel) -> Option<AppTask> {
        if let Some(current_channel) = self.interface_state.selected_channel {
            if current_channel == resulting_channel {
                return None;
            }

            self.interface_state.selected_channel = Some(resulting_channel);

            let (current_set_select, other_set_select) = match resulting_channel {
                Channel::A => (self.ch_b.set_select, &mut self.ch_a.set_select),
                Channel::B => (self.ch_a.set_select, &mut self.ch_b.set_select),
            };
            *other_set_select = current_set_select;

            return self.shift_channel_focus_task(resulting_channel).build();
        }

        None
    }
}
