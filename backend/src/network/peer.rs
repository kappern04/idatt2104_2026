//! `Peer` owns the local document state and the connections to other peers.
//!
//! Design notes:
//!
//! * Each outgoing connection runs in its own Tokio task and pulls from a
//!   broadcast channel — applying an op locally fans out to every peer.
//! * Each incoming connection runs a reader task that pushes parsed ops onto a
//!   single mpsc into the node "core" task, which is the only place mutating
//!   the document. This avoids locks on the CRDT itself.
//! * Reconnection: a peer that drops is re-dialed with exponential backoff.
//!   On reconnect we send our highest-seen `seq` per peer and ask for missing
//!   ops — this is the offline-support story.

// TODO(week 2): implement Peer::new, listen, connect, broadcast, apply.
