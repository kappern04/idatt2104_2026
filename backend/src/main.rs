//! Entry point for a single peer node.
//!
//! Each invocation = one replica. Multiple peers are started on different ports and
//! told about each other via `--connect`. The node:
//!   1. loads (or creates) a persistent op-log,
//!   2. opens a TCP/WebSocket listener for peer traffic,
//!   3. opens a WebSocket bridge for an optional browser frontend,
//!   4. accepts CLI commands on stdin.

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "rustcrdt-node", about = "Peer-to-peer CRDT editor node")]
struct Cli {
    /// Port to listen on for peer-to-peer traffic.
    #[arg(long, default_value_t = 9001)]
    port: u16,

    /// Port to expose the WebSocket bridge for the web frontend.
    #[arg(long, default_value_t = 8001)]
    ui_port: u16,

    /// Other peers to connect to, e.g. `127.0.0.1:9002`. Repeatable.
    #[arg(long = "connect")]
    connect: Vec<String>,

    /// Stable peer id. If omitted a random one is generated.
    #[arg(long)]
    peer_id: Option<u64>,

    /// Path to the on-disk operation log.
    #[arg(long, default_value = "operations.log")]
    log_path: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    tracing::info!(?cli, "starting rustcrdt-node");

    // TODO(week 2): wire crdt + network + storage + ui together here.
    //   1. let store = storage::OpLog::open(&cli.log_path)?;
    //   2. let node  = network::Peer::new(peer_id, store).await?;
    //   3. node.connect_all(cli.connect).await?;
    //   4. tokio::join!(node.listen(cli.port), ui::ws::serve(cli.ui_port, node.handle()), ui::cli::run(node.handle()));

    println!("rustcrdt-node skeleton — replace with real wiring (see TODOs in main.rs).");
    Ok(())
}

