//! Multi-peer convergence tests (in-process, no real sockets).
//!
//! Tests use `Peer::remote_op` to simulate network delivery without real TCP,
//! keeping the tests fast and deterministic. The TCP path itself is covered by
//! `peer::tests::two_peers_exchange_ops_over_loopback`.

use rustcrdt::crdt::sequence::{Char, Id, Op};
use rustcrdt::network::peer::Peer;
use rustcrdt::network::protocol::Message;

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

    // Simulate delivery — deliberately different orderings to stress commutativity.
    p1.remote_op(Message::Op {
        from: 2,
        seq: 0,
        op: b.clone(),
    })
    .await;
    p1.remote_op(Message::Op {
        from: 3,
        seq: 0,
        op: c.clone(),
    })
    .await;

    p2.remote_op(Message::Op {
        from: 3,
        seq: 0,
        op: c.clone(),
    })
    .await;
    p2.remote_op(Message::Op {
        from: 1,
        seq: 0,
        op: a.clone(),
    })
    .await;

    // p3 receives in reverse order.
    p3.remote_op(Message::Op {
        from: 2,
        seq: 0,
        op: b.clone(),
    })
    .await;
    p3.remote_op(Message::Op {
        from: 1,
        seq: 0,
        op: a.clone(),
    })
    .await;

    let t1 = p1.text().await;
    let t2 = p2.text().await;
    let t3 = p3.text().await;

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

    // Both start with the same op.
    let op0 = ins(1, 1, 'a');
    online.local_op(op0.clone()).await.unwrap();
    offline
        .remote_op(Message::Op {
            from: 1,
            seq: 0,
            op: op0,
        })
        .await;

    // While offline is disconnected, online applies two more ops.
    let op1 = ins(1, 2, 'b');
    let op2 = ins(1, 3, 'c');
    online.local_op(op1.clone()).await.unwrap();
    online.local_op(op2.clone()).await.unwrap();

    // Offline reconnects and receives a Sync with the missed ops.
    offline
        .remote_op(Message::Sync {
            from: 1,
            ops: vec![op1, op2],
        })
        .await;

    assert_eq!(online.text().await, offline.text().await);
}
