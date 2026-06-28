use core::fmt::Write;

use heapless::String;

use crate::model::{Limits, PowerType};

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

fn append_navbar_power_suffix(line: &mut String<16>, power_w: f32) {
    if power_w >= 100.0 {
        let _ = write!(line, "{:>3}. W", power_w as u32);
    } else if power_w >= 10.0 {
        let _ = write!(line, "{:>4.1} W", power_w);
    } else {
        let _ = write!(line, "{:>4.2} W", power_w);
    }
}

/// Navbar line 1: source label + fixed-width power (14 chars).
///
/// `"LINKED  100. W"`, `"USB PD  100. W"`, `"USB2.0  2.50 W"`, etc.
pub fn format_navbar_status_line(power_type: PowerType, serial_connected: bool) -> String<16> {
    let limits = match power_type {
        PowerType::PowerDelivery(limits) | PowerType::Standard(limits) => limits,
    };
    let power_w = limits.voltage * limits.current;

    let mut line = String::<16>::new();
    if serial_connected {
        let _ = line.push_str("LINKED  ");
    } else {
        match power_type {
            PowerType::PowerDelivery(_) => {
                let _ = line.push_str("USB PD  ");
            }
            PowerType::Standard(_) => {
                let _ = line.push_str("USB2.0  ");
            }
        }
    }

    append_navbar_power_suffix(&mut line, power_w);
    line
}

/// Navbar line 2: `"5.00 V  0.50 A"` or `"20.0 V  0.50 A"` (one fewer decimal when V ≥ 10).
pub fn format_navbar_va_line(limits: Limits) -> String<16> {
    let mut line = String::<16>::new();
    if limits.voltage >= 10.0 {
        let _ = write!(line, "{:.1} V  {:.2} A", limits.voltage, limits.current);
    } else {
        let _ = write!(line, "{:.2} V  {:.2} A", limits.voltage, limits.current);
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Limits, PowerType};

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

    #[test]
    fn navbar_status_linked() {
        assert_eq!(
            format_navbar_status_line(
                PowerType::PowerDelivery(Limits {
                    voltage: 20.0,
                    current: 5.0,
                }),
                true,
            )
            .as_str(),
            "LINKED  100. W"
        );
        assert_eq!(
            format_navbar_status_line(
                PowerType::PowerDelivery(Limits {
                    voltage: 20.0,
                    current: 2.25,
                }),
                true,
            )
            .as_str(),
            "LINKED  45.0 W"
        );
    }

    #[test]
    fn navbar_status_usb_pd() {
        assert_eq!(
            format_navbar_status_line(
                PowerType::PowerDelivery(Limits {
                    voltage: 20.0,
                    current: 5.0,
                }),
                false,
            )
            .as_str(),
            "USB PD  100. W"
        );
    }

    #[test]
    fn navbar_status_usb2() {
        assert_eq!(
            format_navbar_status_line(
                PowerType::Standard(Limits {
                    voltage: 20.0,
                    current: 5.0,
                }),
                false,
            )
            .as_str(),
            "USB2.0  100. W"
        );
        assert_eq!(
            format_navbar_status_line(
                PowerType::Standard(Limits {
                    voltage: 15.0,
                    current: 1.0,
                }),
                false,
            )
            .as_str(),
            "USB2.0  15.0 W"
        );
        assert_eq!(
            format_navbar_status_line(
                PowerType::Standard(Limits {
                    voltage: 5.0,
                    current: 0.5,
                }),
                false,
            )
            .as_str(),
            "USB2.0  2.50 W"
        );
    }

    #[test]
    fn navbar_va_line_single_digit() {
        assert_eq!(
            format_navbar_va_line(Limits {
                voltage: 5.0,
                current: 0.5,
            })
            .as_str(),
            "5.00 V  0.50 A"
        );
        assert_eq!(
            format_navbar_va_line(Limits {
                voltage: 9.0,
                current: 0.5,
            })
            .as_str(),
            "9.00 V  0.50 A"
        );
    }

    #[test]
    fn navbar_va_line_double_digit() {
        assert_eq!(
            format_navbar_va_line(Limits {
                voltage: 15.0,
                current: 0.5,
            })
            .as_str(),
            "15.0 V  0.50 A"
        );
        assert_eq!(
            format_navbar_va_line(Limits {
                voltage: 20.0,
                current: 3.25,
            })
            .as_str(),
            "20.0 V  3.25 A"
        );
    }
}
