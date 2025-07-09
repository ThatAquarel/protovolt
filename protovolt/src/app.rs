use embassy_time::Duration;
use micromath::F32Ext;

use crate::hal::event::{
    AppEvent, AppTask, AppTaskBuilder, Change, Channel, ChannelFocus, ConfirmState, DisplayTask,
    FunctionButton, HardwareEvent, HardwareTask, InterfaceEvent, Limits, PowerType, Readout,
    SetState,
};

#[derive(Default)]
pub struct App {
    power_type: PowerType,
    set_state: SetState,

    interface_state: InterfaceState,
    hardware_state: HardwareState,

    ch_a: ChannelState,
    ch_b: ChannelState,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DecimalPrecision {
    pub exponent: i8,
}

impl DecimalPrecision {
    fn set_exponent(&mut self, exp: i8) {
        self.exponent = exp.clamp(-2, 1);
    }

    pub fn get_exponent(&self) -> i8 {
        self.exponent
    }

    pub fn cursor_right(&mut self) {
        self.set_exponent(self.exponent - 1);
    }

    pub fn cursor_left(&mut self) {
        self.set_exponent(self.exponent + 1);
    }
}

#[derive(Default)]
pub struct WithPrecision {
    value: f32,
    precision: DecimalPrecision,

    // set_state: SetSelect,
    init_range: Option<(f32, f32)>,
    min_step: f32,
}

impl WithPrecision {
    fn multiplier_voltage(&self) -> f32 {
        10f32.powf(self.precision.exponent as f32)
    }

    fn multiplier(&self) -> f32 {
        self.multiplier_voltage().max(self.min_step)
    }

    pub fn increment(&mut self) {
        let next = self.value + self.multiplier();
        self.value = match self.init_range {
            Some((min, max)) => next.clamp(min, max),
            None => next,
        };
    }

    pub fn decrement(&mut self) {
        let next = self.value - self.multiplier();
        self.value = match self.init_range {
            Some((min, max)) => next.clamp(min, max),
            None => next,
        };
    }

    // pub fn precision(&self) -> DecimalPrecision {
    //     self.precision
    // }

    pub fn cursor_right(&mut self) {
        self.precision.cursor_right();
    }

    pub fn cursor_left(&mut self) {
        self.precision.cursor_left();
    }

    pub fn value(&self) -> f32 {
        return self.value;
    }

    // pub fn exponent(&self) -> i8 {
    //     self.precision.get_exponent()
    // }

    // pub fn set_range(&mut self, min: f32, max: f32) {
    //     self.init_range = Some((min, max));
    //     self.value = self.value.clamp(min, max);
    // }
}

pub struct VoltageCurrentWithSetter {
    pub voltage: WithPrecision,
    pub current: WithPrecision,
}

impl VoltageCurrentWithSetter {
    fn new(limits: Limits, voltage_range: (f32, f32), current_range: (f32, f32)) -> Self {
        let (v, i) = (limits.voltage, limits.current);
        Self {
            voltage: WithPrecision {
                value: v,
                precision: Default::default(),
                init_range: Some(voltage_range),
                min_step: 0.01,
            },
            current: WithPrecision {
                value: i,
                precision: Default::default(),
                init_range: Some(current_range),
                min_step: 0.05,
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
        let target_limits = Limits {
            voltage: 5.000,
            current: 1.000,
        };
        let max_limits = Limits {
            voltage: 20.000,
            current: 5.000,
        };

        Self {
            enable: false,
            target: VoltageCurrentWithSetter::new(
                target_limits,
                (1.0, 20.0), // voltage range
                (0.0, 5.0),  // current range
            ),
            limits: VoltageCurrentWithSetter::new(max_limits, (1.0, 20.0), (0.0, 5.0)),
            set_select: Default::default(),
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
    // Error,
}

#[derive(Default)]
enum Screen {
    #[default]
    Boot,
    Main,
    // Settings,
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

                self.initialize_converters_task()
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
                Change::Released => self.return_current_button_state_task().build(),
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
                Change::Released => self.return_current_button_state_task().build(),
            },
            InterfaceEvent::ButtonEnter(change) => match change {
                Change::Pressed => {
                    let channel = match self.interface_state.selected_channel {
                        None => {
                            return self
                                .current_confirm_state_button_task(Some(FunctionButton::Enter))
                                .build();
                        }
                        Some(channel) => channel,
                    };

                    let mut converter_task = AppTaskBuilder::new();

                    self.interface_state.arrows_function =
                        match self.interface_state.arrows_function {
                            ArrowsFunction::Navigation => ArrowsFunction::SetpointEdit,
                            ArrowsFunction::SetpointEdit => {
                                converter_task = converter_task
                                    .extend(self.update_converter_task(channel));
                                ArrowsFunction::Navigation
                            }
                        };

                    self.setpoints_task()
                        .extend(self.current_confirm_state_button_task(Some(FunctionButton::Enter)))
                        .extend(converter_task)
                        .build()
                }
                Change::Released => {
                    let function_button = match self.interface_state.arrows_function {
                        ArrowsFunction::Navigation => None,
                        ArrowsFunction::SetpointEdit => Some(FunctionButton::Enter),
                    };
                    self.current_confirm_state_button_task(function_button)
                        .build()
                    // None
                }
            },
            InterfaceEvent::ButtonUp => match self.interface_state.arrows_function {
                ArrowsFunction::Navigation => {
                    self.set_current_select_set(SetSelect::Voltage);
                    self.setpoints_task().build()
                }
                ArrowsFunction::SetpointEdit => {
                    let val = self.get_select_precision_mut();
                    if let Some(val) = val {
                        val.increment();
                        return self.setpoints_task().build();
                    }
                    None
                }
            },
            InterfaceEvent::ButtonDown => match self.interface_state.arrows_function {
                ArrowsFunction::Navigation => {
                    self.set_current_select_set(SetSelect::Current);
                    self.setpoints_task().build()
                }
                ArrowsFunction::SetpointEdit => {
                    let val = self.get_select_precision_mut();
                    if let Some(val) = val {
                        val.decrement();
                        return self.setpoints_task().build();
                    }
                    None
                }
            },
            InterfaceEvent::ButtonRight => match self.interface_state.arrows_function {
                ArrowsFunction::Navigation => self.navigation_channel_focus(Channel::B),
                ArrowsFunction::SetpointEdit => {
                    let val = self.get_select_precision_mut();
                    if let Some(val) = val {
                        val.cursor_right();
                        return self.setpoints_task().build();
                    }
                    None
                }
            },
            InterfaceEvent::ButtonLeft => match self.interface_state.arrows_function {
                ArrowsFunction::Navigation => self.navigation_channel_focus(Channel::A),
                ArrowsFunction::SetpointEdit => {
                    let val = self.get_select_precision_mut();
                    if let Some(val) = val {
                        val.cursor_left();
                        return self.setpoints_task().build();
                    }
                    None
                }
            },
            InterfaceEvent::ButtonChannel(event_channel) => {
                let current_state = match event_channel {
                    Channel::A => &mut self.ch_a,
                    Channel::B => &mut self.ch_b,
                };

                let selected_channel = &mut self.interface_state.selected_channel;

                let mut converter_update_task = AppTaskBuilder::new();

                let mut set_value_override = false;
                if selected_channel.as_ref() == Some(&event_channel) {
                    current_state.enable = !current_state.enable;
                    converter_update_task = converter_update_task.hardware(
                        HardwareTask::UpdateConverterState(event_channel, current_state.enable),
                    );
                } else {
                    self.interface_state.arrows_function = ArrowsFunction::Navigation;
                    set_value_override = true;
                }
                *selected_channel = Some(event_channel);

                if set_value_override {
                    self.shift_channel_focus_task(event_channel)
                        .extend(self.current_confirm_state_button_task(None))
                } else {
                    self.shift_channel_focus_task(event_channel)
                        .extend(converter_update_task)
                }
                .build()
            }
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

    fn get_confirm_state(&self) -> ConfirmState {
        match self.interface_state.arrows_function {
            ArrowsFunction::Navigation => ConfirmState::AwaitModify,
            ArrowsFunction::SetpointEdit => {
                ConfirmState::AwaitConfirmModify(self.interface_state.selected_channel)
            }
        }
    }

    fn get_select_precision_mut(&mut self) -> Option<&mut WithPrecision> {
        let (ch_a_set, ch_b_set) = match self.set_state {
            SetState::Set => (&mut self.ch_a.target, &mut self.ch_b.target),
            SetState::Limits => (&mut self.ch_a.limits, &mut self.ch_b.limits),
        };

        let (ch_set, set_select) = match self.interface_state.selected_channel {
            Some(Channel::A) => (ch_a_set, self.ch_a.set_select),
            Some(Channel::B) => (ch_b_set, self.ch_b.set_select),
            _ => return None,
        };

        match set_select {
            SetSelect::Voltage => Some(&mut ch_set.voltage),
            SetSelect::Current => Some(&mut ch_set.current),
        }
    }

    fn get_select_precision(&mut self) -> Option<DecimalPrecision> {
        let with_precision = self.get_select_precision_mut();

        match with_precision {
            Some(value) => Some(value.precision),
            None => None,
        }
    }

    pub fn setpoints_task(&mut self) -> AppTaskBuilder {
        let (ch_a_set, ch_b_set) = self.get_current_set();
        let (ch_a_select, ch_b_select) = self.get_current_select_set();

        let confirm_state = self.get_confirm_state();
        let select_precision = match self.interface_state.arrows_function {
            ArrowsFunction::Navigation => None,
            ArrowsFunction::SetpointEdit => self.get_select_precision(),
        };

        AppTaskBuilder::new()
            .display(DisplayTask::UpdateSetState(
                Channel::A,
                self.set_state,
                ch_a_select,
                confirm_state,
            ))
            .display(DisplayTask::UpdateSetState(
                Channel::B,
                self.set_state,
                ch_b_select,
                confirm_state,
            ))
            .display(DisplayTask::UpdateSetpoint(
                Channel::A,
                ch_a_set,
                ch_a_select,
                confirm_state,
                select_precision,
            ))
            .display(DisplayTask::UpdateSetpoint(
                Channel::B,
                ch_b_set,
                ch_b_select,
                confirm_state,
                select_precision,
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
        let confirm_state = self.get_confirm_state();
        AppTaskBuilder::new().display(DisplayTask::UpdateButton(confirm_state, function_button))
    }

    pub fn return_current_button_state_task(&mut self) -> AppTaskBuilder {
        let function_button = match self.interface_state.arrows_function {
            ArrowsFunction::Navigation => None,
            ArrowsFunction::SetpointEdit => Some(FunctionButton::Enter),
        };

        self.current_confirm_state_button_task(function_button)
    }

    pub fn update_converter_task(&self, channel: Channel) -> AppTaskBuilder {
        let target = match channel {
            Channel::A => &self.ch_a.target,
            Channel::B => &self.ch_b.target,
        };

        AppTaskBuilder::new()
            .hardware(HardwareTask::UpdateConverterVoltage(
                channel,
                target.voltage.value(),
            ))
            .hardware(HardwareTask::UpdateConverterCurrent(
                channel,
                target.current.value(),
            ))
    }

    pub fn initialize_converters_task(&self) -> AppTaskBuilder {
        self.update_converter_task(Channel::A)
            .extend(self.update_converter_task(Channel::B))
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
