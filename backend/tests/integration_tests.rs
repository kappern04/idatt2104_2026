//! End-to-end multi-peer convergence tests.
//!
//! Spawns several in-process peers, drives concurrent edits through their
//! public handles (no real sockets — we test the CRDT/networking glue, not
//! the OS network stack), and asserts every replica's document is identical.
//!
//! Gated on the network module landing in week 2.

#[test]
#[ignore]
fn three_peers_converge_after_concurrent_edits() {
    // TODO(week 2): implement once network::peer is in place.
}
