//! Multi-peer convergence tests.
//!
//! These tests use `Peer::remote_op` and `Peer::local_op` to simulate network
//! delivery in-process — no real sockets needed. `tokio::time::sleep` is used
//! to model network delay and demonstrate that convergence holds regardless of
//! timing. The TCP/WebSocket path is covered separately by
//! `peer::tests::two_peers_exchange_ops_over_loopback`.
//!
//! All tests are enabled (no `#[ignore]`).

use std::time::Duration;

use rustcrdt::crdt::sequence::{Char, Id, Op};
use rustcrdt::network::peer::Peer;
use rustcrdt::network::protocol::Message;

// ── helpers ───────────────────────────────────────────────────────────────────

/// Insert `value` at the beginning of the document (after sentinel / no anchor).
fn ins(peer_id: u64, counter: u64, value: char) -> Op {
    Op::Insert {
        after: None,
        ch: Char {
            id: Id { peer_id, counter },
            value,
            deleted: false,
        },
    }
}

/// Insert `value` after an existing character identified by `(after_peer, after_ctr)`.
fn ins_after(after_peer: u64, after_ctr: u64, peer_id: u64, counter: u64, value: char) -> Op {
    Op::Insert {
        after: Some(Id {
            peer_id: after_peer,
            counter: after_ctr,
        }),
        ch: Char {
            id: Id { peer_id, counter },
            value,
            deleted: false,
        },
    }
}

/// Tombstone the character identified by `(peer_id, counter)`.
fn del(peer_id: u64, counter: u64) -> Op {
    Op::Delete {
        target: Id { peer_id, counter },
    }
}

/// Deliver `ops` to `peer` as `Message::Op` frames (simulates network receipt).
async fn deliver(peer: &Peer, from: u64, ops: &[Op]) {
    for (seq, op) in ops.iter().enumerate() {
        peer.remote_op(Message::Op {
            from,
            seq: seq as u64,
            op: op.clone(),
        })
        .await;
    }
}

// ── existing tests (kept from issue #7) ──────────────────────────────────────

/// Three peers each generate a concurrent insert; ops are delivered in
/// different orders to each replica. All must converge to the same text.
#[tokio::test]
async fn three_peers_converge_after_concurrent_edits() {
    let p1 = Peer::new(1);
    let p2 = Peer::new(2);
    let p3 = Peer::new(3);

    let a = ins(1, 1, 'a');
    let b = ins(2, 1, 'b');
    let c = ins(3, 1, 'c');

    p1.local_op(a.clone()).await.unwrap();
    p2.local_op(b.clone()).await.unwrap();
    p3.local_op(c.clone()).await.unwrap();

    // Different delivery orderings stress commutativity.
    deliver(&p1, 2, &[b.clone()]).await;
    deliver(&p1, 3, &[c.clone()]).await;
    deliver(&p2, 3, &[c.clone()]).await;
    deliver(&p2, 1, &[a.clone()]).await;
    deliver(&p3, 2, &[b.clone()]).await;
    deliver(&p3, 1, &[a.clone()]).await;

    let (t1, t2, t3) = (p1.text().await, p2.text().await, p3.text().await);
    assert_eq!(t1, t2, "p1 and p2 diverged: {t1:?} vs {t2:?}");
    assert_eq!(t2, t3, "p2 and p3 diverged: {t2:?} vs {t3:?}");
    assert_eq!(t1.len(), 3, "expected 3 chars, got: {t1:?}");
}

/// A duplicate op delivered twice must not corrupt the document.
#[tokio::test]
async fn duplicate_op_delivery_is_idempotent() {
    let p = Peer::new(1);
    let msg = Message::Op {
        from: 2,
        seq: 0,
        op: ins(2, 1, 'x'),
    };
    p.remote_op(msg.clone()).await;
    p.remote_op(msg).await;
    assert_eq!(p.text().await, "x");
}

/// A peer that disconnects and reconnects receives ops applied while offline
/// via a Sync message and converges.
#[tokio::test]
async fn offline_peer_converges_after_sync() {
    let online = Peer::new(1);
    let offline = Peer::new(2);

    let op0 = ins(1, 1, 'a');
    online.local_op(op0.clone()).await.unwrap();
    deliver(&offline, 1, &[op0]).await;

    let op1 = ins(1, 2, 'b');
    let op2 = ins(1, 3, 'c');
    online.local_op(op1.clone()).await.unwrap();
    online.local_op(op2.clone()).await.unwrap();

    offline
        .remote_op(Message::Sync {
            from: 1,
            ops: vec![op1, op2],
        })
        .await;

    assert_eq!(online.text().await, offline.text().await);
}

// ── new tests ─────────────────────────────────────────────────────────────────

/// Concurrent delete-vs-insert at the same anchor: one peer deletes a char
/// while another inserts immediately after it. Both must converge.
///
/// Setup:  all peers share "ac"  (a = Id(1,1), c = Id(1,2) after Id(1,1))
/// p1 inserts 'b'(1,3) after 'a'(1,1)  → local: "abc"
/// p2 deletes 'a'(1,1)                  → local: "c"
/// After cross-delivery both must have "bc".
#[tokio::test]
async fn concurrent_delete_and_insert_converge() {
    let p1 = Peer::new(1);
    let p2 = Peer::new(2);

    // Establish shared initial state "ac".
    let op_a = ins(1, 1, 'a');
    let op_c = ins_after(1, 1, 1, 2, 'c');
    for op in &[op_a, op_c] {
        p1.local_op(op.clone()).await.unwrap();
        deliver(&p2, 1, &[op.clone()]).await;
    }
    assert_eq!(p1.text().await, "ac");
    assert_eq!(p2.text().await, "ac");

    // Concurrent ops: p1 inserts, p2 deletes at the same anchor.
    let insert_b = ins_after(1, 1, 1, 3, 'b'); // insert 'b' after 'a'
    let delete_a = del(1, 1); // tombstone 'a'

    p1.local_op(insert_b.clone()).await.unwrap(); // p1: "abc"
    p2.local_op(delete_a.clone()).await.unwrap(); // p2: "c"

    // Cross-deliver.
    deliver(&p1, 2, &[delete_a]).await; // p1 now: "bc"
    deliver(&p2, 1, &[insert_b]).await; // p2 now: "bc"

    let (t1, t2) = (p1.text().await, p2.text().await);
    assert_eq!(t1, t2, "delete-vs-insert diverged: {t1:?} vs {t2:?}");
    assert_eq!(t1, "bc");
}

/// Inserting after a tombstoned anchor must still work — the anchor stays in
/// the log forever so subsequent ops can use it.
#[tokio::test]
async fn insert_after_tombstone_anchor_is_stable() {
    let p1 = Peer::new(1);
    let p2 = Peer::new(2);

    let op_a = ins(1, 1, 'a');
    let op_b = ins_after(1, 1, 1, 2, 'b');

    for op in &[op_a.clone(), op_b.clone()] {
        p1.local_op(op.clone()).await.unwrap();
        deliver(&p2, 1, &[op.clone()]).await;
    }

    // p1 deletes 'a'; p2 inserts 'x' after 'a' (concurrently).
    let delete_a = del(1, 1);
    let insert_x = ins_after(1, 1, 2, 1, 'x');

    p1.local_op(delete_a.clone()).await.unwrap();
    p2.local_op(insert_x.clone()).await.unwrap();

    deliver(&p1, 2, &[insert_x]).await;
    deliver(&p2, 1, &[delete_a]).await;

    let (t1, t2) = (p1.text().await, p2.text().await);
    assert_eq!(t1, t2, "tombstone-anchor test diverged: {t1:?} vs {t2:?}");
    // 'a' deleted, 'x' inserted after it, 'b' after 'a' originally.
    // All visible: 'x' and 'b' (x has peer_id=2 > anchor's next, ordering may vary).
    assert_eq!(t1.len(), 2);
    assert!(t1.contains('x') && t1.contains('b'));
}

/// `tokio::time::sleep` between deliveries models network latency. Convergence
/// must hold regardless of timing.
#[tokio::test]
async fn simulated_network_delay_does_not_prevent_convergence() {
    let p1 = Peer::new(1);
    let p2 = Peer::new(2);
    let p3 = Peer::new(3);

    let a = ins(1, 1, 'a');
    let b = ins(2, 1, 'b');
    let c = ins(3, 1, 'c');

    // Each peer applies its own op with a small delay between them.
    p1.local_op(a.clone()).await.unwrap();
    tokio::time::sleep(Duration::from_millis(5)).await;
    p2.local_op(b.clone()).await.unwrap();
    tokio::time::sleep(Duration::from_millis(5)).await;
    p3.local_op(c.clone()).await.unwrap();

    // Deliver with delays — each peer receives ops in a different order.
    tokio::time::sleep(Duration::from_millis(10)).await;
    deliver(&p1, 3, &[c.clone()]).await; // p1 gets c before b
    tokio::time::sleep(Duration::from_millis(5)).await;
    deliver(&p1, 2, &[b.clone()]).await;

    deliver(&p2, 1, &[a.clone()]).await;
    tokio::time::sleep(Duration::from_millis(5)).await;
    deliver(&p2, 3, &[c.clone()]).await;

    deliver(&p3, 2, &[b.clone()]).await;
    tokio::time::sleep(Duration::from_millis(5)).await;
    deliver(&p3, 1, &[a.clone()]).await;

    let (t1, t2, t3) = (p1.text().await, p2.text().await, p3.text().await);
    assert_eq!(t1, t2, "delay test p1/p2 diverged: {t1:?} vs {t2:?}");
    assert_eq!(t2, t3, "delay test p2/p3 diverged: {t2:?} vs {t3:?}");
    assert_eq!(t1.len(), 3);
}

/// A peer disconnects mid-session, keeps editing locally, then reconnects and
/// receives all missed ops via a Sync message. All three peers must converge.
#[tokio::test]
async fn peer_disconnects_mid_edit_and_reconverges() {
    let pa = Peer::new(1); // always online
    let pb = Peer::new(2); // always online
    let pc = Peer::new(3); // goes offline, then reconnects

    // Phase 1 — everyone starts with "hi".
    let op_h = ins(1, 1, 'h');
    let op_i = ins_after(1, 1, 1, 2, 'i');
    for op in &[op_h.clone(), op_i.clone()] {
        pa.local_op(op.clone()).await.unwrap();
        deliver(&pb, 1, std::slice::from_ref(op)).await;
        deliver(&pc, 1, std::slice::from_ref(op)).await;
    }
    assert_eq!(pa.text().await, "hi");
    assert_eq!(pb.text().await, "hi");
    assert_eq!(pc.text().await, "hi");

    // Phase 2 — pc goes offline. pa and pb keep editing.
    let op_pa1 = ins_after(1, 2, 1, 3, '!'); // pa appends '!' → "hi!"
    let op_pb1 = ins(2, 1, 'x'); // pb inserts 'x' at start → "xhi" locally
    pa.local_op(op_pa1.clone()).await.unwrap();
    pb.local_op(op_pb1.clone()).await.unwrap();

    // pa and pb exchange ops with each other (but not pc).
    deliver(&pa, 2, &[op_pb1.clone()]).await;
    deliver(&pb, 1, &[op_pa1.clone()]).await;

    // pc edits while offline.
    let op_pc1 = ins(3, 1, 'y'); // pc inserts 'y' at start → "yhi" locally
    pc.local_op(op_pc1.clone()).await.unwrap();

    // Phase 3 — pc reconnects. It receives pa's and pb's missed ops via Sync.
    pc.remote_op(Message::Sync {
        from: 0,
        ops: vec![op_pa1, op_pb1.clone()],
    })
    .await;

    // pa and pb receive pc's offline op.
    deliver(&pa, 3, &[op_pc1.clone()]).await;
    deliver(&pb, 3, &[op_pc1]).await;

    let (ta, tb, tc) = (pa.text().await, pb.text().await, pc.text().await);
    assert_eq!(ta, tb, "pa and pb diverged: {ta:?} vs {tb:?}");
    assert_eq!(tb, tc, "pb and pc diverged: {tb:?} vs {tc:?}");
    // "hi" + '!' + 'x' at start + 'y' at start = 5 chars total.
    assert_eq!(ta.len(), 5, "expected 5 chars, got: {ta:?}");
    assert!(
        ta.contains('h')
            && ta.contains('i')
            && ta.contains('!')
            && ta.contains('x')
            && ta.contains('y'),
        "missing expected chars in: {ta:?}"
    );
}

/// Each of three peers applies multiple sequential ops; all are delivered in
/// scrambled order. The final document must be identical on every replica.
#[tokio::test]
async fn three_peers_multiple_ops_per_peer_converge() {
    let p1 = Peer::new(1);
    let p2 = Peer::new(2);
    let p3 = Peer::new(3);

    // p1 builds "ab", p2 builds "cd", p3 builds "ef" — all after None so they
    // race concurrently with each other.
    let ops1 = vec![ins(1, 1, 'a'), ins(1, 2, 'b')];
    let ops2 = vec![ins(2, 1, 'c'), ins(2, 2, 'd')];
    let ops3 = vec![ins(3, 1, 'e'), ins(3, 2, 'f')];

    for op in &ops1 {
        p1.local_op(op.clone()).await.unwrap();
    }
    for op in &ops2 {
        p2.local_op(op.clone()).await.unwrap();
    }
    for op in &ops3 {
        p3.local_op(op.clone()).await.unwrap();
    }

    // Deliver all ops to all peers in different orderings.
    deliver(&p1, 3, &ops3).await;
    deliver(&p1, 2, &ops2).await;

    deliver(&p2, 1, &ops1).await;
    deliver(&p2, 3, &ops3).await;

    deliver(&p3, 2, &ops2).await;
    deliver(&p3, 1, &ops1).await;

    let (t1, t2, t3) = (p1.text().await, p2.text().await, p3.text().await);
    assert_eq!(t1, t2, "multiple-ops p1/p2 diverged: {t1:?} vs {t2:?}");
    assert_eq!(t2, t3, "multiple-ops p2/p3 diverged: {t2:?} vs {t3:?}");
    assert_eq!(t1.len(), 6, "expected 6 chars, got: {t1:?}");
    for ch in ['a', 'b', 'c', 'd', 'e', 'f'] {
        assert!(t1.contains(ch), "missing '{ch}' in {t1:?}");
    }
}
