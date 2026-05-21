//! Line-oriented stdin CLI.
//!
//! Commands:
//!   insert <text>     — append text to the end of the document
//!   delete <pos> <n>  — delete n characters starting at visible position pos
//!   text              — print the current document text
//!   peers             — show the number of connected peers
//!   quit  (or q)      — exit

use anyhow::Result;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::network::peer::Peer;

/// Process one command line against `peer`.
///
/// Returns `true` if the session should continue, `false` when the user types `quit`.
/// Uses `eprintln!` for prompts and errors (keeps stdout clean for piping `text` output).
pub async fn process(peer: &Peer, line: &str) -> Result<bool> {
    let line = line.trim();
    if line.is_empty() {
        return Ok(true);
    }

    let mut parts = line.splitn(2, ' ');
    let cmd = parts.next().unwrap_or("");
    let rest = parts.next().unwrap_or("").trim();

    match cmd {
        "insert" => {
            for ch in rest.chars() {
                let len = peer.text().await.chars().count();
                peer.browser_insert(len, ch).await?;
            }
        }
        "delete" => {
            let mut args = rest.split_whitespace();
            let pos: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or_else(|| {
                eprintln!("usage: delete <pos> <len>");
                0
            });
            let count: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or_else(|| {
                eprintln!("usage: delete <pos> <len>");
                0
            });
            for _ in 0..count {
                peer.browser_delete(pos).await?;
            }
        }
        "text" => println!("{}", peer.text().await),
        "peers" => {
            println!("{} peer(s) connected", peer.peers_connected());
            let counts = peer.op_counts().await;
            if !counts.is_empty() {
                println!("ops applied per peer (G-Counter):");
                for (pid, count) in counts {
                    println!("  peer {pid}: {count}");
                }
            }
        }
        "quit" | "q" => return Ok(false),
        _ => eprintln!("unknown command: {cmd}  (insert | delete | text | peers | quit)"),
    }
    Ok(true)
}

/// Read stdin line by line and dispatch each command until `quit` or EOF.
pub async fn run(peer: Peer) -> Result<()> {
    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    eprintln!("Commands: insert <text> | delete <pos> <len> | text | peers | quit");
    while let Some(line) = lines.next_line().await? {
        if !process(&peer, &line).await? {
            break;
        }
    }
    Ok(())
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::peer::Peer;

    #[tokio::test]
    async fn insert_appends_text_in_order() {
        let p = Peer::new(1);
        process(&p, "insert abc").await.unwrap();
        assert_eq!(p.text().await, "abc");
    }

    #[tokio::test]
    async fn insert_with_spaces_preserved() {
        let p = Peer::new(1);
        process(&p, "insert hello world").await.unwrap();
        assert_eq!(p.text().await, "hello world");
    }

    #[tokio::test]
    async fn delete_removes_range() {
        let p = Peer::new(1);
        process(&p, "insert hello").await.unwrap();
        process(&p, "delete 1 3").await.unwrap(); // removes "ell"
        assert_eq!(p.text().await, "ho");
    }

    #[tokio::test]
    async fn delete_out_of_range_is_noop() {
        let p = Peer::new(1);
        process(&p, "insert hi").await.unwrap();
        process(&p, "delete 99 5").await.unwrap();
        assert_eq!(p.text().await, "hi");
    }

    #[tokio::test]
    async fn text_command_does_not_quit() {
        let p = Peer::new(1);
        process(&p, "insert hi").await.unwrap();
        let cont = process(&p, "text").await.unwrap();
        assert!(cont);
    }

    #[tokio::test]
    async fn peers_command_does_not_quit() {
        let p = Peer::new(1);
        let cont = process(&p, "peers").await.unwrap();
        assert!(cont);
    }

    #[tokio::test]
    async fn quit_returns_false() {
        let p = Peer::new(1);
        assert!(!process(&p, "quit").await.unwrap());
        assert!(!process(&p, "q").await.unwrap());
    }

    #[tokio::test]
    async fn empty_line_continues() {
        let p = Peer::new(1);
        assert!(process(&p, "   ").await.unwrap());
    }

    #[tokio::test]
    async fn unknown_command_continues() {
        let p = Peer::new(1);
        assert!(process(&p, "frobnicate").await.unwrap());
    }
}
