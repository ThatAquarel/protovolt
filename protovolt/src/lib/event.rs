pub enum HardwareEvent {
    ButtonUp,
    ButtonDown,
    ButtonLeft,
    ButtonRight,
    ButtonEnter,
    ButtonSwitch,
    ButtonSettings,
    ButtonChannelA,
    ButtonChannelB,

    InterruptSense,
    InterruptConverter,
}

pub enum DisplayEvent {
    UpdateReadoutA,
    UpdateSetpointA,
    UpdateReadoutB,
    UpdateSetpointB,
}
