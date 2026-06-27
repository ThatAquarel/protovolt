use core::fmt::Write;

use crate::scpi::parser::TempSlot;
use crate::scpi::{RESPONSE_BUF, ScpiContext};

pub fn format_telemetry(ctx: &ScpiContext, buf: &mut heapless::String<RESPONSE_BUF>) {
    let inp_type = if ctx.input_type_pd { "PD" } else { "STD" };
    let sense = if ctx.sense_ok { "1" } else { "0" };
    let converter = if ctx.converter_ok { "1" } else { "0" };
    let _ = write!(
        buf,
        "{},{},{},{},{:.3},{:.3},{},{}",
        fmt3(ctx.temp_ch_a),
        fmt3(ctx.temp_ch_b),
        fmt3(ctx.temp_mcu),
        inp_type,
        ctx.input_voltage,
        ctx.input_current,
        sense,
        converter
    );
}

pub fn format_temp(ctx: &ScpiContext, slot: TempSlot, buf: &mut heapless::String<RESPONSE_BUF>) {
    let value = match slot {
        TempSlot::Cha => ctx.temp_ch_a,
        TempSlot::Chb => ctx.temp_ch_b,
        TempSlot::Mcu => ctx.temp_mcu,
    };
    let _ = write!(buf, "{}", fmt3(value));
}

pub fn format_inp(ctx: &ScpiContext, buf: &mut heapless::String<RESPONSE_BUF>) {
    let inp_type = if ctx.input_type_pd { "PD" } else { "STD" };
    let _ = write!(
        buf,
        "{},{:.3},{:.3}",
        inp_type, ctx.input_voltage, ctx.input_current
    );
}

pub fn format_diag(ctx: &ScpiContext, buf: &mut heapless::String<RESPONSE_BUF>) {
    let sense = if ctx.sense_ok { "1" } else { "0" };
    let converter = if ctx.converter_ok { "1" } else { "0" };
    let _ = write!(buf, "{},{}", sense, converter);
}

fn fmt3(value: f32) -> heapless::String<16> {
    crate::fmt::format_f32::<16>(value, 3)
}

#[cfg(test)]
mod tests;
