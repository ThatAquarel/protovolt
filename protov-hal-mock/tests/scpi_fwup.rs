use protov_hal_mock::{MockDevice, SLOT_PROFILES, pool::identity_from_profile};
use protov_scpi::{FWUP_MAX_BLOCK_LEN, encode_fwup_appl_line, encode_fwup_data_line};

fn device() -> MockDevice {
    MockDevice::with_identity(identity_from_profile(SLOT_PROFILES[0]))
}

fn send_bytes(device: &mut MockDevice, data: &[u8]) -> Vec<String> {
    device.handle_bytes(data)
}

fn send_line(device: &mut MockDevice, line: &str) -> Option<String> {
    let responses = send_bytes(device, format!("{line}\n").as_bytes());
    assert!(responses.len() <= 1);
    responses.into_iter().next()
}

fn send_fwup_data(device: &mut MockDevice, payload: &[u8]) -> Option<String> {
    let line = encode_fwup_data_line(payload);
    let responses = send_bytes(device, &line);
    assert!(responses.len() <= 1);
    responses.into_iter().next()
}

#[test]
fn fwup_star_prepare_recv() {
    let mut device = device();
    assert_eq!(device.handle("SYST:FWUP:STAT?").as_deref(), Some("IDLE"));
    assert_eq!(
        send_line(&mut device, "SYST:FWUP:STAR 4096").as_deref(),
        Some("OK")
    );
    assert_eq!(
        device.handle("SYST:FWUP:STAT?").as_deref(),
        Some("RECV,0/4096")
    );
}

#[test]
fn fwup_data_single_block() {
    let mut device = device();
    send_line(&mut device, "SYST:FWUP:STAR 4096");
    let payload = vec![0xAB; 4096];
    assert_eq!(send_fwup_data(&mut device, &payload).as_deref(), Some("OK"));
    assert_eq!(
        device.handle("SYST:FWUP:STAT?").as_deref(),
        Some("READY,4096")
    );
}

#[test]
fn fwup_full_image_ready_appl_ok() {
    let mut device = device();
    send_line(&mut device, "SYST:FWUP:STAR 256");
    let payload = vec![0xCD; 256];
    assert_eq!(send_fwup_data(&mut device, &payload).as_deref(), Some("OK"));
    assert_eq!(
        device.handle("SYST:FWUP:STAT?").as_deref(),
        Some("READY,256")
    );

    let sig = [0xCD; 64];
    let appl_line = encode_fwup_appl_line(&sig);
    assert_eq!(send_line(&mut device, &appl_line).as_deref(), Some("OK"));
    assert_eq!(device.handle("SYST:FWUP:STAT?").as_deref(), Some("IDLE"));
}

#[test]
fn fwup_abor_mid_transfer() {
    let mut device = device();
    send_line(&mut device, "SYST:FWUP:STAR 4096");
    assert_eq!(
        send_line(&mut device, "SYST:FWUP:ABOR").as_deref(),
        Some("OK")
    );
    assert!(!device.app.is_update_mode());
    assert_eq!(device.handle("SYST:FWUP:STAT?").as_deref(), Some("IDLE"));
}

#[test]
fn fwup_data_before_star() {
    let mut device = device();
    let payload = vec![0u8; 64];
    assert_eq!(
        send_fwup_data(&mut device, &payload).as_deref(),
        Some("ERR")
    );
}

#[test]
fn fwup_oversize_block() {
    let mut device = device();
    send_line(&mut device, "SYST:FWUP:STAR 8192");
    assert_eq!(
        device.handle("SYST:FWUP:STAT?").as_deref(),
        Some("RECV,0/8192")
    );

    let oversize = FWUP_MAX_BLOCK_LEN + 1;
    let line = encode_fwup_data_line(&vec![0u8; oversize]);
    let responses = send_bytes(&mut device, &line);
    assert!(responses.is_empty());
    assert_eq!(
        device.handle("SYST:FWUP:STAT?").as_deref(),
        Some("RECV,0/8192")
    );
}

#[test]
fn fwup_appl_before_ready() {
    let mut device = device();
    send_line(&mut device, "SYST:FWUP:STAR 4096");
    let sig = [0xAB; 64];
    let appl_line = encode_fwup_appl_line(&sig);
    assert_eq!(send_line(&mut device, &appl_line).as_deref(), Some("ERR"));
}

#[test]
fn fwup_multi_block_progress() {
    let mut device = device();
    send_line(&mut device, "SYST:FWUP:STAR 8192");
    let block = vec![0x01; FWUP_MAX_BLOCK_LEN];
    assert_eq!(send_fwup_data(&mut device, &block).as_deref(), Some("OK"));
    assert_eq!(
        device.handle("SYST:FWUP:STAT?").as_deref(),
        Some("RECV,4096/8192")
    );
    assert_eq!(send_fwup_data(&mut device, &block).as_deref(), Some("OK"));
    assert_eq!(
        device.handle("SYST:FWUP:STAT?").as_deref(),
        Some("READY,8192")
    );
}
