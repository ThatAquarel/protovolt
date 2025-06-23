use core::fmt::Write;
use heapless::String;

pub fn format_f32<const N: usize>(value: f32, decimals: u32) -> String<N> {
    let mut buf = String::<N>::new();

    let is_negative = value.is_sign_negative();
    let abs_value = if is_negative { -value } else { value };

    let int_part = abs_value as u32;
    let mut frac_part = abs_value - (int_part as f32);

    // Scale fractional part manually (no powi)
    let mut scale = 1.0;
    for _ in 0..decimals {
        scale *= 10.0;
    }

    frac_part = frac_part * scale;
    let frac_part = frac_part as u32;

    if is_negative {
        let _ = buf.write_char('-');
    }

    if int_part < 10 {
        let _ = buf.write_char('0');
    }

    let _ = write!(buf, "{}", int_part);
    if decimals > 0 {
        let _ = buf.write_char('.');
        let _ = write!(buf, "{:0width$}", frac_part, width = decimals as usize);
    }

    buf
}
