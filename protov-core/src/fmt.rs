use core::fmt::Write;
use heapless::String;

#[allow(unused_imports)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_f32_zero_decimals() {
        assert_eq!(format_f32::<8>(3.7, 0).as_str(), "4");
    }

    #[test]
    fn format_f32_three_decimals() {
        assert_eq!(format_f32::<16>(1.2345, 3).as_str(), "1.235");
    }

    #[test]
    fn format_f32_rounds_half_up() {
        assert_eq!(format_f32::<16>(1.25, 1).as_str(), "1.3");
    }
}
