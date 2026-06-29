use clap::Parser;
use protov_hal_mock::{RunningServer, ServerConfig};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "protov-mock", about = "ProtoV MINI WebSocket simulator")]
struct Args {
    #[arg(long, default_value = "127.0.0.1")]
    bind: String,

    #[arg(long, env = "PROTOV_MOCK_SCPI_PORT", default_value_t = 8765)]
    scpi_port: u16,

    #[arg(long, env = "PROTOV_MOCK_CTRL_PORT", default_value_t = 8766)]
    control_port: u16,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("protov_hal_mock=info".parse().unwrap()),
        )
        .init();

    let args = Args::parse();
    let config = ServerConfig {
        bind: args.bind,
        scpi_port: args.scpi_port,
        control_port: args.control_port,
    };

    let server = RunningServer::start(config).await?;
    tracing::info!(
        scpi = %server.scpi_addr,
        control = %server.control_addr,
        "ProtoV mock server running; press Ctrl+C to exit"
    );

    tokio::signal::ctrl_c().await?;
    tracing::info!("shutting down");
    server.shutdown();
    Ok(())
}
