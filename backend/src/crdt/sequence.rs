//! Phase 3 — Simplified Replicated Growable Array (RGA) for collaborative text.
//!
//! Each character is assigned a globally unique `Id { peer_id, counter }`. An
//! `Insert` carries the id of the character it should follow (`after`); ties
//! between concurrent inserts at the same anchor are broken by ordering the
//! new ids descending (`(peer_id, counter)`), which makes the result
//! independent of arrival order. Deletes set a tombstone bit so that the
//! `after` anchor of subsequent inserts remains valid forever.
//!
//! Convergence argument: two replicas that have applied the same set of
//! operations are identical because (1) `Insert` is idempotent on duplicate
//! ids, (2) the tie-break ordering is a total order on `Id` so concurrent
//! inserts at the same anchor produce the same relative sequence regardless of
//! arrival order, and (3) `Delete` only sets a boolean flag — applying it
//! multiple times is a no-op.

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

/// Outcome of [`Rga::apply`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyResult {
    /// The op was new and modified the document.
    Applied,
    /// The op was already present — idempotent no-op.
    Duplicate,
    /// The insert anchor or delete target has not arrived yet; retry after
    /// the missing op is applied.
    MissingAnchor,
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
    ///
    /// Returns [`ApplyResult::MissingAnchor`] when an `Insert`'s anchor or a
    /// `Delete`'s target is not yet in the document. The caller should retry
    /// once the missing op arrives (see `Message::Sync` handling in `peer.rs`).
    pub fn apply(&mut self, op: &Op) -> ApplyResult {
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
                if duplicate {
                    return ApplyResult::Duplicate;
                }

                let start = match after {
                    None => 0,
                    Some(anchor) => match self.chars.iter().position(|c| c.id == *anchor) {
                        Some(i) => i + 1,
                        None => return ApplyResult::MissingAnchor,
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
                ApplyResult::Applied
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
                if let Some(c) = self.chars.iter_mut().find(|c| c.id == *target) {
                    if c.deleted {
                        ApplyResult::Duplicate
                    } else {
                        c.deleted = true;
                        ApplyResult::Applied
                    }
                } else {
                    // Target not in doc yet — may be out of order in a Sync batch.
                    ApplyResult::MissingAnchor
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
