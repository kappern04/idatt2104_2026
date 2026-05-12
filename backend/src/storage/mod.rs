//! Persistence.
//!
//! We use an append-only JSON-Lines log of operations. On startup the node
//! replays the log to reconstruct the document — same code path as receiving
//! ops over the network, so persistence is "free" once apply is idempotent.

pub mod persistence;
