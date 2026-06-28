#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ConverterFlags {
    pub enabled: bool,
    pub scp: bool,
    pub ocp: bool,
    pub ovp: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TemperatureReading {
    pub ch_a: f32,
    pub ch_b: f32,
    pub mcu: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Readout {
    pub voltage: f32,
    pub current: f32,
    pub power: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub voltage: f32,
    pub current: f32,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            voltage: 5.00,
            current: 0.50,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PowerType {
    PowerDelivery(Limits),
    Standard(Limits),
}

impl Default for PowerType {
    fn default() -> Self {
        PowerType::Standard(Limits::default())
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum ChannelHardwareState {
    #[default]
    Off,
    ConstantVoltage,
    ConstantCurrent,
    ShortCircuit,
    OverTemperature,
    OverCurrent,
    OverVoltage,
}

impl ChannelHardwareState {
    pub fn is_fault(self) -> bool {
        matches!(
            self,
            ChannelHardwareState::ShortCircuit
                | ChannelHardwareState::OverTemperature
                | ChannelHardwareState::OverCurrent
                | ChannelHardwareState::OverVoltage
        )
    }

    pub fn mode_str(self) -> &'static str {
        match self {
            ChannelHardwareState::Off => "OFF",
            ChannelHardwareState::ConstantVoltage => "CV",
            ChannelHardwareState::ConstantCurrent => "CC",
            ChannelHardwareState::ShortCircuit => "SHORT",
            ChannelHardwareState::OverTemperature => "TEMP",
            ChannelHardwareState::OverCurrent => "OCP",
            ChannelHardwareState::OverVoltage => "OVP",
        }
    }
}

pub enum Change {
    Pressed,
    Released,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Channel {
    A,
    B,
}

impl Channel {
    pub fn get_other(self) -> Self {
        match self {
            Channel::A => Channel::B,
            Channel::B => Channel::A,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub enum SetState {
    #[default]
    Set,
    Limits,
}

#[derive(Default, Clone, Copy)]
pub enum SetSelect {
    #[default]
    Voltage,
    Current,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DecimalPrecision {
    pub exponent: i8,
}

impl DecimalPrecision {
    pub fn set_exponent(&mut self, exp: i8) {
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

#[derive(Clone, Copy)]
pub enum ChannelFocus {
    SelectedActive,
    UnselectedActive,
    SelectedInactive,
    UnselectedInactive,
}

#[derive(Clone, Copy)]
pub enum ConfirmState {
    AwaitModify,
    AwaitConfirmModify(Option<Channel>),
}

pub enum FunctionButton {
    Enter,
    Switch,
    Settings,
}

#[derive(Debug)]
pub enum HardwareEvent {
    PowerOn,
    PowerDeliveryReady(PowerType),
    SenseReady(Result<(), ()>),
    ConverterReady(Result<(), ()>),
    StartMainInterface,
    ReadoutAcquired(Channel, Readout),
    TempAcquired(TemperatureReading),
    ConverterStatusAcquired(Channel, ConverterFlags),
    PollConverterStatusInterrupt(Channel),
}

pub enum InterfaceEvent {
    ButtonUp,
    ButtonDown,
    ButtonLeft,
    ButtonRight,
    ButtonEnter(Change),
    ButtonSwitch(Change),
    ButtonSettings(Change),
    ButtonChannel(Channel),
}

pub enum AppEvent {
    Hardware(HardwareEvent),
    Interface(InterfaceEvent),
}

pub enum HardwareTask {
    EnablePowerDelivery,
    EnableSense,
    EnableConverter,
    EnableReadoutLoop,
    PollConverterStatus(Channel),
    UpdateConverterState(Channel, bool),
    UpdateConverterVoltage(Channel, f32),
    UpdateConverterCurrent(Channel, f32),
    /// Delay in milliseconds before dispatching the nested event.
    DelayedHardwareEvent(u64, HardwareEvent),
}

pub enum DisplayTask {
    SetupSplash,
    ConfirmPowerDelivery(PowerType),
    ConfirmSense(Result<(), ()>),
    ConfirmConverter(Result<(), ()>),
    SetupMain(PowerType, Limits, Limits),
    UpdatePowerInfo(PowerType),
    UpdateReadout(Channel, Readout),
    UpdateSetpoint(
        Channel,
        Limits,
        Option<SetSelect>,
        ConfirmState,
        Option<DecimalPrecision>,
    ),
    UpdateChannelFocus(
        ChannelFocus,
        ChannelFocus,
        ChannelHardwareState,
        ChannelHardwareState,
    ),
    UpdateSetState(Channel, SetState, Option<SetSelect>, ConfirmState),
    UpdateChannelHardwareState(Channel, ChannelFocus, ChannelHardwareState),
    UpdateButton(ConfirmState, Option<FunctionButton>),
}
