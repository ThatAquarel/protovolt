use super::*;

#[test]
fn parse_rgb_valid() {
    let rgb = parse_rgb_triplet("255,128,0").unwrap();
    assert_eq!(rgb.r, 255);
    assert_eq!(rgb.g, 128);
    assert_eq!(rgb.b, 0);
}

#[test]
fn parse_rgb_rejects_out_of_range() {
    assert!(parse_rgb_triplet("256,0,0").is_err());
    assert!(parse_rgb_triplet("1,2").is_err());
}

#[test]
fn parse_brightness_edges() {
    assert_eq!(parse_brightness("0").unwrap(), 0);
    assert_eq!(parse_brightness("255").unwrap(), 255);
    assert!(parse_brightness("256").is_err());
}

#[test]
fn darken_scales_channels() {
    let base = Rgb {
        r: 100,
        g: 200,
        b: 50,
    };
    let dark = darken(base);
    assert!(dark.r < base.r);
    assert!(dark.g < base.g);
    assert!(dark.b < base.b);
}

#[test]
fn scale_led_respects_brightness() {
    let rgb = Rgb {
        r: 255,
        g: 255,
        b: 255,
    };
    let dim = scale_led(rgb, 0);
    assert_eq!(dim.r, 0);
    let bright = scale_led(rgb, 255);
    assert_eq!(bright.r, 255);
}
