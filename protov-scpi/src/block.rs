use crate::command::FWUP_MAX_BLOCK_LEN;

#[cfg(feature = "std")]
use crate::command::{FWUP_DATA_PREFIX, FWUP_SIGNATURE_LEN};

#[cfg(feature = "std")]
use core::fmt::Write;

/// Parse a definite-length block header starting at `#`.
///
/// IEEE 488.2 format: `#Nd<d>` where `N` is the count of length digits and `d` is the
/// decimal payload length. Returns `(header_byte_len, payload_len)`.
pub fn parse_definite_block_at(data: &[u8]) -> Option<(usize, usize)> {
    if data.first() != Some(&b'#') {
        return None;
    }
    if data.len() < 2 {
        return None;
    }
    let digit_count = (*data.get(1)?).checked_sub(b'0')? as usize;
    if digit_count == 0 || digit_count > 9 {
        return None;
    }
    let header_len = 2 + digit_count;
    if data.len() < header_len {
        return None;
    }
    let len_bytes = &data[2..header_len];
    if !len_bytes.iter().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let len_str = core::str::from_utf8(len_bytes).ok()?;
    let payload_len: usize = len_str.parse().ok()?;
    Some((header_len, payload_len))
}

/// Find `#` in `s` and parse the definite-length block header there.
pub fn parse_definite_block_in_ascii(s: &str) -> Option<(usize, usize)> {
    let hash = s.find('#')?;
    parse_definite_block_at(s.as_bytes().get(hash..)?).map(|(rel, len)| (hash + rel, len))
}

/// Decode `#H<hex>` into raw bytes. `hex` must be even length, all hex digits.
pub fn decode_hex_block(hex: &str) -> Option<heapless::Vec<u8, 128>> {
    let hex = hex.strip_prefix("#H").unwrap_or(hex);
    if !hex.len().is_multiple_of(2) || hex.len() > 256 {
        return None;
    }
    if !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let mut out = heapless::Vec::new();
    let bytes = hex.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let hi = hex_nibble(bytes[i])?;
        let lo = hex_nibble(bytes[i + 1])?;
        out.push((hi << 4) | lo).ok()?;
        i += 2;
    }
    Some(out)
}

/// Returns true when `len` is a valid FWUP DATA payload size.
pub fn is_valid_block_len(len: usize) -> bool {
    len > 0 && len <= FWUP_MAX_BLOCK_LEN
}

fn hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// Hex encoding failed (buffer too small).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HexFormatError;

/// Format raw bytes as `#H` followed by uppercase hex (Ed25519 signatures in responses).
pub fn format_hex_block<const CAP: usize>(
    bytes: &[u8],
    buf: &mut heapless::String<CAP>,
) -> Result<(), HexFormatError> {
    use core::fmt::Write;

    if buf.len() + 2 + bytes.len() * 2 > CAP {
        return Err(HexFormatError);
    }
    buf.push_str("#H").map_err(|_| HexFormatError)?;
    for b in bytes {
        let _ = write!(buf, "{:02X}", b);
    }
    Ok(())
}

/// Encode payload as IEEE `#Nd` definite-length block (host / std).
#[cfg(feature = "std")]
pub fn encode_definite_block(payload: &[u8]) -> alloc::vec::Vec<u8> {
    let len = payload.len();
    let len_str = len.to_string();
    let nd = len_str.len();
    let mut out = alloc::vec::Vec::with_capacity(2 + nd + len);
    out.push(b'#');
    out.extend_from_slice(format!("{nd}").as_bytes());
    out.extend_from_slice(len_str.as_bytes());
    out.extend_from_slice(payload);
    out
}

/// `SYST:FWUP:DATA #N<payload>\n`
#[cfg(feature = "std")]
pub fn encode_fwup_data_line(payload: &[u8]) -> alloc::vec::Vec<u8> {
    let mut line = alloc::vec::Vec::with_capacity(FWUP_DATA_PREFIX.len() + payload.len() + 32);
    line.extend_from_slice(FWUP_DATA_PREFIX.as_bytes());
    line.push(b' ');
    line.extend_from_slice(&encode_definite_block(payload));
    line.push(b'\n');
    line
}

/// `SYST:FWUP:APPL #H<hex128>\n`
#[cfg(feature = "std")]
pub fn encode_fwup_appl_line(signature: &[u8; FWUP_SIGNATURE_LEN]) -> alloc::string::String {
    let mut hex = alloc::string::String::with_capacity(FWUP_SIGNATURE_LEN * 2);
    for b in signature {
        let _ = write!(hex, "{b:02X}");
    }
    format!("SYST:FWUP:APPL #H{hex}\n")
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn parse_block_header_128_bytes() {
        let (hdr, len) = parse_definite_block_at(b"#3128").unwrap();
        assert_eq!(hdr, 5);
        assert_eq!(len, 128);
    }

    #[test]
    fn parse_block_header_4096_bytes() {
        let (hdr, len) = parse_definite_block_at(b"#44096").unwrap();
        assert_eq!(hdr, 6);
        assert_eq!(len, 4096);
    }

    #[test]
    fn parse_block_in_command_prefix() {
        let (end, len) = parse_definite_block_in_ascii("SYST:FWUP:DATA #44096").unwrap();
        assert_eq!(end, "SYST:FWUP:DATA #44096".len());
        assert_eq!(len, 4096);
    }

    #[test]
    fn decode_signature_hex() {
        let hex = "#H".to_string() + &"ab".repeat(64);
        let bytes = decode_hex_block(&hex).unwrap();
        assert_eq!(bytes.len(), 64);
        assert_eq!(bytes[0], 0xab);
    }

    #[test]
    fn encode_definite_block_roundtrip() {
        let payload = b"hello";
        let encoded = encode_definite_block(payload);
        let (hdr, len) = parse_definite_block_at(&encoded).unwrap();
        assert_eq!(&encoded[hdr..hdr + len], payload);
    }
}
