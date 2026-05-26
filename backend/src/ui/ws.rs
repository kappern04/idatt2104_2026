//! WebSocket bridge between the Rust node and the browser frontend.
//!
//! The browser is treated as a read/write client:
//! - **Outbound** — after every op (local or from a peer) the bridge pushes a
//!   `Message::State { text }` frame so the browser can redraw without running RGA.
//! - **Inbound** — the browser sends `{ "type": "local_insert", … }` /
//!   `{ "type": "local_delete", … }` intents; the bridge converts them to CRDT
//!   ops via `Peer::browser_insert` / `Peer::browser_delete`.

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::Message as WsMsg};
use tracing::{info, warn};

use crate::network::peer::Peer;
use crate::network::protocol::Message;

/// Browser intent decoded from incoming WebSocket frames.
#[derive(serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum BrowserMsg {
    LocalInsert { offset: usize, ch: char },
    LocalDelete { offset: usize },
}

/// Bind a WebSocket listener on `port` and serve browser clients.
/// Each client connection is handled in its own Tokio task.
pub async fn serve(port: u16, peer: Peer) -> Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    info!("WebSocket UI bridge on 0.0.0.0:{port}");
    loop {
        let (stream, addr) = listener.accept().await?;
        info!("browser connected from {addr}");
        let peer = peer.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_client(stream, peer).await {
                warn!("browser {addr} disconnected: {e}");
            }
        });
    }
}

async fn handle_client(stream: tokio::net::TcpStream, peer: Peer) -> Result<()> {
    let ws = accept_async(stream).await?;
    let (mut sink, mut source) = ws.split();

    // Send the current document state immediately so the browser isn't blank.
    let init = serde_json::to_string(&Message::State {
        text: peer.text().await,
    })?;
    sink.send(WsMsg::Text(init)).await?;

    let mut ui_rx = peer.subscribe_ui();

    loop {
        tokio::select! {
            // Inbound: intent from the browser.
            frame = source.next() => {
                match frame {
                    None | Some(Err(_)) => break,
                    Some(Ok(WsMsg::Close(_))) => break,
                    Some(Ok(WsMsg::Text(text))) => {
                        match serde_json::from_str::<BrowserMsg>(&text) {
                            Ok(BrowserMsg::LocalInsert { offset, ch }) => {
                                if let Err(e) = peer.browser_insert(offset, ch).await {
                                    warn!("browser_insert failed: {e}");
                                }
                            }
                            Ok(BrowserMsg::LocalDelete { offset }) => {
                                if let Err(e) = peer.browser_delete(offset).await {
                                    warn!("browser_delete failed: {e}");
                                }
                            }
                            Err(e) => warn!("unknown browser message: {e}: {text}"),
                        }
                    }
                    _ => {}
                }
            }
            // Outbound: document changed — push new text to the browser.
            update = ui_rx.recv() => {
                match update {
                    Ok(text) => {
                        let msg = serde_json::to_string(&Message::State { text })?;
                        sink.send(WsMsg::Text(msg)).await?;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        // Fell behind — send current authoritative state.
                        let msg = serde_json::to_string(&Message::State {
                            text: peer.text().await,
                        })?;
                        sink.send(WsMsg::Text(msg)).await?;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }
    Ok(())
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use crate::crdt::sequence::{Char, Id, Op};
    use crate::network::peer::Peer;

    #[tokio::test]
    async fn browser_insert_appends_char() {
        let p = Peer::new(1);
        p.browser_insert(0, 'a').await.unwrap();
        assert_eq!(p.text().await, "a");
    }

    #[tokio::test]
    async fn browser_insert_at_offset_uses_correct_anchor() {
        let p = Peer::new(1);
        p.browser_insert(0, 'a').await.unwrap(); // "a"
        p.browser_insert(1, 'c').await.unwrap(); // "ac"
        p.browser_insert(1, 'b').await.unwrap(); // "abc"
        assert_eq!(p.text().await, "abc");
    }

    #[tokio::test]
    async fn browser_delete_removes_char() {
        let p = Peer::new(1);
        p.browser_insert(0, 'a').await.unwrap();
        p.browser_insert(1, 'b').await.unwrap();
        p.browser_delete(0).await.unwrap();
        assert_eq!(p.text().await, "b");
    }

    #[tokio::test]
    async fn browser_delete_out_of_range_is_noop() {
        let p = Peer::new(1);
        p.browser_insert(0, 'x').await.unwrap();
        p.browser_delete(99).await.unwrap(); // no panic
        assert_eq!(p.text().await, "x");
    }

    #[tokio::test]
    async fn ui_tx_fires_on_local_op() {
        let p = Peer::new(1);
        let mut rx = p.subscribe_ui();
        p.browser_insert(0, 'z').await.unwrap();
        let text = rx.try_recv().unwrap();
        assert_eq!(text, "z");
    }

    #[tokio::test]
    async fn ui_tx_fires_on_remote_op() {
        use crate::network::protocol::Message;
        let p = Peer::new(1);
        let mut rx = p.subscribe_ui();
        p.remote_op(Message::Op {
            from: 2,
            seq: 0,
            op: Op::Insert {
                after: None,
                ch: Char {
                    id: Id {
                        peer_id: 2,
                        counter: 1,
                    },
                    value: 'r',
                    deleted: false,
                },
            },
        })
        .await;
        let text = rx.try_recv().unwrap();
        assert_eq!(text, "r");
    }
}
