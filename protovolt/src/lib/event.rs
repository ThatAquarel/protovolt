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
    ButtonChannelA,
    ButtonChannelB,
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

#[derive(Debug)]
pub enum Channel {
    A,
    B
}

pub enum DisplayTask {
    // Splash Screen
    SetupSplash,

    ConfirmPowerDelivery,
    ConfirmSense,
    ConfirmConverter,

    // Main Readout
    SetupMain,

    UpdateReadout(Channel, Readout),
    UpdateSetpoint(Channel),

    // Settings
    SetupSettings
}

pub struct AppTask {
    pub hardware: Option<HardwareTask>,
    pub display: Option<DisplayTask>,
}
