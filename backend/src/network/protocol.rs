//! Wire-format definitions shared by peers and the UI bridge.
//!
//! Messages are serialised as newline-delimited JSON (JSON-Lines) over TCP.
//! Every field needed to reconstruct the op at the receiving end is included
//! so replicas can apply messages without any extra state lookup.

use serde::{Deserialize, Serialize};

use crate::crdt::sequence::Op;
use crate::PeerId;

/// Envelope around every message sent between peers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    /// Handshake sent immediately after a TCP connection is established.
    Hello { peer_id: PeerId },
    /// A single CRDT operation. Repeated delivery is harmless — ops are idempotent.
    Op { from: PeerId, seq: u64, op: Op },
    /// Bulk catch-up: a reconnecting peer sends all ops it holds so the remote
    /// can apply any it missed while the connection was down.
    Sync { from: PeerId, ops: Vec<Op> },
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crdt::sequence::{Char, Id};

    fn sample_op() -> Op {
        Op::Insert {
            after: None,
            ch: Char {
                id: Id {
                    peer_id: 1,
                    counter: 1,
                },
                value: 'a',
                deleted: false,
            },
        }
    }

    #[test]
    fn hello_round_trips() {
        let msg = Message::Hello { peer_id: 7 };
        let json = serde_json::to_string(&msg).unwrap();
        let back: Message = serde_json::from_str(&json).unwrap();
        assert!(matches!(back, Message::Hello { peer_id: 7 }));
    }

    #[test]
    fn op_message_round_trips() {
        let msg = Message::Op {
            from: 1,
            seq: 42,
            op: sample_op(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let back: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(json, serde_json::to_string(&back).unwrap());
    }

    #[test]
    fn sync_message_round_trips() {
        let msg = Message::Sync {
            from: 2,
            ops: vec![sample_op(), sample_op()],
        };
        let json = serde_json::to_string(&msg).unwrap();
        let back: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(json, serde_json::to_string(&back).unwrap());
    }

    #[test]
    fn newline_delimited_stream_parses_independently() {
        let m1 = Message::Hello { peer_id: 1 };
        let m2 = Message::Hello { peer_id: 2 };
        let stream = format!(
            "{}\n{}\n",
            serde_json::to_string(&m1).unwrap(),
            serde_json::to_string(&m2).unwrap()
        );
        let parsed: Vec<Message> = stream
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        assert_eq!(parsed.len(), 2);
        assert!(matches!(parsed[0], Message::Hello { peer_id: 1 }));
        assert!(matches!(parsed[1], Message::Hello { peer_id: 2 }));
    }
}
