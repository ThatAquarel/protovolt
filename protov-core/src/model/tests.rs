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
    assert!(
        DisplayTask::UpdateSetpoint(
            Channel::A,
            Limits {
                voltage: 1.0,
                current: 1.0,
            },
            None,
            ConfirmState::AwaitModify,
            None,
        )
        .blocked_while_settings_open()
    );
    assert!(
        DisplayTask::UpdateSetState(Channel::A, SetState::Set, None, ConfirmState::AwaitModify,)
            .blocked_while_settings_open()
    );
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

#[test]
fn decimal_precision_clamps_exponent() {
    let mut p = DecimalPrecision::default();
    p.set_exponent(5);
    assert_eq!(p.get_exponent(), 1);
    p.set_exponent(-5);
    assert_eq!(p.get_exponent(), -2);
    p.set_exponent(0);
    assert_eq!(p.get_exponent(), 0);
}

#[test]
fn decimal_precision_cursor_moves_within_bounds() {
    let mut p = DecimalPrecision { exponent: 0 };
    p.cursor_right();
    assert_eq!(p.get_exponent(), -1);
    p.cursor_left();
    assert_eq!(p.get_exponent(), 0);
    p.cursor_left();
    assert_eq!(p.get_exponent(), 1);
    p.cursor_right();
    p.cursor_right();
    assert_eq!(p.get_exponent(), -1);
    p.cursor_right();
    assert_eq!(p.get_exponent(), -2);
    p.cursor_right();
    assert_eq!(p.get_exponent(), -2);
}

#[test]
fn app_task_builder_collects_and_iterates() {
    let built = AppTaskBuilder::new()
        .hardware(HardwareTask::UpdateConverterState(Channel::A, true))
        .display(DisplayTask::UpdateSettings(true))
        .build()
        .unwrap();
    assert_eq!(built.count, 2);
    let kinds: heapless::Vec<&'static str, 4> = built
        .into_iter()
        .map(|t| match t {
            Task::Hardware(_) => "hw",
            Task::Display(_) => "disp",
        })
        .collect();
    assert_eq!(kinds.as_slice(), &["hw", "disp"]);
}

#[test]
fn app_task_builder_extend_merges_tasks() {
    let a = AppTaskBuilder::new().hardware(HardwareTask::EnableSense);
    let b = AppTaskBuilder::new().display(DisplayTask::UpdateSettings(false));
    let merged = a.extend(b).build().unwrap();
    assert_eq!(merged.count, 2);
}

#[test]
fn app_task_builder_caps_at_limit() {
    let mut builder = AppTaskBuilder::default();
    for _ in 0..15 {
        builder = builder.hardware(HardwareTask::PollConverterStatus(Channel::A));
    }
    let built = builder.build().unwrap();
    assert_eq!(built.count, 12);
}

#[test]
fn app_task_display_task_helper() {
    let built =
        AppTaskBuilder::display_task(DisplayTask::DfuStatus(DfuStatus::Idle)).unwrap();
    assert_eq!(built.count, 1);
    let only = built.into_iter().next().unwrap();
    assert!(matches!(only, Task::Display(DisplayTask::DfuStatus(DfuStatus::Idle))));
}
