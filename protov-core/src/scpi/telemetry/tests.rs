use super::*;
use crate::scpi::ScpiContext;
use crate::scpi::parser::TempSlot;

#[test]
fn format_telemetry_shape() {
    let ctx = ScpiContext {
        temp_ch_a: 25.0,
        temp_ch_b: 26.0,
        temp_mcu: 27.0,
        input_type_pd: true,
        input_voltage: 20.0,
        input_current: 0.5,
        sense_ok: true,
        converter_ok: false,
        prot_latched_a: false,
        prot_latched_b: false,
        ..Default::default()
    };
    let mut buf = heapless::String::<512>::new();
    format_telemetry(&ctx, &mut buf);
    assert!(buf.as_str().contains("PD"));
    assert!(buf.as_str().contains("20.000"));
}

#[test]
fn format_temp_slots() {
    let ctx = ScpiContext::default();
    let mut buf = heapless::String::<512>::new();
    format_temp(&ctx, TempSlot::Mcu, &mut buf);
    assert!(!buf.is_empty());
}

#[test]
fn format_diag_output() {
    let ctx = ScpiContext {
        sense_ok: true,
        converter_ok: true,
        ..Default::default()
    };
    let mut buf = heapless::String::<512>::new();
    super::format_diag(&ctx, &mut buf);
    assert_eq!(buf.as_str(), "1,1");
}

#[test]
fn format_inp_std() {
    let ctx = ScpiContext {
        input_type_pd: false,
        input_voltage: 5.0,
        input_current: 1.0,
        ..Default::default()
    };
    let mut buf = heapless::String::<512>::new();
    format_inp(&ctx, &mut buf);
    assert!(buf.as_str().starts_with("STD,"));
}
