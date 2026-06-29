use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, info};

use crate::dispatch::{POOL_FULL_RESPONSE, handle_command};
use crate::pool::DevicePool;

struct SlotGuard {
    pool: Arc<DevicePool>,
    slot: usize,
}

impl Drop for SlotGuard {
    fn drop(&mut self) {
        self.pool.release(self.slot);
    }
}

pub async fn serve_scpi(pool: Arc<DevicePool>, listener: TcpListener) -> std::io::Result<()> {
    let addr = listener.local_addr()?;
    info!("SCPI WebSocket listening on ws://{addr}");

    loop {
        let (stream, peer) = listener.accept().await?;
        let pool = Arc::clone(&pool);
        tokio::spawn(async move {
            if let Err(error) = handle_connection(pool, stream, peer).await {
                debug!("SCPI client {peer} disconnected: {error}");
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

    let slot = match pool.acquire() {
        Some(slot) => slot,
        None => {
            write
                .send(Message::Text(format!("{POOL_FULL_RESPONSE}\n").into()))
                .await?;
            write.send(Message::Close(None)).await?;
            return Ok(());
        }
    };

    info!("SCPI client {peer} assigned slot {slot}");
    let _guard = SlotGuard {
        pool: Arc::clone(&pool),
        slot,
    };

    while let Some(message) = read.next().await {
        let message = message?;
        if !message.is_text() {
            continue;
        }
        let command = message.to_text()?.trim();
        if command.is_empty() {
            continue;
        }

        let response = pool.with_device(slot, |device| handle_command(device, command));
        if let Some(text) = response.flatten() {
            write
                .send(Message::Text(format!("{text}\n").into()))
                .await?;
        }
    }

    info!("SCPI client {peer} closed slot {slot}");
    Ok(())
}
