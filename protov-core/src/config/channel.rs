//! Per-channel factory setpoints and protection limits.

#[derive(Clone, Copy, Debug)]
pub struct ChannelProfile {
    pub voltage_set: f32,
    pub current_set: f32,
    pub ovp: f32,
    pub ocp: f32,
}

pub const VOLTAGE_EDIT_RANGE: (f32, f32) = (0.2, 20.0);
pub const CURRENT_EDIT_RANGE: (f32, f32) = (0.0, 5.0);

pub const CH1_FACTORY: ChannelProfile = ChannelProfile {
    voltage_set: 5.0,
    current_set: 1.0,
    ovp: 20.0,
    ocp: 5.0,
};

pub const CH2_FACTORY: ChannelProfile = ChannelProfile {
    voltage_set: 3.3,
    current_set: 1.0,
    ovp: 20.0,
    ocp: 5.0,
};
