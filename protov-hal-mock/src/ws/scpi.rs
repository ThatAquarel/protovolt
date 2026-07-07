use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, info};

use crate::pool::DevicePool;
use crate::scpi::ScpiReader;

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
    use crate::scpi::POOL_FULL_RESPONSE;

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

    let mut reader = ScpiReader::new();

    while let Some(message) = read.next().await {
        let message = message?;
        let data = match message {
            Message::Text(text) => text.as_bytes().to_vec(),
            Message::Binary(bytes) => bytes.to_vec(),
            Message::Close(_) => break,
            _ => continue,
        };
        if data.is_empty() {
            continue;
        }

        let responses = pool.with_device(slot, |device| {
            device.handle_bytes_with_reader(&mut reader, &data)
        });
        if let Some(responses) = responses {
            for text in responses {
                write
                    .send(Message::Text(format!("{text}\n").into()))
                    .await?;
            }
        }
    }

    info!("SCPI client {peer} closed slot {slot}");
    Ok(())
}
