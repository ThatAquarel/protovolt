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

#[test]
fn readout_display_blocked_while_settings_open() {
    let readout = Readout {
        voltage: 1.0,
        current: 1.0,
        power: 1.0,
    };
    assert!(DisplayTask::UpdateReadout(Channel::A, readout).blocked_while_settings_open());
    assert!(DisplayTask::UpdateSetpoint(
        Channel::A,
        Limits {
            voltage: 1.0,
            current: 1.0,
        },
        None,
        ConfirmState::AwaitModify,
        None,
    )
    .blocked_while_settings_open());
    assert!(DisplayTask::UpdateSetState(
        Channel::A,
        SetState::Set,
        None,
        ConfirmState::AwaitModify,
    )
    .blocked_while_settings_open());
    assert!(!DisplayTask::UpdateSettings(true).blocked_while_settings_open());
    assert!(DisplayTask::UpdateChannelUnits(Channel::A).blocked_while_settings_open());
}

#[test]
fn interface_events_blocked_while_settings_open() {
    assert!(!InterfaceEvent::ButtonSettings(Change::Pressed).blocked_while_settings_open());
    assert!(InterfaceEvent::ButtonSwitch(Change::Pressed).blocked_while_settings_open());
    assert!(InterfaceEvent::ButtonEnter(Change::Pressed).blocked_while_settings_open());
    assert!(InterfaceEvent::ButtonUp.blocked_while_settings_open());
    assert!(InterfaceEvent::ButtonChannel(Channel::A).blocked_while_settings_open());
}
