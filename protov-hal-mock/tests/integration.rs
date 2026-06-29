use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use protov_hal_mock::state::{ControlResponse, apply_snapshot, dump_snapshot, load_snapshot_json};
use protov_hal_mock::{
    MockDevice, RunningServer, SLOT_PROFILES, ServerConfig, pool::identity_from_profile,
};
use tokio_tungstenite::{connect_async, tungstenite::Message};

fn ws_url(addr: std::net::SocketAddr) -> String {
    format!("ws://{addr}")
}

async fn wait_for_scpi(addr: std::net::SocketAddr) {
    for _ in 0..100 {
        if let Ok((mut ws, _)) = connect_async(ws_url(addr)).await {
            let _ = ws.close(None).await;
            tokio::time::sleep(Duration::from_millis(20)).await;
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("timed out waiting for SCPI websocket at {addr}");
}

fn slot_for_serial(serial: &str) -> u8 {
    SLOT_PROFILES
        .iter()
        .position(|profile| profile.serial == serial)
        .unwrap_or(0) as u8
}

#[serial_test::serial]
#[tokio::test]
async fn scpi_websocket_idn() {
    let server = RunningServer::start(ServerConfig {
        bind: "127.0.0.1".to_owned(),
        scpi_port: 0,
        control_port: 0,
    })
    .await
    .unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    let (mut ws, _) = connect_async(ws_url(server.scpi_addr)).await.unwrap();
    ws.send(Message::Text("*IDN?\n".into())).await.unwrap();
    let response = ws.next().await.unwrap().unwrap().into_text().unwrap();
    assert_eq!(response.trim(), "FBRD Inc.,ProtoV MINI,550e8400,1.0.0,A.1");
    let _ = ws.close(None).await;
    server.shutdown();
}

#[serial_test::serial]
#[tokio::test]
async fn scpi_websocket_four_distinct_devices() {
    let server = RunningServer::start(ServerConfig {
        bind: "127.0.0.1".to_owned(),
        scpi_port: 0,
        control_port: 0,
    })
    .await
    .unwrap();
    wait_for_scpi(server.scpi_addr).await;

    let mut sockets = Vec::new();
    let mut serials = Vec::new();
    for _ in 0..4 {
        let (mut ws, _) = connect_async(ws_url(server.scpi_addr)).await.unwrap();
        ws.send(Message::Text("*IDN?\n".into())).await.unwrap();
        let msg = ws.next().await.expect("missing IDN response").unwrap();
        let response = msg.into_text().unwrap();
        serials.push(response.split(',').nth(2).unwrap().to_owned());
        sockets.push(ws);
    }
    drop(sockets);

    serials.sort();
    serials.dedup();
    assert_eq!(serials.len(), 4);
    assert!(serials.contains(&"550e8400".to_owned()));
    assert!(serials.contains(&"32983fe4".to_owned()));
    assert!(serials.contains(&"deadbeef".to_owned()));
    assert!(serials.contains(&"a1b2c3d4".to_owned()));
    server.shutdown();
}

#[serial_test::serial]
#[tokio::test]
async fn scpi_websocket_pool_full() {
    let server = RunningServer::start(ServerConfig {
        bind: "127.0.0.1".to_owned(),
        scpi_port: 0,
        control_port: 0,
    })
    .await
    .unwrap();
    wait_for_scpi(server.scpi_addr).await;

    let mut sockets = Vec::new();
    for _ in 0..4 {
        let (ws, _) = connect_async(ws_url(server.scpi_addr)).await.unwrap();
        sockets.push(ws);
    }

    let (mut ws, _) = connect_async(ws_url(server.scpi_addr)).await.unwrap();
    ws.send(Message::Text("*IDN?\n".into())).await.unwrap();
    let response = ws.next().await.unwrap().unwrap().into_text().unwrap();
    assert!(response.contains("All four mock device slots are in use"));
    server.shutdown();
}

#[serial_test::serial]
#[tokio::test]
async fn control_websocket_ping_status_load() {
    let server = RunningServer::start(ServerConfig {
        bind: "127.0.0.1".to_owned(),
        scpi_port: 0,
        control_port: 0,
    })
    .await
    .unwrap();
    wait_for_scpi(server.scpi_addr).await;

    let (mut scpi, _) = connect_async(ws_url(server.scpi_addr)).await.unwrap();
    scpi.send(Message::Text("*IDN?\n".into())).await.unwrap();
    let idn = scpi.next().await.unwrap().unwrap().into_text().unwrap();
    let serial = idn.split(',').nth(2).unwrap();
    let slot = slot_for_serial(serial);

    let (mut ctrl, _) = connect_async(ws_url(server.control_addr)).await.unwrap();
    ctrl.send(Message::Text(r#"{"action":"ping"}"#.into()))
        .await
        .unwrap();
    let pong: ControlResponse =
        serde_json::from_str(&ctrl.next().await.unwrap().unwrap().into_text().unwrap()).unwrap();
    assert!(pong.ok);
    assert_eq!(pong.message.as_deref(), Some("pong"));

    let state_json = include_str!("../states/ch1-active.json");
    ctrl.send(Message::Text(
        format!(r#"{{"action":"load","slot":{slot},"state":{state_json}}}"#).into(),
    ))
    .await
    .unwrap();
    let loaded: ControlResponse =
        serde_json::from_str(&ctrl.next().await.unwrap().unwrap().into_text().unwrap()).unwrap();
    assert!(loaded.ok);

    scpi.send(Message::Text("MEAS:VOLT? CH1\n".into()))
        .await
        .unwrap();
    let voltage: f32 = scpi
        .next()
        .await
        .unwrap()
        .unwrap()
        .into_text()
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    assert!((voltage - 3.298).abs() < 0.01);

    server.shutdown();
}

#[test]
fn parity_idn_default_profile() {
    let mut device = MockDevice::with_identity(identity_from_profile(SLOT_PROFILES[0]));
    assert_eq!(
        device.handle("*IDN?").as_deref(),
        Some("FBRD Inc.,ProtoV MINI,550e8400,1.0.0,A.1")
    );
}

#[test]
fn parity_voltage_set_query() {
    let mut device = MockDevice::with_identity(identity_from_profile(SLOT_PROFILES[0]));
    device.handle("CH1:VOLT 3.3");
    assert_eq!(device.handle("CH1:VOLT?").as_deref(), Some("3.300"));
}

#[test]
fn parity_output_on_off() {
    let mut device = MockDevice::with_identity(identity_from_profile(SLOT_PROFILES[0]));
    device.handle("OUTP CH1,ON");
    assert_eq!(device.handle("OUTP? CH1").as_deref(), Some("ON"));
    device.handle("OUTP CH1,OFF");
    assert_eq!(device.handle("OUTP? CH1").as_deref(), Some("OFF"));
}

#[test]
fn parity_error_queue() {
    let mut device = MockDevice::with_identity(identity_from_profile(SLOT_PROFILES[0]));
    device.scpi.seed_error(-221, "Settings conflict");
    assert_eq!(
        device.handle("SYST:ERR?").as_deref(),
        Some(r#"-221,"Settings conflict""#)
    );
    assert_eq!(
        device.handle("SYST:ERR?").as_deref(),
        Some(r#"0,"No error""#)
    );
}

#[test]
fn parity_protection_tripped_preset() {
    let json = include_str!("../states/protection-tripped.json");
    let snapshot = load_snapshot_json(json).unwrap();
    let mut device = MockDevice::with_identity(identity_from_profile(SLOT_PROFILES[2]));
    apply_snapshot(&mut device, &snapshot);
    assert_eq!(device.handle("CH1:MODE?").as_deref(), Some("OVP"));
}

#[test]
fn state_roundtrip_default() {
    for file in [
        "default.json",
        "ch1-active.json",
        "dual-output.json",
        "protection-tripped.json",
    ] {
        let json = match file {
            "default.json" => include_str!("../states/default.json"),
            "ch1-active.json" => include_str!("../states/ch1-active.json"),
            "dual-output.json" => include_str!("../states/dual-output.json"),
            "protection-tripped.json" => include_str!("../states/protection-tripped.json"),
            _ => unreachable!(),
        };
        let snapshot = load_snapshot_json(json).unwrap();
        let mut device = MockDevice::with_identity(identity_from_profile(SLOT_PROFILES[0]));
        apply_snapshot(&mut device, &snapshot);
        let dumped = dump_snapshot(&device);
        assert_eq!(dumped.remote, snapshot.remote);
        assert_eq!(
            dumped.channel("CH1").map(|c| c.voltage),
            snapshot.channel("CH1").map(|c| c.voltage)
        );
    }
}
