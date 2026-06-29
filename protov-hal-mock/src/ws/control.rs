use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, info};

use crate::pool::{DevicePool, MAX_MOCK_DEVICES};
use crate::state::{ControlRequest, ControlResponse, apply_snapshot, dump_snapshot};

pub async fn serve_control(pool: Arc<DevicePool>, listener: TcpListener) -> std::io::Result<()> {
    let addr = listener.local_addr()?;
    info!("Control WebSocket listening on ws://{addr}");

    loop {
        let (stream, peer) = listener.accept().await?;
        let pool = Arc::clone(&pool);
        tokio::spawn(async move {
            if let Err(error) = handle_connection(pool, stream, peer).await {
                debug!("Control client {peer} disconnected: {error}");
            }
        });
    }
}

async fn handle_connection(
    pool: Arc<DevicePool>,
    stream: tokio::net::TcpStream,
    peer: std::net::SocketAddr,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ws = tokio_tungstenite::accept_async(stream).await?;
    let (mut write, mut read) = ws.split();

    while let Some(message) = read.next().await {
        let message = message?;
        if !message.is_text() {
            continue;
        }

        let response = match serde_json::from_str::<ControlRequest>(message.to_text()?) {
            Ok(request) => handle_request(&pool, request),
            Err(error) => ControlResponse::err(format!("invalid JSON: {error}")),
        };

        let payload = serde_json::to_string(&response)?;
        write.send(Message::Text(payload.into())).await?;
    }

    debug!("Control session closed for {peer}");
    Ok(())
}

fn handle_request(pool: &DevicePool, request: ControlRequest) -> ControlResponse {
    match request {
        ControlRequest::Ping => ControlResponse::pong(),
        ControlRequest::Status { slot } => match validated_slot(slot) {
            Ok(slot) => pool
                .with_device(slot, |device| dump_snapshot(device))
                .map(|state| ControlResponse::status(slot as u8, state))
                .unwrap_or_else(|| ControlResponse::err("slot unavailable")),
            Err(error) => ControlResponse::err(error),
        },
        ControlRequest::Reset { slot } => match validated_slot(slot) {
            Ok(slot) => {
                let _ = pool.with_device(slot, |device| {
                    device.reset_to_profile(crate::pool::identity_from_profile(
                        crate::pool::SLOT_PROFILES[slot],
                    ));
                });
                ControlResponse::ok_slot(slot as u8)
            }
            Err(error) => ControlResponse::err(error),
        },
        ControlRequest::Load { slot, state } => match validated_slot(slot) {
            Ok(slot) => pool
                .with_device(slot, |device| apply_snapshot(device, &state))
                .map(|_| ControlResponse::ok_slot(slot as u8))
                .unwrap_or_else(|| ControlResponse::err("slot unavailable")),
            Err(error) => ControlResponse::err(error),
        },
    }
}

fn validated_slot(slot: u8) -> Result<usize, String> {
    let index = slot as usize;
    if index >= MAX_MOCK_DEVICES {
        return Err(format!(
            "invalid slot {slot}; expected 0..{}",
            MAX_MOCK_DEVICES - 1
        ));
    }
    Ok(index)
}
