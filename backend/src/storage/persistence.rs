//! Append-only JSON-Lines operation log.
//!
//! Each applied op is serialised as one JSON line and flushed to disk.
//! On startup `OpLog::load` replays every line into a fresh `Rga`, restoring
//! the document exactly as it was before the process exited.
//!
//! Idempotency comes for free: `Rga::apply` silently skips ops whose `Id` is
//! already in the document, so replaying a file that contains duplicates (e.g.
//! from a crash mid-write) is safe.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::crdt::sequence::Op;

/// A handle to an open op-log file, positioned at the end for appending.
pub struct OpLog {
    path: PathBuf,
    file: File,
}

impl OpLog {
    /// Open (or create) the log at `path` in append mode.
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .await
            .with_context(|| format!("opening op-log {path:?}"))?;
        Ok(Self { path, file })
    }

    /// Serialise `op` as a JSON line and flush to disk.
    pub async fn append(&mut self, op: &Op) -> Result<()> {
        let mut line = serde_json::to_string(op)?;
        line.push('\n');
        self.file
            .write_all(line.as_bytes())
            .await
            .with_context(|| format!("writing to op-log {:?}", self.path))?;
        self.file.flush().await?;
        Ok(())
    }

    /// Truncate the log to zero bytes.
    ///
    /// Call this on clean shutdown so the next session starts with an empty
    /// document instead of replaying the current session's ops. The file is
    /// kept (not deleted) so `OpLog::load` on the next startup returns an
    /// empty `Vec` rather than treating a missing file as a fresh start.
    pub async fn clear(&mut self) -> Result<()> {
        // Reopen with truncate rather than calling set_len — more reliable on
        // Windows where set_len on an append-mode handle can be restricted.
        self.file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.path)
            .await
            .with_context(|| format!("clearing op-log {:?}", self.path))?;
        Ok(())
    }

    /// Read every op from `path` in order.
    ///
    /// - Returns an empty `Vec` if the file does not exist (fresh node).
    /// - Skips blank lines.
    /// - Logs a warning and skips lines that fail to parse (corrupt entries
    ///   must not prevent the remaining log from loading).
    pub async fn load(path: impl AsRef<Path>) -> Result<Vec<Op>> {
        let path = path.as_ref();
        let file = match File::open(path).await {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(e).with_context(|| format!("opening op-log {path:?}")),
        };

        let mut ops = Vec::new();
        let mut lines = BufReader::new(file).lines();
        while let Some(line) = lines.next_line().await? {
            let line = line.trim().to_owned();
            if line.is_empty() {
                continue;
            }
            match serde_json::from_str::<Op>(&line) {
                Ok(op) => ops.push(op),
                Err(e) => tracing::warn!("skipping malformed log entry: {e}"),
            }
        }
        Ok(ops)
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crdt::sequence::{Char, Id, Rga};

    fn test_path(id: &str) -> PathBuf {
        std::env::temp_dir().join(format!("rustcrdt_oplog_{id}.log"))
    }

    fn ins(peer_id: u64, counter: u64, value: char) -> Op {
        Op::Insert {
            after: None,
            ch: Char {
                id: Id { peer_id, counter },
                value,
                deleted: false,
            },
        }
    }

    #[tokio::test]
    async fn append_and_load_round_trip() {
        let path = test_path("round_trip");
        let _ = tokio::fs::remove_file(&path).await;

        let op1 = ins(1, 1, 'a');
        let op2 = ins(1, 2, 'b');

        let mut log = OpLog::open(&path).await.unwrap();
        log.append(&op1).await.unwrap();
        log.append(&op2).await.unwrap();
        drop(log);

        let loaded = OpLog::load(&path).await.unwrap();
        assert_eq!(loaded.len(), 2);

        let mut rga = Rga::new();
        for op in &loaded {
            rga.apply(op);
        }
        // id(1,2) > id(1,1), both after None → 'b' sorts before 'a'
        assert_eq!(rga.text(), "ba");

        tokio::fs::remove_file(&path).await.unwrap();
    }

    #[tokio::test]
    async fn missing_file_returns_empty_vec() {
        let path = test_path("nonexistent_xyz_abc");
        let ops = OpLog::load(&path).await.unwrap();
        assert!(ops.is_empty());
    }

    #[tokio::test]
    async fn replay_is_idempotent_with_duplicate_entries() {
        let path = test_path("idempotent");
        let _ = tokio::fs::remove_file(&path).await;

        let op = ins(1, 1, 'x');

        let mut log = OpLog::open(&path).await.unwrap();
        log.append(&op).await.unwrap();
        log.append(&op).await.unwrap(); // intentional duplicate
        drop(log);

        let loaded = OpLog::load(&path).await.unwrap();
        assert_eq!(loaded.len(), 2); // both lines are in the file…

        let mut rga = Rga::new();
        for op in &loaded {
            rga.apply(op); // …but apply skips the second (same Id)
        }
        assert_eq!(rga.text(), "x");

        tokio::fs::remove_file(&path).await.unwrap();
    }

    #[tokio::test]
    async fn state_survives_simulated_restart() {
        let path = test_path("restart");
        let _ = tokio::fs::remove_file(&path).await;

        // Simulate first run — write three ops.
        {
            let mut log = OpLog::open(&path).await.unwrap();
            log.append(&ins(1, 1, 'h')).await.unwrap();
            log.append(&ins(1, 2, 'i')).await.unwrap();
            log.append(&ins(1, 3, '!')).await.unwrap();
        } // log dropped — file closed

        // Simulate restart — replay into a fresh Rga.
        let ops = OpLog::load(&path).await.unwrap();
        assert_eq!(ops.len(), 3);

        let mut rga = Rga::new();
        for op in ops {
            rga.apply(&op);
        }
        let text = rga.text();
        assert_eq!(text.len(), 3);
        assert!(text.contains('h') && text.contains('i') && text.contains('!'));

        tokio::fs::remove_file(&path).await.unwrap();
    }

    #[tokio::test]
    async fn empty_file_returns_empty_vec() {
        let path = test_path("empty_file");
        let _ = tokio::fs::remove_file(&path).await;

        // Create an empty file.
        OpLog::open(&path).await.unwrap();

        let ops = OpLog::load(&path).await.unwrap();
        assert!(ops.is_empty());

        tokio::fs::remove_file(&path).await.unwrap();
    }

    #[tokio::test]
    async fn clear_empties_the_log() {
        let path = test_path("clear");
        let _ = tokio::fs::remove_file(&path).await;

        let mut log = OpLog::open(&path).await.unwrap();
        log.append(&ins(1, 1, 'a')).await.unwrap();
        log.append(&ins(1, 2, 'b')).await.unwrap();
        log.clear().await.unwrap();
        drop(log);

        // The file still exists but is empty — load returns no ops.
        let ops = OpLog::load(&path).await.unwrap();
        assert!(ops.is_empty());

        tokio::fs::remove_file(&path).await.unwrap();
    }
}
