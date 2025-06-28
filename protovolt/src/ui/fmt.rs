use core::fmt::Write;
use heapless::String;
use micromath::F32Ext;

pub fn format_f32<const N: usize>(value: f32, decimals: u32) -> String<N> {
    let mut buf = String::<N>::new();

    let scale = 10f32.powi(decimals as i32);
    let rounded = (value * scale).round();
    let int_part = (rounded / scale) as u32;
    let frac_part = (rounded as u32) % (scale as u32);

    let _ = write!(buf, "{}", int_part);
    if decimals > 0 {
        let _ = write!(buf, ".{:0width$}", frac_part, width = decimals as usize);
    }

    buf
}
