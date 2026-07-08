use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, info};

use crate::device::FwupAfter;
use crate::pool::DevicePool;
use crate::scpi::ScpiReader;

const FWUP_STAR_POST_DELAY: Duration = Duration::from_secs(2);

struct SlotGuard {
    pool: Arc<DevicePool>,
    slot: usize,
    skip_release: bool,
}

impl Drop for SlotGuard {
    fn drop(&mut self) {
        if !self.skip_release {
            self.pool.release(self.slot);
        }
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
    let mut guard = SlotGuard {
        pool: Arc::clone(&pool),
        slot,
        skip_release: false,
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

        let (responses, fwup_after) = pool
            .with_device(slot, |device| {
                let responses = device.handle_bytes_with_reader(&mut reader, &data);
                let fwup_after = device.fwup_after;
                device.fwup_after = FwupAfter::None;
                (responses, fwup_after)
            })
            .unwrap_or((Vec::new(), FwupAfter::None));

        for text in responses {
            write
                .send(Message::Text(format!("{text}\n").into()))
                .await?;
        }

        match fwup_after {
            FwupAfter::StarDelay => {
                tokio::time::sleep(FWUP_STAR_POST_DELAY).await;
            }
            FwupAfter::ApplReboot => {
                guard.skip_release = true;
                write.send(Message::Close(None)).await?;
                pool.begin_fwup_reboot(slot);
                info!("SCPI client {peer} disconnected for firmware reboot on slot {slot}");
                return Ok(());
            }
            FwupAfter::None => {}
        }
    }

    info!("SCPI client {peer} closed slot {slot}");
    Ok(())
}
