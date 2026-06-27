use crate::model::{
    AppEvent, Channel, ChannelHardwareState, ConverterFlags, HardwareEvent, Readout,
    TemperatureReading,
};
use crate::scpi::ScpiChannel;
use crate::scpi::state::ScpiState;

use super::AppCore;

fn standby_app() -> (AppCore, ScpiState) {
    let mut app = AppCore::default();
    app.force_standby();
    (app, ScpiState::default())
}

#[test]
fn readout_ocp_trips_channel() {
    let (mut app, mut scpi) = standby_app();
    app.set_channel_enable(Channel::A, true);
    app.set_converter_flags(
        Channel::A,
        ConverterFlags {
            enabled: true,
            scp: false,
            ocp: false,
            ovp: false,
        },
    );
    let readout = Readout {
        voltage: 5.0,
        current: 6.0,
        power: 30.0,
    };
    let _ = app.handle_event(
        AppEvent::Hardware(HardwareEvent::ReadoutAcquired(Channel::A, readout)),
        &mut scpi,
    );
    assert!(!app.channel_enable(Channel::A));
    assert_eq!(app.hw_state(Channel::A), ChannelHardwareState::OverCurrent);
    assert!(scpi.prot_latched(ScpiChannel::Ch1));
}

#[test]
fn mcu_temp_trips_both_channels() {
    let (mut app, mut scpi) = standby_app();
    app.set_channel_enable(Channel::A, true);
    app.set_channel_enable(Channel::B, true);
    let temp = TemperatureReading {
        mcu: 50.0,
        ch_a: 25.0,
        ch_b: 25.0,
    };
    let _ = app.handle_event(
        AppEvent::Hardware(HardwareEvent::TempAcquired(temp)),
        &mut scpi,
    );
    assert!(!app.channel_enable(Channel::A));
    assert!(!app.channel_enable(Channel::B));
    assert_eq!(
        app.hw_state(Channel::A),
        ChannelHardwareState::OverTemperature
    );
    assert!(scpi.prot_latched(ScpiChannel::Ch1));
    assert!(scpi.prot_latched(ScpiChannel::Ch2));
}

#[test]
fn converter_cc_flag() {
    let (mut app, mut scpi) = standby_app();
    app.set_channel_enable(Channel::B, true);
    let flags = ConverterFlags {
        enabled: true,
        scp: false,
        ocp: true,
        ovp: false,
    };
    let _ = app.handle_event(
        AppEvent::Hardware(HardwareEvent::ConverterStatusAcquired(Channel::B, flags)),
        &mut scpi,
    );
    assert_eq!(
        app.hw_state(Channel::B),
        ChannelHardwareState::ConstantCurrent
    );
}

#[test]
fn reset_prot_re_derives_state() {
    let (mut app, mut scpi) = standby_app();
    app.set_channel_enable(Channel::A, true);
    app.set_converter_flags(
        Channel::A,
        ConverterFlags {
            enabled: true,
            scp: false,
            ocp: true,
            ovp: false,
        },
    );
    let _ = app.update_hw_state(&mut scpi, Channel::A);
    scpi.set_prot_latched(Some(ScpiChannel::Ch1), true);
    app.set_channel_enable(Channel::A, true);
    let cmd = crate::scpi::parser::parse_command("OUTP:RESET:PROT CH1").unwrap();
    let ctx = crate::scpi::ScpiContext::default();
    let _ = app.handle_scpi(cmd, &mut scpi, &ctx);
    assert!(!scpi.prot_latched(ScpiChannel::Ch1));
    assert_eq!(
        app.hw_state(Channel::A),
        ChannelHardwareState::ConstantCurrent
    );
}
