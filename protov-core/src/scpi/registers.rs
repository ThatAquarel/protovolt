//! Static register dump strings for simulator / mock SCPI responses.

use super::{RESPONSE_BUF, RegisterChannel};

const INA226_CHA: &str = "INA226 @ 0x41 (I2C1)|00 CONFIG        4127|01 SHUNT_V       0008|02 BUS_V         28A0|03 POWER         0064|04 CURRENT       00A3|05 CALIBRATION    1000|06 ENABLE         0007|07 ALERT_LIMIT    0000|FE MANUF_ID       5449|FF DIE_ID         2260";

const INA226_CHB: &str = "INA226 @ 0x40 (I2C1)|00 CONFIG        4127|01 SHUNT_V       0008|02 BUS_V         28A0|03 POWER         0064|04 CURRENT       00A3|05 CALIBRATION    1000|06 ENABLE         0007|07 ALERT_LIMIT    0000|FE MANUF_ID       5449|FF DIE_ID         2260";

const TPS55289_CHA: &str = "TPS55289 @ 0x75 (I2C0)|00 REF_LSB         C8|01 REF_MSB         00|02 IOUT_LIMIT      64|03 VOUT_SR          0|04 VOUT_FS          0|05 CDC              0|06 MODE            03|07 STATUS          20";

const TPS55289_CHB: &str = "TPS55289 @ 0x74 (I2C0)|00 REF_LSB         C8|01 REF_MSB         00|02 IOUT_LIMIT      64|03 VOUT_SR          0|04 VOUT_FS          0|05 CDC              0|06 MODE            03|07 STATUS          20";

pub fn ina226_dump(channel: RegisterChannel) -> &'static str {
    match channel {
        RegisterChannel::Cha => INA226_CHA,
        RegisterChannel::Chb => INA226_CHB,
    }
}

pub fn tps55289_dump(channel: RegisterChannel) -> &'static str {
    match channel {
        RegisterChannel::Cha => TPS55289_CHA,
        RegisterChannel::Chb => TPS55289_CHB,
    }
}

pub fn format_ina226<const N: usize>(
    channel: RegisterChannel,
    buf: &mut heapless::String<N>,
) -> Result<(), ()> {
    buf.push_str(ina226_dump(channel)).map_err(|_| ())
}

pub fn format_tps55289<const N: usize>(
    channel: RegisterChannel,
    buf: &mut heapless::String<N>,
) -> Result<(), ()> {
    buf.push_str(tps55289_dump(channel)).map_err(|_| ())
}

pub fn format_ina226_response(channel: RegisterChannel) -> Option<heapless::String<RESPONSE_BUF>> {
    let mut buf = heapless::String::<RESPONSE_BUF>::new();
    format_ina226(channel, &mut buf).ok()?;
    Some(buf)
}

pub fn format_tps55289_response(
    channel: RegisterChannel,
) -> Option<heapless::String<RESPONSE_BUF>> {
    let mut buf = heapless::String::<RESPONSE_BUF>::new();
    format_tps55289(channel, &mut buf).ok()?;
    Some(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ina226_dump_contains_address() {
        assert!(ina226_dump(RegisterChannel::Cha).contains("INA226 @ 0x41"));
        assert!(ina226_dump(RegisterChannel::Cha).contains('|'));
    }

    #[test]
    fn tps55289_dump_contains_address() {
        assert!(tps55289_dump(RegisterChannel::Chb).contains("TPS55289 @ 0x74"));
    }
}
