//! Phase 1 — Grow-only counter (state-based CRDT).
//!
//! Each peer owns its own slot and only it may increment that slot. The visible
//! counter value is the sum of all slots. Merging two counters takes the
//! element-wise maximum, which is trivially commutative, associative and
//! idempotent.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::PeerId;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GCounter {
    counts: HashMap<PeerId, u64>,
}

impl GCounter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Increment this peer's slot. Only the owning peer should call this.
    pub fn increment(&mut self, peer: PeerId, by: u64) {
        *self.counts.entry(peer).or_insert(0) += by;
    }

    /// Total value across all replicas observed so far.
    pub fn value(&self) -> u64 {
        self.counts.values().sum()
    }

    /// Element-wise max merge. Commutative, associative, idempotent.
    pub fn merge(&mut self, other: &GCounter) {
        for (peer, &v) in &other.counts {
            let slot = self.counts.entry(*peer).or_insert(0);
            if v > *slot {
                *slot = v;
            }
        }
    }
}
