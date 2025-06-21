pub enum HardwareEvent {
    PowerDeliveryReady,
    SenseReady,
    ConverterReady,    
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
}

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

    UpdateReadout(Channel),
    UpdateSetpoint(Channel),

    // Settings
    SetupSettings
}

pub struct AppTask {
    pub hardware: Option<HardwareTask>,
    pub display: Option<DisplayTask>,
}
