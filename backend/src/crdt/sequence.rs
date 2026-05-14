//! Phase 3 — Simplified Replicated Growable Array (RGA) for collaborative text.
//!
//! Each character is assigned a globally unique `Id { peer_id, counter }`. An
//! `Insert` carries the id of the character it should follow (`after`); ties
//! between concurrent inserts at the same anchor are broken by ordering the
//! new ids descending (`(peer_id, counter)`), which makes the result
//! independent of arrival order. Deletes set a tombstone bit so that the
//! `after` anchor of subsequent inserts remains valid forever.
//!
//! See `docs/architecture.md` for the convergence argument.

use serde::{Deserialize, Serialize};

use crate::PeerId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Id {
    pub peer_id: PeerId,
    pub counter: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Char {
    pub id: Id,
    pub value: char,
    pub deleted: bool,
}

/// An operation that can be sent over the network. Idempotent on apply.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Op {
    Insert { after: Option<Id>, ch: Char },
    Delete { target: Id },
}

#[derive(Debug, Clone, Default)]
pub struct Rga {
    chars: Vec<Char>,
}

impl Rga {
    pub fn new() -> Self {
        Self::default()
    }

    /// Visible text (deleted entries skipped).
    pub fn text(&self) -> String {
        self.chars
            .iter()
            .filter(|c| !c.deleted)
            .map(|c| c.value)
            .collect()
    }

    /// Apply a remote or local op. Safe to call multiple times with the same op.
    pub fn apply(&mut self, op: &Op) {
        match op {
            Op::Insert { after, ch } => {
                let duplicate = self.chars.iter().any(|c| c.id == ch.id);
                tracing::debug!(
                    op_type = "insert",
                    peer_id = ch.id.peer_id,
                    counter = ch.id.counter,
                    payload = %ch.value,
                    after_peer    = after.map(|id| id.peer_id),
                    after_counter = after.map(|id| id.counter),
                    duplicate,
                    doc_len = self.chars.len(),
                    "rga_apply",
                );
                // Idempotency: skip duplicate ids.
                if duplicate {
                    return;
                }

                let start = match after {
                    None => 0,
                    Some(anchor) => match self.chars.iter().position(|c| c.id == *anchor) {
                        Some(i) => i + 1,
                        // Anchor not yet delivered; skip — replayed from op-log on reconnect.
                        None => return,
                    },
                };

                // Walk forward past concurrent inserts with a higher id (descending order).
                // This deterministic tie-break ensures all replicas reach the same position
                // regardless of delivery order.
                let mut pos = start;
                while pos < self.chars.len() && self.chars[pos].id > ch.id {
                    pos += 1;
                }

                self.chars.insert(pos, ch.clone());
            }

            Op::Delete { target } => {
                let entry = self.chars.iter().find(|c| c.id == *target);
                tracing::debug!(
                    op_type = "delete",
                    peer_id = target.peer_id,
                    counter = target.counter,
                    found = entry.is_some(),
                    tombstone = entry.map(|c| c.deleted).unwrap_or(false),
                    doc_len = self.chars.len(),
                    "rga_apply",
                );
                // Tombstone. Idempotent: already-deleted chars are unchanged.
                if let Some(c) = self.chars.iter_mut().find(|c| c.id == *target) {
                    c.deleted = true;
                }
            }
        }
    }

    /// The `Id` of the n-th visible (non-deleted) character, 0-indexed.
    /// Used by the WebSocket bridge to translate browser text offsets to CRDT anchors.
    pub fn id_at_visible(&self, offset: usize) -> Option<Id> {
        self.chars
            .iter()
            .filter(|c| !c.deleted)
            .nth(offset)
            .map(|c| c.id)
    }
}
