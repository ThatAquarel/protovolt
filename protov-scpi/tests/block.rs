use protov_scpi::{
    FWUP_SIGNATURE_LEN, decode_hex_block, encode_definite_block, encode_fwup_appl_line,
    encode_fwup_data_line, parse_definite_block_at, parse_definite_block_in_ascii,
};

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
fn encode_fwup_data_line_format() {
    let line = encode_fwup_data_line(b"abc");
    assert!(line.starts_with(b"SYST:FWUP:DATA #"));
    assert!(line.ends_with(b"\n"));
}

#[test]
fn encode_fwup_appl_line_format() {
    let sig = [0xABu8; FWUP_SIGNATURE_LEN];
    let line = encode_fwup_appl_line(&sig);
    assert!(line.starts_with("SYST:FWUP:APPL #H"));
}

#[test]
fn encode_definite_block_roundtrip() {
    let payload = b"hello";
    let encoded = encode_definite_block(payload);
    let (hdr, len) = parse_definite_block_at(&encoded).unwrap();
    assert_eq!(&encoded[hdr..hdr + len], payload);
}
