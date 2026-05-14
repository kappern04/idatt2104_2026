//! Entry point for a single peer node.
//!
//! Each invocation is one replica. Start multiple nodes on different ports and
//! connect them with `--connect`. CLI commands arrive on stdin (issue #9).

use clap::Parser;
use rustcrdt::network::peer::Peer;
use rustcrdt::storage::persistence::OpLog;
use rustcrdt::ui::{cli, ws};

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

    /// Stable peer id. If omitted a random one is generated from the current time.
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

    let peer_id = cli.peer_id.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as u64
    });

    tracing::info!(peer_id, port = cli.port, "starting rustcrdt-node");

    let peer = Peer::new(peer_id);

    // Spawn TCP listener.
    {
        let p = peer.clone();
        let port = cli.port;
        tokio::spawn(async move {
            if let Err(e) = p.listen(port).await {
                tracing::error!("listener failed: {e}");
            }
        });
    }

    // Spawn one outgoing-connection task per --connect address.
    for addr in cli.connect {
        let p = peer.clone();
        tokio::spawn(async move { p.connect(addr).await });
    }

    // Replay persisted ops before accepting connections.
    let prior_ops = OpLog::load(&cli.log_path).await?;
    if !prior_ops.is_empty() {
        tracing::info!("replaying {} ops from {}", prior_ops.len(), cli.log_path);
    }
    peer.replay_ops(prior_ops).await;
    let log = OpLog::open(&cli.log_path).await?;
    peer.set_log(log).await;

    // Spawn WebSocket UI bridge.
    {
        let p = peer.clone();
        let ui_port = cli.ui_port;
        tokio::spawn(async move {
            if let Err(e) = ws::serve(ui_port, p).await {
                tracing::error!("WebSocket bridge failed: {e}");
            }
        });
    }

    println!(
        "Node {peer_id} on :{} (UI :{}).  Commands: insert | delete | text | peers | quit",
        cli.port, cli.ui_port
    );

    // Run the CLI loop; also exit on Ctrl-C.
    tokio::select! {
        res = cli::run(peer.clone()) => {
            if let Err(e) = res { tracing::error!("CLI error: {e}"); }
        }
        _ = tokio::signal::ctrl_c() => {}
    }

    // Clear the log on clean shutdown so the next session starts with an empty
    // document. A crash (no clean shutdown) leaves the log intact for replay.
    peer.clear_log().await;
    tracing::info!("shutting down");
    Ok(())
}
