//! RGA / sequence CRDT tests.
//!
//! Covers the three CRDT axioms (commutativity, associativity, idempotency)
//! plus tombstone and convergence behaviour, using both targeted unit tests
//! and proptest-based property tests.

use proptest::prelude::*;
use rustcrdt::crdt::sequence::{Char, Id, Op, Rga};

// ── helpers ──────────────────────────────────────────────────────────────────

fn id(peer_id: u64, counter: u64) -> Id {
    Id { peer_id, counter }
}

fn mk_insert(after: Option<Id>, peer_id: u64, counter: u64, value: char) -> Op {
    Op::Insert {
        after,
        ch: Char { id: id(peer_id, counter), value, deleted: false },
    }
}

fn mk_delete(peer_id: u64, counter: u64) -> Op {
    Op::Delete { target: id(peer_id, counter) }
}

fn apply_all(ops: &[Op]) -> Rga {
    let mut r = Rga::new();
    for op in ops {
        r.apply(op);
    }
    r
}

// ── unit tests ────────────────────────────────────────────────────────────────

#[test]
fn empty_doc_renders_as_empty_string() {
    assert_eq!(Rga::new().text(), "");
}

#[test]
fn single_insert_at_start() {
    let r = apply_all(&[mk_insert(None, 1, 1, 'a')]);
    assert_eq!(r.text(), "a");
}

#[test]
fn sequential_inserts_build_string() {
    let ops = [
        mk_insert(None, 1, 1, 'a'),
        mk_insert(Some(id(1, 1)), 1, 2, 'b'),
        mk_insert(Some(id(1, 2)), 1, 3, 'c'),
    ];
    assert_eq!(apply_all(&ops).text(), "abc");
}

#[test]
fn delete_tombstones_char_and_hides_it() {
    let ops = [
        mk_insert(None, 1, 1, 'a'),
        mk_insert(Some(id(1, 1)), 1, 2, 'b'),
        mk_delete(1, 1),
    ];
    assert_eq!(apply_all(&ops).text(), "b");
}

#[test]
fn insert_is_idempotent() {
    let op = mk_insert(None, 1, 1, 'a');
    let mut r = Rga::new();
    r.apply(&op);
    r.apply(&op);
    assert_eq!(r.text(), "a");
}

#[test]
fn delete_is_idempotent() {
    let del = mk_delete(1, 1);
    let mut r = apply_all(&[mk_insert(None, 1, 1, 'a')]);
    r.apply(&del);
    r.apply(&del);
    assert_eq!(r.text(), "");
}

#[test]
fn tombstone_anchor_remains_valid_for_later_insert() {
    // Delete 'a', then insert 'c' after the (now-deleted) 'a' anchor.
    // Tombstones must never be removed from the log for this to work.
    let ops = [
        mk_insert(None, 1, 1, 'a'),
        mk_insert(Some(id(1, 1)), 1, 2, 'b'),
        mk_delete(1, 1),
        mk_insert(Some(id(1, 1)), 1, 3, 'c'), // after deleted 'a'
    ];
    // Sequence: [a(deleted), c(1,3), b(1,2)] — (1,3) > (1,2) so c comes first
    assert_eq!(apply_all(&ops).text(), "cb");
}

/// Core convergence test: two replicas apply the same ops in opposite order and
/// must produce identical text. This is the commutativity axiom for RGA.
#[test]
fn concurrent_inserts_at_same_anchor_converge() {
    let anchor = mk_insert(None, 1, 1, 'x');
    // op_a id=(1,2), op_b id=(2,1). Since peer_id 2 > 1, op_b sorts first.
    let op_a = mk_insert(Some(id(1, 1)), 1, 2, 'a');
    let op_b = mk_insert(Some(id(1, 1)), 2, 1, 'b');

    let r1 = apply_all(&[anchor.clone(), op_a.clone(), op_b.clone()]);
    let r2 = apply_all(&[anchor.clone(), op_b.clone(), op_a.clone()]);

    assert_eq!(r1.text(), r2.text());
    assert_eq!(r1.text(), "xba"); // (2,1) > (1,2) so 'b' precedes 'a'
}

/// All six orderings of three concurrent inserts must yield the same text.
/// This covers both commutativity and associativity for the op-based model.
#[test]
fn three_concurrent_inserts_converge_in_all_orderings() {
    // Disjoint peer_ids guarantee distinct ids and a clear total order: p3>p2>p1.
    let op1 = mk_insert(None, 1, 1, 'a');
    let op2 = mk_insert(None, 2, 1, 'b');
    let op3 = mk_insert(None, 3, 1, 'c');

    let orderings: [&[&Op]; 6] = [
        &[&op1, &op2, &op3],
        &[&op1, &op3, &op2],
        &[&op2, &op1, &op3],
        &[&op2, &op3, &op1],
        &[&op3, &op1, &op2],
        &[&op3, &op2, &op1],
    ];

    let texts: Vec<String> = orderings
        .iter()
        .map(|order| {
            let mut r = Rga::new();
            for op in *order {
                r.apply(op);
            }
            r.text()
        })
        .collect();

    // All orderings must agree.
    for t in &texts {
        assert_eq!(t, &texts[0]);
    }
    // p3(3) > p2(2) > p1(1) so the sort order is c, b, a.
    assert_eq!(texts[0], "cba");
}

// ── property tests ────────────────────────────────────────────────────────────

proptest! {
    /// A single insert applied twice must look the same as applied once.
    #[test]
    fn prop_insert_idempotent(p in 1u64..=10, c in 1u64..=100, ch in 'a'..='z') {
        let op = mk_insert(None, p, c, ch);
        let once = apply_all(&[op.clone()]);
        let twice = {
            let mut r = Rga::new();
            r.apply(&op);
            r.apply(&op);
            r
        };
        prop_assert_eq!(once.text(), twice.text());
    }

    /// Two concurrent inserts must produce the same text regardless of order.
    #[test]
    fn prop_concurrent_inserts_commute(
        p1 in 1u64..=4,  c1 in 1u64..=10, ch1 in 'a'..='m',
        p2 in 5u64..=8,  c2 in 1u64..=10, ch2 in 'n'..='z',
    ) {
        let op1 = mk_insert(None, p1, c1, ch1);
        let op2 = mk_insert(None, p2, c2, ch2);

        let r1 = apply_all(&[op1.clone(), op2.clone()]);
        let r2 = apply_all(&[op2.clone(), op1.clone()]);

        prop_assert_eq!(r1.text(), r2.text());
    }

    /// Three concurrent inserts must converge in all six orderings.
    #[test]
    fn prop_three_concurrent_inserts_associative(
        p1 in 1u64..=3,  c1 in 1u64..=10,
        p2 in 4u64..=6,  c2 in 1u64..=10,
        p3 in 7u64..=9,  c3 in 1u64..=10,
        ch1 in 'a'..='i', ch2 in 'j'..='r', ch3 in 's'..='z',
    ) {
        let op1 = mk_insert(None, p1, c1, ch1);
        let op2 = mk_insert(None, p2, c2, ch2);
        let op3 = mk_insert(None, p3, c3, ch3);

        let t123 = apply_all(&[op1.clone(), op2.clone(), op3.clone()]).text();
        let t132 = apply_all(&[op1.clone(), op3.clone(), op2.clone()]).text();
        let t213 = apply_all(&[op2.clone(), op1.clone(), op3.clone()]).text();
        let t231 = apply_all(&[op2.clone(), op3.clone(), op1.clone()]).text();
        let t312 = apply_all(&[op3.clone(), op1.clone(), op2.clone()]).text();
        let t321 = apply_all(&[op3.clone(), op2.clone(), op1.clone()]).text();

        prop_assert_eq!(&t123, &t132);
        prop_assert_eq!(&t123, &t213);
        prop_assert_eq!(&t123, &t231);
        prop_assert_eq!(&t123, &t312);
        prop_assert_eq!(&t123, &t321);
    }
}
