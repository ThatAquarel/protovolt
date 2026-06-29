use std::net::SocketAddr;
use std::sync::Arc;

use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tracing::info;

use crate::pool::DevicePool;
use crate::ws::{serve_control, serve_scpi};

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub bind: String,
    pub scpi_port: u16,
    pub control_port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind: "127.0.0.1".to_owned(),
            scpi_port: env_u16("PROTOV_MOCK_SCPI_PORT", 8765),
            control_port: env_u16("PROTOV_MOCK_CTRL_PORT", 8766),
        }
    }
}

fn env_u16(name: &str, default: u16) -> u16 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

pub struct RunningServer {
    pub scpi_addr: SocketAddr,
    pub control_addr: SocketAddr,
    pool: Arc<DevicePool>,
    shutdown: oneshot::Sender<()>,
    scpi_task: JoinHandle<()>,
    control_task: JoinHandle<()>,
}

impl RunningServer {
    pub fn pool(&self) -> Arc<DevicePool> {
        Arc::clone(&self.pool)
    }

    pub async fn start(config: ServerConfig) -> std::io::Result<Self> {
        let scpi_listener =
            tokio::net::TcpListener::bind((config.bind.as_str(), config.scpi_port)).await?;
        let control_listener =
            tokio::net::TcpListener::bind((config.bind.as_str(), config.control_port)).await?;
        let scpi_addr = scpi_listener.local_addr()?;
        let control_addr = control_listener.local_addr()?;
        let pool = Arc::new(DevicePool::new());
        let (shutdown_tx, _shutdown_rx) = oneshot::channel();

        let pool_for_scpi = Arc::clone(&pool);
        let scpi_task = tokio::spawn(async move {
            if let Err(error) = serve_scpi(pool_for_scpi, scpi_listener).await {
                tracing::error!("SCPI server error: {error}");
            }
        });

        let pool_for_control = Arc::clone(&pool);
        let control_task = tokio::spawn(async move {
            if let Err(error) = serve_control(pool_for_control, control_listener).await {
                tracing::error!("control server error: {error}");
            }
        });

        info!("ProtoV mock server ready (scpi={scpi_addr}, control={control_addr})");

        Ok(Self {
            scpi_addr,
            control_addr,
            pool,
            shutdown: shutdown_tx,
            scpi_task,
            control_task,
        })
    }

    pub fn shutdown(self) {
        let _ = self.shutdown.send(());
        self.scpi_task.abort();
        self.control_task.abort();
    }
}

// Backwards-compatible wrapper used by main.
pub struct MockServer {
    pub config: ServerConfig,
    pub scpi_addr: SocketAddr,
    pub control_addr: SocketAddr,
    inner: Option<RunningServer>,
}

impl MockServer {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            scpi_addr: addr_from_config(&config.bind, config.scpi_port),
            control_addr: addr_from_config(&config.bind, config.control_port),
            config,
            inner: None,
        }
    }

    pub fn pool(&self) -> Arc<DevicePool> {
        self.inner
            .as_ref()
            .map(RunningServer::pool)
            .unwrap_or_else(|| Arc::new(DevicePool::new()))
    }

    pub async fn run(&mut self) -> std::io::Result<()> {
        let running = RunningServer::start(self.config.clone()).await?;
        self.scpi_addr = running.scpi_addr;
        self.control_addr = running.control_addr;
        self.inner = Some(running);
        futures_util::future::pending::<()>().await;
        Ok(())
    }

    pub fn shutdown(&mut self) {
        if let Some(inner) = self.inner.take() {
            inner.shutdown();
        }
    }
}

fn addr_from_config(bind: &str, port: u16) -> SocketAddr {
    format!("{bind}:{port}").parse().expect("valid socket addr")
}
