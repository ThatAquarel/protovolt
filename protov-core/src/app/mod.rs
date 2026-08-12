#[allow(unused_imports)]
use micromath::F32Ext;

use crate::config::{
    CURRENT_EDIT_RANGE, ChannelProfile, FACTORY, SCPI_SYSTEM_VERSION, VOLTAGE_EDIT_RANGE,
    format_idat_parts, format_idn, hardware_revision, manufacturing_date, serial_attestation,
    serial_number,
};
use crate::dfu::{DfuAction, DfuError, DfuSession};
use crate::fmt::format_f32;
use crate::model::ConverterFlags;
use crate::model::TemperatureReading;
use crate::model::{
    AppEvent, AppTask, AppTaskBuilder, Change, Channel, ChannelFocus, ChannelHardwareState,
    ConfirmState, DecimalPrecision, DfuEvent, DfuStatus, DisplayTask, FunctionButton,
    HardwareEvent, HardwareTask, InterfaceEvent, Limits, PowerType, Readout, SetSelect, SetState,
};
use crate::protection;
use crate::scpi::colors::{self, Rgb};
use crate::scpi::parser::{ChannelParam, MeasKind, ScpiCommand};
use crate::scpi::state::{ChannelSnapshot, ScpiState};
use crate::scpi::telemetry;
use crate::scpi::{
    RESPONSE_BUF, ScpiChannel, ScpiChannelExt, ScpiContext, ScpiHandleResult, ScpiResponse,
};

pub struct AppCore {
    power_type: PowerType,
    set_state: SetState,

    interface_state: InterfaceState,
    hardware_state: HardwareState,

    last_temp: TemperatureReading,

    ch_a: ChannelState,
    ch_b: ChannelState,

    dfu: DfuSession,
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
        self.value
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = match self.init_range {
            Some((min, max)) => value.clamp(min, max),
            None => value,
        };
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

struct ChannelState {
    pub enable: bool,

    pub hw_state: ChannelHardwareState,
    pub converter_flags: Option<ConverterFlags>,

    pub set_select: SetSelect,

    pub target: VoltageCurrentWithSetter,
    pub limits: VoltageCurrentWithSetter,

    pub readout: Option<Readout>,
}

impl ChannelState {
    fn from_profile(profile: &ChannelProfile) -> Self {
        let target_limits = Limits {
            voltage: profile.voltage_set,
            current: profile.current_set,
        };
        let max_limits = Limits {
            voltage: profile.ovp,
            current: profile.ocp,
        };

        Self {
            enable: false,
            hw_state: Default::default(),
            converter_flags: None,
            target: VoltageCurrentWithSetter::new(
                target_limits,
                VOLTAGE_EDIT_RANGE,
                CURRENT_EDIT_RANGE,
            ),
            limits: VoltageCurrentWithSetter::new(
                max_limits,
                VOLTAGE_EDIT_RANGE,
                CURRENT_EDIT_RANGE,
            ),
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

impl Default for AppCore {
    fn default() -> Self {
        Self {
            power_type: Default::default(),
            set_state: Default::default(),
            interface_state: Default::default(),
            hardware_state: Default::default(),
            last_temp: Default::default(),
            ch_a: ChannelState::from_profile(&FACTORY.ch1),
            ch_b: ChannelState::from_profile(&FACTORY.ch2),
            dfu: DfuSession::new(),
        }
    }
}

#[derive(Default)]
struct InterfaceState {
    pub screen: Screen,
    pub selected_channel: Option<Channel>,

    pub arrows_function: ArrowsFunction,
    pub settings_open: bool,
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
    FirmwareUpdate,
    // Error,
}

#[derive(Default)]
enum Screen {
    #[default]
    Boot,
    Main,
    // Settings,
}

impl AppCore {
    pub fn handle_event(&mut self, event: AppEvent, scpi: &mut ScpiState) -> Option<AppTask> {
        match event {
            AppEvent::Dfu(dfu) => self.handle_dfu_event(dfu, scpi),
            AppEvent::Hardware(hw) => {
                if self.is_update_mode() {
                    return None;
                }
                self.handle_hardware_event(hw, scpi)
            }
            AppEvent::Interface(ui) => {
                if self.is_update_mode() {
                    return None;
                }
                self.handle_interface_event(ui, scpi)
            }
        }
    }

    fn handle_hardware_event(
        &mut self,
        event: HardwareEvent,
        scpi: &mut ScpiState,
    ) -> Option<AppTask> {
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
                        500,
                        HardwareEvent::StartMainInterface,
                    ))
                    .display(DisplayTask::ConfirmConverter(result))
                    .build()
            }
            (HardwareState::WaitingMainUi, HardwareEvent::StartMainInterface) => {
                self.hardware_state = HardwareState::Standby;
                self.interface_state.screen = Screen::Main;
                self.interface_state.selected_channel = Some(Channel::A);

                let power_type = self.power_type;
                let (ch_a_limit, ch_b_limit) = self.get_current_set();

                self.initialize_converters_task()
                    .hardware(HardwareTask::EnableReadoutLoop)
                    .display(DisplayTask::SetupMain(power_type, ch_a_limit, ch_b_limit))
                    .extend(self.refresh_channels_display())
                    .extend(self.setpoints_task())
                    .build()
            }
            (HardwareState::Standby, HardwareEvent::ReadoutAcquired(channel, readout)) => {
                let current_readout = match channel {
                    Channel::A => &mut self.ch_a.readout,
                    Channel::B => &mut self.ch_b.readout,
                };
                *current_readout = Some(readout);

                self.update_hw_state(scpi, channel)
                    .extend(self.display_if_allowed(DisplayTask::UpdateReadout(channel, readout)))
                    .build()
            }
            (HardwareState::Standby, HardwareEvent::TempAcquired(temperature)) => {
                self.last_temp = temperature;
                self.update_all_hw_states(scpi).build()
            }
            (HardwareState::Standby, HardwareEvent::ConverterStatusAcquired(channel, flags)) => {
                match channel {
                    Channel::A => self.ch_a.converter_flags = Some(flags),
                    Channel::B => self.ch_b.converter_flags = Some(flags),
                }
                self.update_hw_state(scpi, channel).build()
            }
            (HardwareState::Standby, HardwareEvent::PollConverterStatusInterrupt(channel)) => {
                AppTaskBuilder::new()
                    .hardware(HardwareTask::PollConverterStatus(channel))
                    .build()
            }
            _ => None,
        }
    }

    fn handle_interface_event(
        &mut self,
        event: InterfaceEvent,
        _scpi: &mut ScpiState,
    ) -> Option<AppTask> {
        match event {
            InterfaceEvent::ButtonSettings(change) => match change {
                Change::Pressed => {
                    let opening = !self.interface_state.settings_open;
                    if opening {
                        self.interface_state.arrows_function = ArrowsFunction::Navigation;
                    }
                    self.interface_state.settings_open = !self.interface_state.settings_open;
                    let mut tasks =
                        AppTaskBuilder::new()
                            .display(DisplayTask::UpdateSettings(
                                self.interface_state.settings_open,
                            ))
                            .extend(self.current_confirm_state_button_task(
                                self.navigation_function_button(),
                            ));
                    if !opening {
                        tasks = tasks.extend(self.settings_close_display_task());
                    }
                    tasks.build()
                }
                Change::Released => None,
            },
            _ if !self.interface_event_allowed(&event) => None,
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
                                converter_task =
                                    converter_task.extend(self.update_converter_task(channel));
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
                    current_state.hw_state = if current_state.enable {
                        ChannelHardwareState::ConstantVoltage
                    } else {
                        ChannelHardwareState::Off
                    };
                    converter_update_task = converter_update_task
                        .hardware(HardwareTask::UpdateConverterState(
                            event_channel,
                            current_state.enable,
                        ))
                        .hardware(HardwareTask::PollConverterStatus(event_channel));
                } else {
                    self.interface_state.arrows_function = ArrowsFunction::Navigation;
                    set_value_override = true;
                }
                *selected_channel = Some(event_channel);

                if set_value_override {
                    self.shift_channel_focus_task(event_channel)
                        .extend(self.current_confirm_state_button_task(None))
                } else {
                    converter_update_task.extend(self.shift_channel_focus_task(event_channel))
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

    fn scpi_channel(channel: Channel) -> ScpiChannel {
        match channel {
            Channel::A => ScpiChannel::Ch1,
            Channel::B => ScpiChannel::Ch2,
        }
    }

    fn header_chip_task(&self, channel: Channel) -> AppTaskBuilder {
        let hw_state = self.channel_ref(channel).hw_state;
        AppTaskBuilder::new().display(DisplayTask::UpdateChannelHardwareState(
            channel,
            self.focus_for(channel),
            hw_state,
        ))
    }

    fn refresh_channels_display(&self) -> AppTaskBuilder {
        let (focus_a, focus_b) = self.channel_focuses();
        let function_button = match self.interface_state.arrows_function {
            ArrowsFunction::Navigation => None,
            ArrowsFunction::SetpointEdit => Some(FunctionButton::Enter),
        };
        AppTaskBuilder::new()
            .display(DisplayTask::UpdateChannelFocus(
                focus_a,
                focus_b,
                self.ch_a.hw_state,
                self.ch_b.hw_state,
            ))
            .display(DisplayTask::UpdateButton(
                self.get_confirm_state(),
                function_button,
            ))
    }

    fn derive_channel_hw_state(&self, channel: Channel, scpi: &ScpiState) -> ChannelHardwareState {
        let ch = self.channel_ref(channel);
        protection::derive_hw_state(
            channel,
            ch.enable,
            ch.converter_flags,
            ch.readout,
            ch.limits.get_limits(),
            &self.last_temp,
            scpi.prot_latched(Self::scpi_channel(channel)),
            ch.hw_state,
        )
    }

    fn trip_channel(
        &mut self,
        scpi: &mut ScpiState,
        channel: Channel,
        fault: ChannelHardwareState,
    ) -> AppTaskBuilder {
        let state = self.channel_mut(channel);
        state.enable = false;
        state.hw_state = fault;
        scpi.set_prot_latched(Some(Self::scpi_channel(channel)), true);

        AppTaskBuilder::new()
            .hardware(HardwareTask::UpdateConverterState(channel, false))
            .extend(self.refresh_channels_display())
    }

    fn trip_both_channels(
        &mut self,
        scpi: &mut ScpiState,
        fault: ChannelHardwareState,
    ) -> AppTaskBuilder {
        for ch in [Channel::A, Channel::B] {
            let state = self.channel_mut(ch);
            state.enable = false;
            state.hw_state = fault;
        }
        scpi.set_prot_latched(None, true);

        AppTaskBuilder::new()
            .hardware(HardwareTask::UpdateConverterState(Channel::A, false))
            .hardware(HardwareTask::UpdateConverterState(Channel::B, false))
            .extend(self.refresh_channels_display())
    }

    fn disable_output_channels(&mut self) -> AppTaskBuilder {
        for ch in [Channel::A, Channel::B] {
            let state = self.channel_mut(ch);
            state.enable = false;
            state.hw_state = ChannelHardwareState::Off;
        }

        AppTaskBuilder::new()
            .hardware(HardwareTask::UpdateConverterState(Channel::A, false))
            .hardware(HardwareTask::UpdateConverterState(Channel::B, false))
    }

    pub fn update_hw_state(&mut self, scpi: &mut ScpiState, channel: Channel) -> AppTaskBuilder {
        if protection::mcu_overtemp(&self.last_temp) {
            if self.ch_a.enable || self.ch_b.enable {
                return self.trip_both_channels(scpi, ChannelHardwareState::OverTemperature);
            }
            let mut tasks = AppTaskBuilder::new();
            let mut changed = false;
            for ch in [Channel::A, Channel::B] {
                if self.channel_ref(ch).hw_state != ChannelHardwareState::OverTemperature {
                    self.channel_mut(ch).hw_state = ChannelHardwareState::OverTemperature;
                    changed = true;
                }
            }
            if changed {
                tasks = tasks.extend(self.refresh_channels_display());
            }
            return tasks;
        }

        let derived = self.derive_channel_hw_state(channel, scpi);
        let prev = self.channel_ref(channel).hw_state;
        let prot = scpi.prot_latched(Self::scpi_channel(channel));

        if protection::is_fault_state(derived) {
            if !prot || self.channel_ref(channel).enable {
                return self.trip_channel(scpi, channel, derived);
            }
            if derived != prev {
                self.channel_mut(channel).hw_state = derived;
                return self.header_chip_task(channel);
            }
            return AppTaskBuilder::new();
        }

        if derived != prev {
            self.channel_mut(channel).hw_state = derived;
            return self.header_chip_task(channel);
        }
        AppTaskBuilder::new()
    }

    pub fn update_all_hw_states(&mut self, scpi: &mut ScpiState) -> AppTaskBuilder {
        if protection::mcu_overtemp(&self.last_temp) {
            return self.update_hw_state(scpi, Channel::A);
        }
        self.update_hw_state(scpi, Channel::A)
            .extend(self.update_hw_state(scpi, Channel::B))
    }

    fn focus_for(&self, channel: Channel) -> ChannelFocus {
        let selected = self.interface_state.selected_channel.unwrap_or(Channel::A);
        let enabled = match channel {
            Channel::A => self.ch_a.enable,
            Channel::B => self.ch_b.enable,
        };
        Self::channel_focus(selected == channel, enabled)
    }

    fn channel_focuses(&self) -> (ChannelFocus, ChannelFocus) {
        (self.focus_for(Channel::A), self.focus_for(Channel::B))
    }

    fn channel_focus(selected: bool, active: bool) -> ChannelFocus {
        if selected {
            match active {
                true => ChannelFocus::SelectedActive,
                false => ChannelFocus::SelectedInactive,
            }
        } else {
            match active {
                true => ChannelFocus::UnselectedActive,
                false => ChannelFocus::UnselectedInactive,
            }
        }
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
            .display(DisplayTask::UpdateChannelFocus(
                focus_a,
                focus_b,
                self.ch_a.hw_state,
                self.ch_b.hw_state,
            ))
    }

    pub fn current_confirm_state_button_task(
        &mut self,
        function_button: Option<FunctionButton>,
    ) -> AppTaskBuilder {
        let confirm_state = self.get_confirm_state();
        AppTaskBuilder::new().display(DisplayTask::UpdateButton(confirm_state, function_button))
    }

    pub fn return_current_button_state_task(&mut self) -> AppTaskBuilder {
        self.current_confirm_state_button_task(self.navigation_function_button())
    }

    fn navigation_function_button(&self) -> Option<FunctionButton> {
        if self.interface_state.settings_open {
            Some(FunctionButton::Settings)
        } else {
            match self.interface_state.arrows_function {
                ArrowsFunction::Navigation => None,
                ArrowsFunction::SetpointEdit => Some(FunctionButton::Enter),
            }
        }
    }

    pub fn appearance_refresh_task(&mut self) -> AppTaskBuilder {
        let (focus_a, focus_b) = self.channel_focuses();

        AppTaskBuilder::new()
            .display(DisplayTask::UpdateChannelFocus(
                focus_a,
                focus_b,
                self.ch_a.hw_state,
                self.ch_b.hw_state,
            ))
            .display(DisplayTask::UpdateButton(
                self.get_confirm_state(),
                self.navigation_function_button(),
            ))
            .extend(self.setpoints_task())
    }

    pub fn settings_close_display_task(&mut self) -> AppTaskBuilder {
        self.appearance_refresh_task()
            .extend(self.channel_units_display_task())
    }

    pub fn channel_units_display_task(&self) -> AppTaskBuilder {
        AppTaskBuilder::new()
            .display(DisplayTask::UpdateChannelUnits(Channel::A))
            .display(DisplayTask::UpdateChannelUnits(Channel::B))
    }

    pub fn interface_event_allowed(&self, event: &InterfaceEvent) -> bool {
        !(self.interface_state.settings_open && event.blocked_while_settings_open())
    }

    pub fn allows_display(&self, task: &DisplayTask) -> bool {
        !(self.interface_state.settings_open && task.blocked_while_settings_open())
    }

    pub fn display_if_allowed(&self, task: DisplayTask) -> AppTaskBuilder {
        if self.allows_display(&task) {
            AppTaskBuilder::new().display(task)
        } else {
            AppTaskBuilder::new()
        }
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

    pub fn is_standby(&self) -> bool {
        matches!(self.hardware_state, HardwareState::Standby)
    }

    pub fn is_update_mode(&self) -> bool {
        matches!(self.hardware_state, HardwareState::FirmwareUpdate)
    }

    pub fn dfu_session(&self) -> &DfuSession {
        &self.dfu
    }

    fn dfu_status_task(&self) -> AppTaskBuilder {
        AppTaskBuilder::new().display(DisplayTask::DfuStatus(self.dfu.to_status()))
    }

    fn handle_dfu_event(&mut self, event: DfuEvent, _scpi: &mut ScpiState) -> Option<AppTask> {
        match event {
            DfuEvent::PrepareComplete => {
                if self.dfu.on_prepare_complete().is_err() {
                    return None;
                }
                None
            }
            DfuEvent::PrepareFailed => {
                self.dfu.on_prepare_failed();
                self.dfu_status_task().build()
            }
            DfuEvent::BlockWriteComplete { .. } => self.dfu_status_task().build(),
            DfuEvent::BlockWriteFailed => {
                self.dfu.on_block_failed();
                self.dfu_status_task().build()
            }
            DfuEvent::VerifyApplyComplete => AppTaskBuilder::new()
                .display(DisplayTask::DfuStatus(DfuStatus::Verified))
                .build(),
            DfuEvent::VerifyApplyFailed => {
                self.dfu.on_verify_failed();
                self.dfu_status_task().build()
            }
        }
    }

    fn fwup_ok_result(tasks: Option<AppTask>) -> ScpiHandleResult {
        ScpiHandleResult {
            response: ScpiResponse::ok(),
            tasks,
        }
    }

    fn fwup_err_result(scpi: &mut ScpiState, code: i32, msg: &'static str) -> ScpiHandleResult {
        scpi.push_error(code, msg);
        ScpiHandleResult {
            response: Self::push_response_text("ERR"),
            tasks: None,
        }
    }

    fn handle_fwup_star(&mut self, size: u32, scpi: &mut ScpiState) -> ScpiHandleResult {
        if self.is_update_mode() {
            return Self::fwup_err_result(scpi, -200, "Update already in progress");
        }
        if !self.is_standby() {
            scpi.push_error(-221, "Not in Standby");
            return ScpiHandleResult {
                response: ScpiResponse::none(),
                tasks: None,
            };
        }
        match self.dfu.start(size) {
            Ok(DfuAction::Prepare) => {
                self.hardware_state = HardwareState::FirmwareUpdate;
                let tasks = AppTaskBuilder::new()
                    .display(DisplayTask::DfuStatus(DfuStatus::Preparing { total: size }))
                    .extend(self.disable_output_channels())
                    .hardware(HardwareTask::DfuPrepare)
                    .build();
                Self::fwup_ok_result(tasks)
            }
            Err(e) => Self::fwup_err_result(scpi, -200, e.scpi_message()),
            _ => Self::fwup_err_result(scpi, -200, "Unexpected update action"),
        }
    }

    fn handle_fwup_abor(&mut self) -> ScpiHandleResult {
        self.dfu.abort();
        if self.is_update_mode() {
            self.hardware_state = HardwareState::Standby;
        }
        let tasks = AppTaskBuilder::new()
            .display(DisplayTask::DfuStatus(DfuStatus::Idle))
            .build();
        Self::fwup_ok_result(tasks)
    }

    fn handle_fwup_stat(&self) -> ScpiHandleResult {
        let mut stat = heapless::String::<64>::new();
        self.dfu.format_stat(&mut stat);
        ScpiHandleResult {
            response: Self::push_response_text(stat.as_str()),
            tasks: None,
        }
    }

    fn handle_fwup_appl(&mut self, signature: [u8; 64], scpi: &mut ScpiState) -> ScpiHandleResult {
        if !self.is_update_mode() {
            return Self::fwup_err_result(scpi, -200, "Use SYST:FWUP:STAR first");
        }
        match self.dfu.apply(signature) {
            Ok(DfuAction::VerifyApply { len, signature }) => {
                let tasks = AppTaskBuilder::new()
                    .hardware(HardwareTask::DfuVerifyApply { len, signature })
                    .build();
                ScpiHandleResult {
                    response: ScpiResponse::none(),
                    tasks,
                }
            }
            Err(e) => Self::fwup_err_result(scpi, -200, e.scpi_message()),
            _ => Self::fwup_err_result(scpi, -200, "Unexpected update action"),
        }
    }

    /// Build the deferred `SYST:FWUP:APPL` response after hardware verify completes.
    pub fn fwup_appl_response(&mut self, success: bool, scpi: &mut ScpiState) -> ScpiResponse {
        if success {
            ScpiResponse::ok()
        } else {
            scpi.push_error(-200, DfuError::VerifyFailed.scpi_message());
            Self::push_response_text("ERR")
        }
    }

    pub fn handle_fwup_data(&mut self, block_len: u32, scpi: &mut ScpiState) -> ScpiHandleResult {
        if !self.is_update_mode() {
            return Self::fwup_err_result(scpi, -200, "Use SYST:FWUP:STAR first");
        }
        if !self.dfu.accepts_data() {
            return Self::fwup_err_result(scpi, -200, "Not receiving firmware");
        }
        match self.dfu.accept_block(block_len) {
            Ok((_offset, DfuAction::WriteBlock { offset, len })) => {
                let tasks = AppTaskBuilder::new()
                    .hardware(HardwareTask::DfuWriteBlock { offset, len })
                    .build();
                Self::fwup_ok_result(tasks)
            }
            Err(e) => Self::fwup_err_result(scpi, -200, e.scpi_message()),
            _ => Self::fwup_err_result(scpi, -200, "Unexpected update action"),
        }
    }

    fn channel_mut(&mut self, ch: Channel) -> &mut ChannelState {
        match ch {
            Channel::A => &mut self.ch_a,
            Channel::B => &mut self.ch_b,
        }
    }

    fn channel_ref(&self, ch: Channel) -> &ChannelState {
        match ch {
            Channel::A => &self.ch_a,
            Channel::B => &self.ch_b,
        }
    }

    fn scpi_channel_mut(&mut self, ch: ScpiChannel) -> &mut ChannelState {
        self.channel_mut(ch.to_hal())
    }

    fn scpi_channel_ref(&self, ch: ScpiChannel) -> &ChannelState {
        self.channel_ref(ch.to_hal())
    }

    fn snapshot_channel(&self, ch: ScpiChannel, scpi: &ScpiState) -> ChannelSnapshot {
        let state = self.scpi_channel_ref(ch);
        let rgb = scpi.color(ch);
        ChannelSnapshot {
            voltage_set: state.target.voltage.value(),
            current_set: state.target.current.value(),
            ovp: state.limits.voltage.value(),
            ocp: state.limits.current.value(),
            output_on: state.enable,
            prot_latched: scpi.prot_latched(ch),
            color_r: rgb.r,
            color_g: rgb.g,
            color_b: rgb.b,
        }
    }

    fn restore_channel(&mut self, ch: ScpiChannel, snap: &ChannelSnapshot) {
        let state = self.scpi_channel_mut(ch);
        state.target.voltage.set_value(snap.voltage_set);
        state.target.current.set_value(snap.current_set);
        state.limits.voltage.set_value(snap.ovp);
        state.limits.current.set_value(snap.ocp);
        state.enable = snap.output_on;
    }

    fn default_reset_channels(&mut self) {
        self.ch_a = ChannelState::from_profile(&FACTORY.ch1);
        self.ch_b = ChannelState::from_profile(&FACTORY.ch2);
    }

    fn format_f32_3(value: f32) -> heapless::String<16> {
        format_f32::<16>(value, 3)
    }

    fn push_response_text(text: &str) -> ScpiResponse {
        let mut buf = heapless::String::<RESPONSE_BUF>::new();
        let _ = buf.push_str(text);
        ScpiResponse::with_text(buf)
    }

    fn push_response_u8(value: u8) -> ScpiResponse {
        use core::fmt::Write;

        let mut buf = heapless::String::<RESPONSE_BUF>::new();
        let _ = write!(buf, "{}", value);
        ScpiResponse::with_text(buf)
    }

    fn push_response_f32(value: f32) -> ScpiResponse {
        let s = Self::format_f32_3(value);
        Self::push_response_text(s.as_str())
    }

    pub fn handle_scpi(
        &mut self,
        cmd: ScpiCommand,
        scpi: &mut ScpiState,
        ctx: &ScpiContext,
    ) -> ScpiHandleResult {
        use core::fmt::Write;

        if self.is_update_mode() && !crate::scpi::parser::is_allowed_in_update_mode(&cmd) {
            return Self::fwup_err_result(scpi, -200, "Only FWUP commands in update mode");
        }

        if !self.is_update_mode() && crate::scpi::parser::requires_active_update_session(&cmd) {
            return Self::fwup_err_result(scpi, -200, "Use SYST:FWUP:STAR first");
        }

        match cmd {
            ScpiCommand::FwupStatQuery => return self.handle_fwup_stat(),
            ScpiCommand::FwupStar { size } => return self.handle_fwup_star(size, scpi),
            ScpiCommand::FwupAbor => return self.handle_fwup_abor(),
            ScpiCommand::FwupAppl { signature } => return self.handle_fwup_appl(signature, scpi),
            ScpiCommand::FwupData => {
                return Self::fwup_err_result(scpi, -200, "Missing firmware block length");
            }
            _ => {}
        }

        if crate::scpi::parser::is_mutation(&cmd) && !self.is_standby() {
            scpi.push_error(-221, "Not in Standby; mutating command rejected");
            return ScpiHandleResult {
                response: ScpiResponse::none(),
                tasks: None,
            };
        }

        match cmd {
            ScpiCommand::Unknown {
                command,
                command_len,
            } => {
                let cmd_text = core::str::from_utf8(&command[..command_len as usize]).unwrap_or("");
                let mut msg = heapless::String::<64>::new();
                let _ = write!(msg, "Unknown command: {}", cmd_text);
                scpi.push_error(-113, msg.as_str());
                ScpiHandleResult {
                    response: ScpiResponse::none(),
                    tasks: None,
                }
            }
            ScpiCommand::IdnQuery => {
                let mut buf = heapless::String::<RESPONSE_BUF>::new();
                format_idn(&mut buf);
                ScpiHandleResult {
                    response: ScpiResponse::with_text(buf),
                    tasks: None,
                }
            }
            ScpiCommand::SystIdatQuery => {
                let mut buf = heapless::String::<RESPONSE_BUF>::new();
                if format_idat_parts(
                    serial_number(),
                    hardware_revision(),
                    manufacturing_date(),
                    &ctx.flash_unique_id,
                    serial_attestation(),
                    &mut buf,
                )
                .is_err()
                {
                    scpi.push_error(-200, "Identification response too large");
                    ScpiHandleResult {
                        response: ScpiResponse::none(),
                        tasks: None,
                    }
                } else {
                    ScpiHandleResult {
                        response: ScpiResponse::with_text(buf),
                        tasks: None,
                    }
                }
            }
            ScpiCommand::SystVersQuery => ScpiHandleResult {
                response: Self::push_response_text(SCPI_SYSTEM_VERSION),
                tasks: None,
            },
            ScpiCommand::SystErrQuery => {
                let (code, msg) = scpi.pop_error();
                let mut buf = heapless::String::<RESPONSE_BUF>::new();
                let _ = write!(buf, "{},\"{}\"", code, msg.as_str());
                ScpiHandleResult {
                    response: ScpiResponse::with_text(buf),
                    tasks: None,
                }
            }
            ScpiCommand::SystLoc => {
                scpi.remote = false;
                ScpiHandleResult {
                    response: ScpiResponse::none(),
                    tasks: None,
                }
            }
            ScpiCommand::SystRem => {
                scpi.remote = true;
                ScpiHandleResult {
                    response: ScpiResponse::none(),
                    tasks: None,
                }
            }
            ScpiCommand::TelemQuery => {
                let mut buf = heapless::String::<RESPONSE_BUF>::new();
                telemetry::format_telemetry(ctx, &mut buf);
                ScpiHandleResult {
                    response: ScpiResponse::with_text(buf),
                    tasks: None,
                }
            }
            ScpiCommand::TempQuery { slot } => {
                let mut buf = heapless::String::<RESPONSE_BUF>::new();
                telemetry::format_temp(ctx, slot, &mut buf);
                ScpiHandleResult {
                    response: ScpiResponse::with_text(buf),
                    tasks: None,
                }
            }
            ScpiCommand::InpQuery => {
                let mut buf = heapless::String::<RESPONSE_BUF>::new();
                telemetry::format_inp(ctx, &mut buf);
                ScpiHandleResult {
                    response: ScpiResponse::with_text(buf),
                    tasks: None,
                }
            }
            ScpiCommand::DiagQuery => {
                let mut buf = heapless::String::<RESPONSE_BUF>::new();
                telemetry::format_diag(ctx, &mut buf);
                ScpiHandleResult {
                    response: ScpiResponse::with_text(buf),
                    tasks: None,
                }
            }
            ScpiCommand::Ina226RegQuery { .. } | ScpiCommand::Tps55289RegQuery { .. } => {
                unreachable!("register dumps are handled in main")
            }
            ScpiCommand::FwupStatQuery
            | ScpiCommand::FwupStar { .. }
            | ScpiCommand::FwupData
            | ScpiCommand::FwupAppl { .. }
            | ScpiCommand::FwupAbor => {
                unreachable!("FWUP commands are handled before the main match")
            }
            ScpiCommand::MeasQuery { kind, channel } => {
                let state = self.scpi_channel_ref(channel);
                let value = if state.enable {
                    match state.readout {
                        Some(r) => match kind {
                            MeasKind::Volt => r.voltage,
                            MeasKind::Curr => r.current,
                            MeasKind::Pow => r.power,
                        },
                        None => 0.0,
                    }
                } else {
                    0.0
                };
                ScpiHandleResult {
                    response: Self::push_response_f32(value),
                    tasks: None,
                }
            }
            ScpiCommand::ChannelQuery { channel, param } => {
                let state = self.scpi_channel_ref(channel);
                match param {
                    ChannelParam::Colr => {
                        let rgb = scpi.color(channel);
                        ScpiHandleResult {
                            response: Self::push_response_text(colors::format_rgb(rgb).as_str()),
                            tasks: None,
                        }
                    }
                    ChannelParam::Mode => ScpiHandleResult {
                        response: Self::push_response_text(state.hw_state.mode_str()),
                        tasks: None,
                    },
                    ChannelParam::Volt => ScpiHandleResult {
                        response: Self::push_response_f32(state.target.voltage.value()),
                        tasks: None,
                    },
                    ChannelParam::Curr => ScpiHandleResult {
                        response: Self::push_response_f32(state.target.current.value()),
                        tasks: None,
                    },
                    ChannelParam::Ovp => ScpiHandleResult {
                        response: Self::push_response_f32(state.limits.voltage.value()),
                        tasks: None,
                    },
                    ChannelParam::Ocp => ScpiHandleResult {
                        response: Self::push_response_f32(state.limits.current.value()),
                        tasks: None,
                    },
                }
            }
            ScpiCommand::OutputQuery { channel } => {
                let on = self.scpi_channel_ref(channel).enable;
                ScpiHandleResult {
                    response: Self::push_response_text(if on { "ON" } else { "OFF" }),
                    tasks: None,
                }
            }
            ScpiCommand::Rst => {
                self.default_reset_channels();
                scpi.remote = true;
                scpi.reset_appearance();
                scpi.set_prot_latched(None, false);
                let tasks = self
                    .update_converter_task(Channel::A)
                    .extend(self.update_converter_task(Channel::B))
                    .hardware(HardwareTask::UpdateConverterState(Channel::A, false))
                    .hardware(HardwareTask::UpdateConverterState(Channel::B, false))
                    .extend(self.setpoints_task())
                    .extend(self.appearance_refresh_task())
                    .build();
                ScpiHandleResult {
                    response: ScpiResponse::none(),
                    tasks,
                }
            }
            ScpiCommand::Sav { slot } => {
                if !(1..=9).contains(&slot) {
                    let mut msg = heapless::String::<64>::new();
                    let _ = write!(msg, "Save slot {} out of range (1-9)", slot);
                    scpi.push_error(-222, msg.as_str());
                    return ScpiHandleResult {
                        response: ScpiResponse::none(),
                        tasks: None,
                    };
                }
                let ch1 = self.snapshot_channel(ScpiChannel::Ch1, scpi);
                let ch2 = self.snapshot_channel(ScpiChannel::Ch2, scpi);
                scpi.save_slot(slot, ch1, ch2);
                ScpiHandleResult {
                    response: ScpiResponse::none(),
                    tasks: None,
                }
            }
            ScpiCommand::Rcl { slot } => {
                if !(1..=9).contains(&slot) {
                    let mut msg = heapless::String::<64>::new();
                    let _ = write!(msg, "Recall slot {} out of range (1-9)", slot);
                    scpi.push_error(-222, msg.as_str());
                    return ScpiHandleResult {
                        response: ScpiResponse::none(),
                        tasks: None,
                    };
                }
                if let Some((ch1, ch2)) = scpi.recall_slot(slot) {
                    self.restore_channel(ScpiChannel::Ch1, &ch1);
                    self.restore_channel(ScpiChannel::Ch2, &ch2);
                    scpi.set_color(
                        ScpiChannel::Ch1,
                        Rgb {
                            r: ch1.color_r,
                            g: ch1.color_g,
                            b: ch1.color_b,
                        },
                    );
                    scpi.set_color(
                        ScpiChannel::Ch2,
                        Rgb {
                            r: ch2.color_r,
                            g: ch2.color_g,
                            b: ch2.color_b,
                        },
                    );
                    scpi.set_prot_latched(None, ch1.prot_latched || ch2.prot_latched);
                    let tasks = self
                        .update_converter_task(Channel::A)
                        .extend(self.update_converter_task(Channel::B))
                        .hardware(HardwareTask::UpdateConverterState(
                            Channel::A,
                            self.ch_a.enable,
                        ))
                        .hardware(HardwareTask::UpdateConverterState(
                            Channel::B,
                            self.ch_b.enable,
                        ))
                        .extend(self.setpoints_task())
                        .extend(self.appearance_refresh_task())
                        .build();
                    ScpiHandleResult {
                        response: ScpiResponse::none(),
                        tasks,
                    }
                } else {
                    let mut msg = heapless::String::<64>::new();
                    let _ = write!(msg, "Empty save slot {}", slot);
                    scpi.push_error(-222, msg.as_str());
                    ScpiHandleResult {
                        response: ScpiResponse::none(),
                        tasks: None,
                    }
                }
            }
            ScpiCommand::Del { slot } => {
                if !(1..=9).contains(&slot) {
                    let mut msg = heapless::String::<64>::new();
                    let _ = write!(msg, "Delete slot {} out of range (1-9)", slot);
                    scpi.push_error(-222, msg.as_str());
                    return ScpiHandleResult {
                        response: ScpiResponse::none(),
                        tasks: None,
                    };
                }
                scpi.delete_slot(slot);
                ScpiHandleResult {
                    response: ScpiResponse::none(),
                    tasks: None,
                }
            }
            ScpiCommand::ChannelSet {
                channel,
                param,
                value,
            } => {
                let hal_ch = channel.to_hal();
                let mut tasks = AppTaskBuilder::new();
                {
                    let state = self.scpi_channel_mut(channel);
                    match param {
                        ChannelParam::Volt => state.target.voltage.set_value(value),
                        ChannelParam::Curr => state.target.current.set_value(value),
                        ChannelParam::Ovp => state.limits.voltage.set_value(value),
                        ChannelParam::Ocp => state.limits.current.set_value(value),
                        ChannelParam::Colr | ChannelParam::Mode => unreachable!(),
                    }
                }
                if matches!(param, ChannelParam::Volt | ChannelParam::Curr) {
                    tasks = tasks.extend(self.update_converter_task(hal_ch));
                }
                ScpiHandleResult {
                    response: ScpiResponse::none(),
                    tasks: tasks.extend(self.setpoints_task()).build(),
                }
            }
            ScpiCommand::ColorSet { channel, rgb } => {
                scpi.set_color(channel, rgb);
                ScpiHandleResult {
                    response: ScpiResponse::none(),
                    tasks: self.appearance_refresh_task().build(),
                }
            }
            ScpiCommand::LcdBrightnessQuery => ScpiHandleResult {
                response: Self::push_response_u8(scpi.lcd_brightness),
                tasks: None,
            },
            ScpiCommand::LcdBrightnessSet { value } => {
                scpi.lcd_brightness = value;
                ScpiHandleResult {
                    response: ScpiResponse::none(),
                    tasks: None,
                }
            }
            ScpiCommand::LedBrightnessQuery => ScpiHandleResult {
                response: Self::push_response_u8(scpi.led_brightness),
                tasks: None,
            },
            ScpiCommand::LedBrightnessSet { value } => {
                scpi.led_brightness = value;
                ScpiHandleResult {
                    response: ScpiResponse::none(),
                    tasks: self.appearance_refresh_task().build(),
                }
            }
            ScpiCommand::OutputSet { channel, on } => {
                let hal_ch = channel.to_hal();
                let state = self.scpi_channel_mut(channel);
                state.enable = on;
                state.hw_state = if on {
                    ChannelHardwareState::ConstantVoltage
                } else {
                    ChannelHardwareState::Off
                };
                self.interface_state.selected_channel = Some(hal_ch);
                let tasks = AppTaskBuilder::new()
                    .hardware(HardwareTask::UpdateConverterState(hal_ch, on))
                    .hardware(HardwareTask::PollConverterStatus(hal_ch))
                    .extend(self.shift_channel_focus_task(hal_ch))
                    .build();
                ScpiHandleResult {
                    response: ScpiResponse::none(),
                    tasks,
                }
            }
            ScpiCommand::ResetProt { channel } => {
                scpi.set_prot_latched(channel, false);
                for ch in [Channel::A, Channel::B] {
                    if (channel.map(|c| c.to_hal()) == Some(ch) || channel.is_none())
                        && protection::is_fault_state(self.channel_ref(ch).hw_state)
                    {
                        self.channel_mut(ch).hw_state = ChannelHardwareState::Off;
                    }
                }
                let mut tasks = AppTaskBuilder::new();
                match channel {
                    None => {
                        tasks = tasks.extend(self.update_all_hw_states(scpi));
                    }
                    Some(ch) => {
                        tasks = tasks.extend(self.update_hw_state(scpi, ch.to_hal()));
                    }
                }
                ScpiHandleResult {
                    response: ScpiResponse::none(),
                    tasks: tasks.build(),
                }
            }
        }
    }
}

#[cfg(any(test, feature = "test-harness"))]
impl AppCore {
    pub fn force_standby(&mut self) {
        self.hardware_state = HardwareState::Standby;
        self.interface_state.screen = Screen::Main;
        self.interface_state.selected_channel = Some(Channel::A);
    }

    pub fn clear_selected_channel(&mut self) {
        self.interface_state.selected_channel = None;
    }

    pub fn channel_enable(&self, ch: Channel) -> bool {
        self.channel_ref(ch).enable
    }

    pub fn hw_state(&self, ch: Channel) -> ChannelHardwareState {
        self.channel_ref(ch).hw_state
    }

    pub fn set_channel_enable(&mut self, ch: Channel, on: bool) {
        self.channel_mut(ch).enable = on;
    }

    pub fn set_channel_readout(&mut self, ch: Channel, readout: Readout) {
        self.channel_mut(ch).readout = Some(readout);
    }

    pub fn set_converter_flags(&mut self, ch: Channel, flags: ConverterFlags) {
        self.channel_mut(ch).converter_flags = Some(flags);
    }

    pub fn set_temp(&mut self, temp: TemperatureReading) {
        self.last_temp = temp;
    }

    pub fn selected_channel(&self) -> Option<Channel> {
        self.interface_state.selected_channel
    }

    pub fn set_select(&self, ch: Channel) -> SetSelect {
        match ch {
            Channel::A => self.ch_a.set_select,
            Channel::B => self.ch_b.set_select,
        }
    }

    pub fn set_state(&self) -> SetState {
        self.set_state
    }

    pub fn is_setpoint_edit(&self) -> bool {
        matches!(
            self.interface_state.arrows_function,
            ArrowsFunction::SetpointEdit
        )
    }

    pub fn target_voltage(&self, ch: Channel) -> f32 {
        match ch {
            Channel::A => self.ch_a.target.voltage.value(),
            Channel::B => self.ch_b.target.voltage.value(),
        }
    }

    pub fn target_current(&self, ch: Channel) -> f32 {
        match ch {
            Channel::A => self.ch_a.target.current.value(),
            Channel::B => self.ch_b.target.current.value(),
        }
    }

    pub fn last_temp(&self) -> TemperatureReading {
        self.last_temp
    }

    pub fn limit_voltage(&self, ch: Channel) -> f32 {
        match ch {
            Channel::A => self.ch_a.limits.voltage.value(),
            Channel::B => self.ch_b.limits.voltage.value(),
        }
    }

    pub fn limit_current(&self, ch: Channel) -> f32 {
        match ch {
            Channel::A => self.ch_a.limits.current.value(),
            Channel::B => self.ch_b.limits.current.value(),
        }
    }

    pub fn channel_readout(&self, ch: Channel) -> Option<Readout> {
        self.channel_ref(ch).readout
    }

    pub fn set_channel_setpoints(
        &mut self,
        ch: Channel,
        voltage: f32,
        current: f32,
        ovp: f32,
        ocp: f32,
    ) {
        let state = self.channel_mut(ch);
        state.target.voltage.set_value(voltage);
        state.target.current.set_value(current);
        state.limits.voltage.set_value(ovp);
        state.limits.current.set_value(ocp);
    }

    pub fn set_channel_hw_state(&mut self, ch: Channel, hw_state: ChannelHardwareState) {
        self.channel_mut(ch).hw_state = hw_state;
    }

    pub fn reset_to_factory(&mut self, scpi: &mut ScpiState) {
        self.default_reset_channels();
        scpi.remote = true;
        scpi.reset_appearance();
        scpi.set_prot_latched(None, false);
        scpi.clear_error_queue();
    }

    pub fn settings_open(&self) -> bool {
        self.interface_state.settings_open
    }
}

#[cfg(feature = "simulator")]
impl AppCore {
    pub fn set_power_type(&mut self, power_type: PowerType) {
        self.power_type = power_type;
    }

    pub fn power_type(&self) -> PowerType {
        self.power_type
    }

    pub fn set_selected_channel(&mut self, channel: Option<Channel>) {
        self.interface_state.selected_channel = channel;
    }

    pub fn set_settings_open(&mut self, open: bool) {
        self.interface_state.settings_open = open;
    }

    pub fn set_set_state(&mut self, state: SetState) {
        self.set_state = state;
    }

    pub fn set_arrows_function(&mut self, function: ArrowsFunction) {
        self.interface_state.arrows_function = function;
    }

    pub fn set_channel_set_select(&mut self, channel: Channel, select: SetSelect) {
        self.channel_mut(channel).set_select = select;
    }

    pub fn set_edit_precision_exponent(&mut self, exponent: i8) {
        if let Some(field) = self.get_select_precision_mut() {
            field.precision.set_exponent(exponent);
        }
    }

    pub fn main_screen_display_tasks(
        &mut self,
        nav_button: Option<FunctionButton>,
    ) -> heapless::Vec<DisplayTask, 16> {
        let power_type = self.power_type;
        let (ch_a_limit, ch_b_limit) = self.get_current_set();
        let mut tasks = heapless::Vec::new();
        let _ = tasks.push(DisplayTask::SetupMain(power_type, ch_a_limit, ch_b_limit));
        push_builder_tasks(&mut tasks, self.refresh_channels_display());
        if let Some(button) = nav_button {
            let confirm = self.get_confirm_state();
            let _ = tasks.push(DisplayTask::UpdateButton(confirm, Some(button)));
        }
        push_builder_tasks(&mut tasks, self.setpoints_task());
        for channel in [Channel::A, Channel::B] {
            let readout = self.channel_readout(channel).unwrap_or(Readout {
                voltage: 0.0,
                current: 0.0,
                power: 0.0,
            });
            let _ = tasks.push(DisplayTask::UpdateReadout(channel, readout));
        }
        tasks
    }
}

#[cfg(feature = "simulator")]
fn push_builder_tasks(tasks: &mut heapless::Vec<DisplayTask, 16>, builder: AppTaskBuilder) {
    if let Some(batch) = builder.build() {
        for task in batch {
            if let Task::Display(display_task) = task {
                let _ = tasks.push(display_task);
            }
        }
    }
}

#[cfg(test)]
mod dfu_tests;
#[cfg(test)]
mod scpi_response_tests;
#[cfg(test)]
mod scpi_tests;

#[cfg(test)]
mod event_tests;
