use protov_hal_mock::{MockDevice, SLOT_PROFILES, pool::identity_from_profile};

fn device() -> MockDevice {
    MockDevice::with_identity(identity_from_profile(SLOT_PROFILES[0]))
}

#[test]
fn unknown_command_pushes_err() {
    let mut device = device();
    assert_eq!(device.handle("NOTREAL"), None);
    assert_eq!(
        device.handle("SYST:ERR?").as_deref(),
        Some(r#"-113,"Unknown command: NOTREAL""#)
    );
}

#[test]
fn normalization_lowercase() {
    let mut device = device();
    device.handle("ch1:volt 3.3");
    assert_eq!(device.handle("ch1:volt?").as_deref(), Some("3.300"));
}

#[test]
fn mutation_silent_query_responds() {
    let mut device = device();
    assert_eq!(device.handle("CH1:VOLT 3.3"), None);
    assert_eq!(device.handle("CH1:VOLT?").as_deref(), Some("3.300"));
}

#[test]
fn update_mode_rejects_outp() {
    let mut device = device();
    assert_eq!(device.handle("SYST:FWUP:STAR 4096").as_deref(), Some("OK"));
    assert_eq!(
        device.handle("SYST:FWUP:STAT?").as_deref(),
        Some("RECV,0/4096")
    );
    assert_eq!(device.handle("OUTP CH1,ON").as_deref(), Some("ERR"));
}

#[test]
fn register_dump_static() {
    let mut device = device();
    let response = device.handle("INA226:REG? CHA");
    assert!(response.is_some());
    assert!(!response.unwrap().is_empty());
}

#[test]
fn lcd_brig_protocol_form() {
    let mut device = device();
    assert_eq!(device.handle("LCD:BRIG 128"), None);
    assert_eq!(device.handle("LCD:BRIG?").as_deref(), Some("128"));
}

#[test]
fn syst_idat_returns_serial_hw_and_signature() {
    let mut device = device();
    let response = device.handle("SYST:IDAT?").expect("SYST:IDAT? response");
    assert!(response.starts_with("550e8400,A.1,2026-06-27,#H5555555555555555,#H"));
    assert_eq!(
        response.len(),
        "550e8400,A.1,2026-06-27,#H5555555555555555,#H".len() + 128
    );
}

#[test]
fn sav_rcl_smoke() {
    let mut device = device();
    device.handle("CH1:VOLT 5.0");
    assert_eq!(device.handle("*SAV 1"), None);
    device.handle("CH1:VOLT 1.0");
    assert_eq!(device.handle("*RCL 1"), None);
    assert_eq!(device.handle("CH1:VOLT?").as_deref(), Some("5.000"));
}
