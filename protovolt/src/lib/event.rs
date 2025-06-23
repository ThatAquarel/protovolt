use embassy_rp::pio::State;
use embassy_time::Duration;

#[derive(Debug)]
pub enum HardwareEvent {
    PowerOn,

    PowerDeliveryReady(PowerType),
    SenseReady(Result<(), ()>),
    ConverterReady(Result<(), ()>),

    StartMainInterface,

    ReadoutAcquired(Channel, Readout)
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
    pub current: f32
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
        PowerType::Standard(Default::default())
    }
}

pub enum Change {
    Pressed,
    Released
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
    Interface(InterfaceEvent)
}


pub enum HardwareTask {
    // Initialization sequence + self-checks
    EnablePowerDelivery,
    EnableSense,
    EnableConverter,


    // Idle
    EnableReadoutLoop,


    DelayedInterfaceEvent(Duration, InterfaceEvent),
    DelayedHardwareEvent(Duration, HardwareEvent),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Channel {
    A,
    B
}

#[derive(Default)]
pub enum SetState {
    #[default]
    SetLimits,
    SetProtection,
}

#[derive(Clone, Copy)]
pub enum ChannelFocus {
    SelectedActive,
    UnselectedActive,
    SelectedInactive,
    UnselectedInactive,
}

pub enum FunctionButton {
    Enter,
    Switch,
    Settings
}

pub enum DisplayTask {
    // Splash Screen
    SetupSplash,

    ConfirmPowerDelivery(PowerType),
    ConfirmSense(Result<(), ()>),
    ConfirmConverter(Result<(), ()>),

    // Main Readout
    SetupMain(PowerType, Limits, Limits),

    // Updates
    UpdateReadout(Channel, Readout),
    UpdateSetpoint(Channel),
    UpdateChannelFocus(ChannelFocus, ChannelFocus),

    // Navbar
    UpdateButton(Option<FunctionButton>),

    // Settings
    SetupSettings
}

pub struct AppTask {
    pub hardware: Option<HardwareTask>,
    pub display: Option<DisplayTask>,
}
