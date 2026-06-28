use super::*;

#[test]
fn factory_profiles_ordered() {
    assert!(FACTORY.ch1.voltage_set >= FACTORY.ch2.voltage_set);
    assert_eq!(FACTORY.ch1.ocp, CH1_FACTORY.ocp);
    assert_eq!(FACTORY.ch2.voltage_set, CH2_FACTORY.voltage_set);
}

#[test]
fn format_idn_contains_product_fields() {
    let mut buf = heapless::String::<128>::new();
    format_idn(&mut buf);
    let s = buf.as_str();
    assert!(s.contains(MANUFACTURER));
    assert!(s.contains(PRODUCT_NAME));
    assert!(s.contains(SERIAL_NUMBER));
    assert!(s.contains(HARDWARE_REVISION));
}

#[test]
fn edit_ranges_valid() {
    assert!(VOLTAGE_EDIT_RANGE.0 < VOLTAGE_EDIT_RANGE.1);
    assert!(CURRENT_EDIT_RANGE.0 <= CURRENT_EDIT_RANGE.1);
}

#[test]
fn otp_thresholds() {
    assert_eq!(MCU_OTP_C, 50.0);
    assert_eq!(CH_OTP_C, 55.0);
}
