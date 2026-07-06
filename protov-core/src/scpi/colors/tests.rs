use super::*;
use embedded_graphics::pixelcolor::RgbColor;

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

#[test]
fn format_rgb_triplet() {
    let rgb = Rgb {
        r: 10,
        g: 20,
        b: 30,
    };
    assert_eq!(format_rgb(rgb).as_str(), "10,20,30");
}

#[test]
fn to_rgb565_quantizes_channels() {
    let rgb = Rgb {
        r: 255,
        g: 255,
        b: 255,
    };
    let px = to_rgb565(rgb);
    assert_eq!(px.r(), 31);
    assert_eq!(px.g(), 63);
    assert_eq!(px.b(), 31);
}

#[test]
fn scale_white_follows_brightness() {
    let off = scale_white(0);
    assert_eq!(off.r, 0);
    assert_eq!(off.g, 0);
    assert_eq!(off.b, 0);
    let half = scale_white(128);
    assert_eq!(half.r, half.g);
    assert_eq!(half.g, half.b);
    assert!(half.r > 0 && half.r < 255);
}

#[test]
fn parse_brightness_rejects_non_numeric() {
    assert!(parse_brightness("abc").is_err());
}
