use super::HardwareProfile;

/// Production ProtoV MINI A.2
pub const PROFILE: HardwareProfile = HardwareProfile {
    revision: "A.2",
    converter_voltage_offset_mv: 0,
    converter_current_scale_ma: 50.00,
    ina226_r_shunt_ohm: 0.010,
    ina226_i_max_a: 5.0,
};
