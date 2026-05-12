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
    pub fn apply(&mut self, _op: &Op) {
        // TODO: implement insert-after with tie-breaking on (peer_id, counter)
        // and delete-as-tombstone. Idempotency: if the id already exists, skip.
        todo!("RGA::apply — phase 3 milestone");
    }
}
