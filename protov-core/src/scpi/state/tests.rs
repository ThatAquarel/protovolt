use super::*;
use crate::config::FACTORY;
use crate::scpi::ScpiChannel;

#[test]
fn error_queue_fifo() {
    let mut state = ScpiState::default();
    state.push_error(-100, "first");
    state.push_error(-200, "second");
    let (c1, m1) = state.pop_error();
    assert_eq!(c1, -100);
    assert_eq!(m1.as_str(), "first");
    let (c2, _) = state.pop_error();
    assert_eq!(c2, -200);
}

#[test]
fn empty_error_queue() {
    let mut state = ScpiState::default();
    let (code, msg) = state.pop_error();
    assert_eq!(code, 0);
    assert_eq!(msg.as_str(), "No error");
}

#[test]
fn save_recall_delete_slots() {
    let mut state = ScpiState::default();
    let snap1 = ChannelSnapshot {
        voltage_set: 1.0,
        ..Default::default()
    };
    let snap2 = ChannelSnapshot {
        voltage_set: 2.0,
        ..Default::default()
    };
    state.save_slot(1, snap1, snap2);
    let recalled = state.recall_slot(1).unwrap();
    assert_eq!(recalled.0.voltage_set, 1.0);
    assert_eq!(recalled.1.voltage_set, 2.0);
    state.delete_slot(1);
    assert!(state.recall_slot(1).is_none());
}

#[test]
fn prot_latched_per_channel() {
    let mut state = ScpiState::default();
    state.set_prot_latched(Some(ScpiChannel::Ch1), true);
    assert!(state.prot_latched(ScpiChannel::Ch1));
    assert!(!state.prot_latched(ScpiChannel::Ch2));
    state.set_prot_latched(None, true);
    assert!(state.prot_latched(ScpiChannel::Ch1));
    assert!(state.prot_latched(ScpiChannel::Ch2));
}

#[test]
fn reset_appearance() {
    let mut state = ScpiState::default();
    state.set_color(ScpiChannel::Ch1, Rgb { r: 1, g: 2, b: 3 });
    state.lcd_brightness = 10;
    state.reset_appearance();
    assert_eq!(state.color(ScpiChannel::Ch1), FACTORY.appearance.ch1);
    assert_eq!(state.lcd_brightness, FACTORY.appearance.lcd_brightness);
}

#[test]
fn color_for_hal_mapping() {
    let mut state = ScpiState::default();
    let rgb = Rgb {
        r: 10,
        g: 20,
        b: 30,
    };
    state.set_color(ScpiChannel::Ch2, rgb);
    assert_eq!(state.color_for_hal(crate::model::Channel::B), rgb);
}
