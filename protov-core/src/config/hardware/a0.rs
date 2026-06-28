use super::HardwareProfile;

/// Early / pre-calibration boards
pub const PROFILE: HardwareProfile = HardwareProfile {
    revision: "A.0",
    converter_voltage_offset_mv: 0,
    converter_current_scale_ma: 38.299_625,
    ina226_r_shunt_ohm: 0.010,
    ina226_i_max_a: 5.0,
};
