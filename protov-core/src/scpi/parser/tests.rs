use super::*;

#[test]
fn normalize_uppercases_and_collapses_spaces() {
    let s = normalize_command("  ch1:volt?  \t  ");
    assert_eq!(s.as_str(), "CH1:VOLT?");
}

#[test]
fn parse_idn_and_rst() {
    assert_eq!(parse_command("*idn?"), Some(ScpiCommand::IdnQuery));
    assert_eq!(parse_command("*RST"), Some(ScpiCommand::Rst));
}

#[test]
fn parse_star_slots() {
    assert_eq!(parse_command("*SAV 3"), Some(ScpiCommand::Sav { slot: 3 }));
    assert_eq!(parse_command("*RCL 9"), Some(ScpiCommand::Rcl { slot: 9 }));
    assert_eq!(parse_command("*DEL 1"), Some(ScpiCommand::Del { slot: 1 }));
    assert!(parse_command("*SAV 0").is_some());
}

#[test]
fn parse_channel_queries() {
    assert_eq!(
        parse_command("CH1:MODE?"),
        Some(ScpiCommand::ChannelQuery {
            channel: ScpiChannel::Ch1,
            param: ChannelParam::Mode,
        })
    );
    assert_eq!(
        parse_command("CH2:OVP?"),
        Some(ScpiCommand::ChannelQuery {
            channel: ScpiChannel::Ch2,
            param: ChannelParam::Ovp,
        })
    );
}

#[test]
fn parse_output_and_meas() {
    assert_eq!(
        parse_command("OUTP CH1,ON"),
        Some(ScpiCommand::OutputSet {
            channel: ScpiChannel::Ch1,
            on: true,
        })
    );
    assert_eq!(
        parse_command("MEAS:VOLT? CH2"),
        Some(ScpiCommand::MeasQuery {
            kind: MeasKind::Volt,
            channel: ScpiChannel::Ch2,
        })
    );
}

#[test]
fn parse_reset_prot() {
    assert_eq!(
        parse_command("OUTP:RESET:PROT"),
        Some(ScpiCommand::ResetProt { channel: None })
    );
    assert_eq!(
        parse_command("OUTP:RESET:PROT CH1"),
        Some(ScpiCommand::ResetProt {
            channel: Some(ScpiChannel::Ch1),
        })
    );
}

#[test]
fn unknown_command() {
    let cmd = parse_command("NOTREAL").unwrap();
    assert!(matches!(cmd, ScpiCommand::Unknown { .. }));
}

#[test]
fn parse_fwup_commands() {
    assert_eq!(parse_command("SYST:FWUP:STAT?"), Some(ScpiCommand::FwupStatQuery));
    assert_eq!(
        parse_command("SYST:FWUP:STAR 8192"),
        Some(ScpiCommand::FwupStar { size: 8192 })
    );
    assert_eq!(parse_command("SYST:FWUP:ABOR"), Some(ScpiCommand::FwupAbor));
    assert_eq!(parse_command("SYST:FWUP:DATA"), Some(ScpiCommand::FwupData));
    let sig_hex = "#H".to_string() + &"ab".repeat(64);
    let cmd = parse_command(&format!("SYST:FWUP:APPL {sig_hex}")).unwrap();
    assert!(matches!(cmd, ScpiCommand::FwupAppl { .. }));
}

#[test]
fn is_mutation_matrix() {
    assert!(!is_mutation(&ScpiCommand::IdnQuery));
    assert!(!is_mutation(&ScpiCommand::MeasQuery {
        kind: MeasKind::Volt,
        channel: ScpiChannel::Ch1,
    }));
    assert!(is_mutation(&ScpiCommand::Rst));
    assert!(is_mutation(&ScpiCommand::OutputSet {
        channel: ScpiChannel::Ch1,
        on: true,
    }));
    assert!(is_mutation(&ScpiCommand::SystLoc));
    assert!(is_mutation(&ScpiCommand::FwupStar { size: 4096 }));
    assert!(!is_mutation(&ScpiCommand::FwupStatQuery));
}

#[test]
fn update_mode_whitelist() {
    assert!(is_allowed_in_update_mode(&ScpiCommand::IdnQuery));
    assert!(is_allowed_in_update_mode(&ScpiCommand::FwupData));
    assert!(!is_allowed_in_update_mode(&ScpiCommand::OutputSet {
        channel: ScpiChannel::Ch1,
        on: true,
    }));
    assert!(requires_active_update_session(&ScpiCommand::FwupData));
}
