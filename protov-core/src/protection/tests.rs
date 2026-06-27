use super::*;
use crate::model::ChannelHardwareState;

fn limits(v: f32, i: f32) -> Limits {
    Limits {
        voltage: v,
        current: i,
    }
}

fn readout(v: f32, i: f32) -> Readout {
    Readout {
        voltage: v,
        current: i,
        power: v * i,
    }
}

fn temps(mcu: f32, ch_a: f32, ch_b: f32) -> TemperatureReading {
    TemperatureReading { mcu, ch_a, ch_b }
}

fn flags(enabled: bool, scp: bool, ocp: bool, ovp: bool) -> ConverterFlags {
    ConverterFlags {
        enabled,
        scp,
        ocp,
        ovp,
    }
}

#[test]
fn latch_sticks_fault_state() {
    let state = derive_hw_state(
        Channel::A,
        true,
        None,
        None,
        limits(20.0, 5.0),
        &temps(25.0, 25.0, 25.0),
        true,
        ChannelHardwareState::OverCurrent,
    );
    assert_eq!(state, ChannelHardwareState::OverCurrent);
}

#[test]
fn mcu_otp_trips() {
    let state = derive_hw_state(
        Channel::A,
        true,
        Some(flags(true, false, false, false)),
        None,
        limits(20.0, 5.0),
        &temps(50.0, 25.0, 25.0),
        false,
        ChannelHardwareState::Off,
    );
    assert_eq!(state, ChannelHardwareState::OverTemperature);
}

#[test]
fn mcu_otp_boundary_below() {
    let state = derive_hw_state(
        Channel::B,
        true,
        Some(flags(true, false, false, false)),
        None,
        limits(20.0, 5.0),
        &temps(49.9, 25.0, 25.0),
        false,
        ChannelHardwareState::Off,
    );
    assert_ne!(state, ChannelHardwareState::OverTemperature);
}

#[test]
fn channel_otp_trips() {
    let state = derive_hw_state(
        Channel::A,
        true,
        Some(flags(true, false, false, false)),
        None,
        limits(20.0, 5.0),
        &temps(25.0, 55.0, 25.0),
        false,
        ChannelHardwareState::Off,
    );
    assert_eq!(state, ChannelHardwareState::OverTemperature);
}

#[test]
fn channel_b_otp() {
    let state = derive_hw_state(
        Channel::B,
        true,
        Some(flags(true, false, false, false)),
        None,
        limits(20.0, 5.0),
        &temps(25.0, 25.0, 55.0),
        false,
        ChannelHardwareState::Off,
    );
    assert_eq!(state, ChannelHardwareState::OverTemperature);
}

#[test]
fn software_ocp() {
    let state = derive_hw_state(
        Channel::A,
        true,
        Some(flags(true, false, false, false)),
        Some(readout(5.0, 5.1)),
        limits(20.0, 5.0),
        &temps(25.0, 25.0, 25.0),
        false,
        ChannelHardwareState::Off,
    );
    assert_eq!(state, ChannelHardwareState::OverCurrent);
}

#[test]
fn software_ovp() {
    let state = derive_hw_state(
        Channel::A,
        true,
        Some(flags(true, false, false, false)),
        Some(readout(20.1, 1.0)),
        limits(20.0, 5.0),
        &temps(25.0, 25.0, 25.0),
        false,
        ChannelHardwareState::Off,
    );
    assert_eq!(state, ChannelHardwareState::OverVoltage);
}

#[test]
fn converter_ovp_flag() {
    let state = derive_hw_state(
        Channel::A,
        true,
        Some(flags(true, false, false, true)),
        None,
        limits(20.0, 5.0),
        &temps(25.0, 25.0, 25.0),
        false,
        ChannelHardwareState::Off,
    );
    assert_eq!(state, ChannelHardwareState::OverVoltage);
}

#[test]
fn converter_scp_flag() {
    let state = derive_hw_state(
        Channel::A,
        true,
        Some(flags(true, true, false, false)),
        None,
        limits(20.0, 5.0),
        &temps(25.0, 25.0, 25.0),
        false,
        ChannelHardwareState::Off,
    );
    assert_eq!(state, ChannelHardwareState::ShortCircuit);
}

#[test]
fn disabled_is_off() {
    let state = derive_hw_state(
        Channel::A,
        false,
        Some(flags(true, false, false, false)),
        Some(readout(5.0, 1.0)),
        limits(20.0, 5.0),
        &temps(25.0, 25.0, 25.0),
        false,
        ChannelHardwareState::Off,
    );
    assert_eq!(state, ChannelHardwareState::Off);
}

#[test]
fn enabled_ocp_flag_is_cc() {
    let state = derive_hw_state(
        Channel::A,
        true,
        Some(flags(true, false, true, false)),
        None,
        limits(20.0, 5.0),
        &temps(25.0, 25.0, 25.0),
        false,
        ChannelHardwareState::Off,
    );
    assert_eq!(state, ChannelHardwareState::ConstantCurrent);
}

#[test]
fn enabled_no_fault_is_cv() {
    let state = derive_hw_state(
        Channel::A,
        true,
        Some(flags(true, false, false, false)),
        None,
        limits(20.0, 5.0),
        &temps(25.0, 25.0, 25.0),
        false,
        ChannelHardwareState::Off,
    );
    assert_eq!(state, ChannelHardwareState::ConstantVoltage);
}

#[test]
fn temp_priority_over_readout() {
    let state = derive_hw_state(
        Channel::A,
        true,
        Some(flags(true, false, false, false)),
        Some(readout(20.1, 5.1)),
        limits(20.0, 5.0),
        &temps(50.0, 25.0, 25.0),
        false,
        ChannelHardwareState::Off,
    );
    assert_eq!(state, ChannelHardwareState::OverTemperature);
}

#[test]
fn readout_priority_over_converter_flags() {
    let state = derive_hw_state(
        Channel::A,
        true,
        Some(flags(true, true, true, true)),
        Some(readout(20.1, 1.0)),
        limits(20.0, 5.0),
        &temps(25.0, 25.0, 25.0),
        false,
        ChannelHardwareState::Off,
    );
    assert_eq!(state, ChannelHardwareState::OverVoltage);
}

#[test]
fn no_flags_enabled_still_cv() {
    let state = derive_hw_state(
        Channel::A,
        true,
        None,
        None,
        limits(20.0, 5.0),
        &temps(25.0, 25.0, 25.0),
        false,
        ChannelHardwareState::Off,
    );
    assert_eq!(state, ChannelHardwareState::ConstantVoltage);
}

#[test]
fn mcu_overtemp_helper() {
    assert!(mcu_overtemp(&temps(50.0, 0.0, 0.0)));
    assert!(!mcu_overtemp(&temps(49.9, 0.0, 0.0)));
}
