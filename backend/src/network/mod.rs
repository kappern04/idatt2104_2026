//! Peer-to-peer transport layer.
//!
//! The protocol is intentionally tiny: newline-delimited JSON over a TCP
//! WebSocket. Every node speaks the same protocol to every other node — there
//! is no leader. Duplicate messages are tolerated because CRDT ops are
//! idempotent.

pub mod peer;
pub mod protocol;
