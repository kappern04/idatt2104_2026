//! Wire-format definitions shared by peers and the UI bridge.

use serde::{Deserialize, Serialize};

use crate::crdt::sequence::Op;
use crate::PeerId;

/// Envelope around every message sent between peers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    /// Handshake announcing the sender's peer id.
    Hello { peer_id: PeerId },
    /// A single CRDT operation. Repeated delivery is fine — ops are idempotent.
    Op { from: PeerId, seq: u64, op: Op },
    /// Bulk catch-up: a vector of ops a reconnecting peer asks for.
    Sync { from: PeerId, ops: Vec<Op> },
}

