use crate::config::{HARDWARE_PROFILE, hardware_revision};

#[test]
fn hardware_revision_matches_profile() {
    assert_eq!(hardware_revision(), HARDWARE_PROFILE.revision);
}

#[cfg(feature = "hw-a1")]
#[test]
fn a1_converter_trim() {
    assert_eq!(HARDWARE_PROFILE.converter_voltage_offset_mv, 0);
    assert!((HARDWARE_PROFILE.converter_current_scale_ma - 41.15).abs() < f32::EPSILON);
}

#[test]
fn ina226_calibration_is_nonzero() {
    let cal = HARDWARE_PROFILE.ina226_cal_reg();
    assert_ne!(cal, [0, 0]);
}

#[cfg(feature = "hw-a2")]
#[test]
fn a2_converter_trim() {
    assert_eq!(HARDWARE_PROFILE.converter_voltage_offset_mv, 0);
    assert!((HARDWARE_PROFILE.converter_current_scale_ma - 50.0).abs() < f32::EPSILON);
}

#[cfg(feature = "hw-a0")]
#[test]
fn a0_uses_proto_current_scale() {
    assert!((HARDWARE_PROFILE.converter_current_scale_ma - 38.299_625).abs() < f32::EPSILON);
}
