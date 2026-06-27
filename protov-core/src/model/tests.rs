use super::*;

#[test]
fn mode_str_all_variants() {
    assert_eq!(ChannelHardwareState::Off.mode_str(), "OFF");
    assert_eq!(ChannelHardwareState::ConstantVoltage.mode_str(), "CV");
    assert_eq!(ChannelHardwareState::ConstantCurrent.mode_str(), "CC");
    assert_eq!(ChannelHardwareState::ShortCircuit.mode_str(), "SHORT");
    assert_eq!(ChannelHardwareState::OverTemperature.mode_str(), "TEMP");
    assert_eq!(ChannelHardwareState::OverCurrent.mode_str(), "OCP");
    assert_eq!(ChannelHardwareState::OverVoltage.mode_str(), "OVP");
}

#[test]
fn fault_detection() {
    assert!(!ChannelHardwareState::ConstantVoltage.is_fault());
    assert!(ChannelHardwareState::OverVoltage.is_fault());
    assert!(ChannelHardwareState::ShortCircuit.is_fault());
}

#[test]
fn channel_get_other() {
    assert_eq!(Channel::A.get_other(), Channel::B);
    assert_eq!(Channel::B.get_other(), Channel::A);
}
