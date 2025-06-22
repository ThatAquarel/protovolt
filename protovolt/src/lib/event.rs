use embassy_time::Duration;

#[derive(Debug)]
pub enum HardwareEvent {
    PowerOn,

    PowerDeliveryReady,
    SenseReady,
    ConverterReady,

    StartMainInterface,

    ReadoutAcquired(Channel, Readout)
}

#[derive(Clone, Copy, Debug)]
pub struct Readout {
    pub voltage: f32,
    pub current: f32,
    pub power: f32,
}

pub enum InterfaceEvent {
    ButtonUp,
    ButtonDown,
    ButtonLeft,
    ButtonRight,
    ButtonEnter,
    ButtonSwitch,
    ButtonSettings,
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


    DelayedHardwareEvent(Duration, HardwareEvent)
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Channel {
    A,
    B
}

#[derive(Clone, Copy)]
pub enum ChannelFocus {
    SelectedActive,
    UnselectedActive,
    SelectedInactive,
    UnselectedInactive,
}

pub enum DisplayTask {
    // Splash Screen
    SetupSplash,

    ConfirmPowerDelivery,
    ConfirmSense,
    ConfirmConverter,

    // Main Readout
    SetupMain,

    // Updates
    UpdateReadout(Channel, Readout),
    UpdateSetpoint(Channel),

    // Channel
    UpdateChannelFocus(ChannelFocus, ChannelFocus),

    // Settings
    SetupSettings
}

pub struct AppTask {
    pub hardware: Option<HardwareTask>,
    pub display: Option<DisplayTask>,
}
