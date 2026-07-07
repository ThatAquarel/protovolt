use protov_scpi::{
    ChannelParam, MeasKind, ScpiChannel, ScpiCommand, is_allowed_in_update_mode, is_mutation,
    normalize_command, parse_command, requires_active_update_session,
};

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
fn parse_syst_idat() {
    assert_eq!(
        parse_command("SYST:IDAT?"),
        Some(ScpiCommand::SystIdatQuery)
    );
    assert_eq!(
        parse_command("syst:idat?"),
        Some(ScpiCommand::SystIdatQuery)
    );
    assert!(!is_mutation(&ScpiCommand::SystIdatQuery));
    assert!(is_allowed_in_update_mode(&ScpiCommand::SystIdatQuery));
}

#[test]
fn parse_fwup_commands() {
    assert_eq!(
        parse_command("SYST:FWUP:STAT?"),
        Some(ScpiCommand::FwupStatQuery)
    );
    assert_eq!(
        parse_command("SYST:FWUP:STAR 8192"),
        Some(ScpiCommand::FwupStar { size: 8192 })
    );
    let sig_hex = "#H".to_string() + &"ab".repeat(64);
    let cmd = parse_command(&format!("SYST:FWUP:APPL {sig_hex}")).unwrap();
    assert!(matches!(cmd, ScpiCommand::FwupAppl { .. }));
}

#[test]
fn is_mutation_matrix() {
    assert!(!is_mutation(&ScpiCommand::IdnQuery));
    assert!(is_mutation(&ScpiCommand::FwupStar { size: 4096 }));
    assert!(!is_mutation(&ScpiCommand::FwupStatQuery));
}

#[test]
fn update_mode_whitelist() {
    assert!(is_allowed_in_update_mode(&ScpiCommand::IdnQuery));
    assert!(is_allowed_in_update_mode(&ScpiCommand::FwupData));
    assert!(requires_active_update_session(&ScpiCommand::FwupData));
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
}

#[test]
fn parse_output_and_meas() {
    assert_eq!(
        parse_command("MEAS:VOLT? CH2"),
        Some(ScpiCommand::MeasQuery {
            kind: MeasKind::Volt,
            channel: ScpiChannel::Ch2,
        })
    );
}
