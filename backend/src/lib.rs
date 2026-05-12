//! RustCRDT — a peer-to-peer CRDT library used by the collaborative editor node.
//!
//! The crate is split into:
//!
//! - [`crdt`]    : pure, side-effect-free CRDT implementations (G-Counter, OR-Set, RGA).
//! - [`network`] : async TCP / WebSocket transport between peers and toward a UI client.
//! - [`storage`] : append-only operation log persistence.
//! - [`ui`]      : CLI and WebSocket-to-frontend bridge.
//!
//! Public API is intentionally small — most types live behind `pub mod` boundaries
//! so internals can evolve without breaking integration tests.

pub mod crdt;
pub mod network;
pub mod storage;
pub mod ui;

/// Globally unique identifier of a peer/replica.
///
/// Using a `u64` keeps wire messages compact; in a real system a UUID would be safer
/// (collision-free without coordination). Documented as a known limitation.
pub type PeerId = u64;
