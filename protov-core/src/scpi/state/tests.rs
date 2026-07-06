use super::*;
use crate::config::FACTORY;
use crate::model::Channel;
use crate::scpi::ScpiChannel;
use embedded_graphics::pixelcolor::RgbColor;

#[test]
fn default_state_matches_factory_appearance() {
    let state = ScpiState::default();
    assert!(state.remote);
    assert_eq!(state.color(ScpiChannel::Ch1), FACTORY.appearance.ch1);
    assert_eq!(state.color(ScpiChannel::Ch2), FACTORY.appearance.ch2);
    assert_eq!(state.lcd_brightness, FACTORY.appearance.lcd_brightness);
    assert_eq!(state.led_brightness, FACTORY.appearance.led_brightness);
    assert!(!state.prot_latched(ScpiChannel::Ch1));
    assert!(!state.prot_latched(ScpiChannel::Ch2));
}

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
fn error_queue_drops_when_full() {
    let mut state = ScpiState::default();
    for i in 0..ERR_QUEUE_SIZE {
        state.push_error(-100 - i as i32, "queued");
    }
    state.push_error(-999, "overflow");
    for i in 0..ERR_QUEUE_SIZE {
        let (code, _) = state.pop_error();
        assert_eq!(code, -100 - i as i32);
    }
    let (code, msg) = state.pop_error();
    assert_eq!(code, 0);
    assert_eq!(msg.as_str(), "No error");
}

#[test]
fn clear_and_seed_error_helpers() {
    let mut state = ScpiState::default();
    state.seed_error(-50, "seeded");
    let (code, msg) = state.pop_error();
    assert_eq!(code, -50);
    assert_eq!(msg.as_str(), "seeded");
    state.clear_error_queue();
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
fn save_recall_ignore_invalid_slots() {
    let mut state = ScpiState::default();
    let snap = ChannelSnapshot::default();
    state.save_slot(10, snap, snap);
    assert!(state.recall_slot(10).is_none());
    state.delete_slot(10);
}

#[test]
fn recall_unused_slot_returns_none() {
    let state = ScpiState::default();
    assert!(state.recall_slot(5).is_none());
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
fn set_prot_latched_ch2_only() {
    let mut state = ScpiState::default();
    state.set_prot_latched(Some(ScpiChannel::Ch2), true);
    assert!(!state.prot_latched(ScpiChannel::Ch1));
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

#[test]
fn color_roundtrip_per_scpi_channel() {
    let mut state = ScpiState::default();
    let ch1 = Rgb {
        r: 1,
        g: 2,
        b: 3,
    };
    let ch2 = Rgb {
        r: 4,
        g: 5,
        b: 6,
    };
    state.set_color(ScpiChannel::Ch1, ch1);
    state.set_color(ScpiChannel::Ch2, ch2);
    assert_eq!(state.color(ScpiChannel::Ch1), ch1);
    assert_eq!(state.color(ScpiChannel::Ch2), ch2);
    assert_eq!(state.color_for_hal(Channel::A), ch1);
    assert_eq!(state.color_for_hal(Channel::B), ch2);
}

#[test]
fn rgb565_and_led_outputs() {
    let mut state = ScpiState::default();
    let rgb = Rgb {
        r: 255,
        g: 128,
        b: 64,
    };
    state.set_color(ScpiChannel::Ch1, rgb);
    state.led_brightness = 128;

    let selected = state.selected_rgb565(Channel::A);
    let unselected = state.unselected_rgb565(Channel::A);
    assert!(selected.r() > unselected.r());

    let channel_led = state.channel_led(Channel::A);
    assert!(channel_led.r > 0 && channel_led.r < 255);

    let white = state.white_led();
    assert_eq!(white.r, white.g);
    assert_eq!(white.g, white.b);
    assert!(white.r > 0);
}
