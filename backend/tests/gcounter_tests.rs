//! G-Counter CRDT property tests.
//!
//! Verifies the three fundamental CRDT axioms — these are the same tests every
//! state-based CRDT in this crate must pass.

use proptest::prelude::*;
use rustcrdt::crdt::gcounter::GCounter;

fn counter(slots: &[(u64, u64)]) -> GCounter {
    let mut g = GCounter::new();
    for &(p, v) in slots {
        g.increment(p, v);
    }
    g
}

#[test]
fn merge_is_commutative() {
    let mut a = counter(&[(1, 3), (2, 1)]);
    let mut b = counter(&[(1, 2), (3, 7)]);
    let a0 = a.clone();
    let b0 = b.clone();
    a.merge(&b0);
    b.merge(&a0);
    assert_eq!(a, b);
}

#[test]
fn merge_is_idempotent() {
    let a = counter(&[(1, 3), (2, 1)]);
    let mut b = a.clone();
    b.merge(&a);
    assert_eq!(a, b);
}

#[test]
fn merge_is_associative() {
    let a = counter(&[(1, 3)]);
    let b = counter(&[(2, 5)]);
    let c = counter(&[(1, 7), (3, 2)]);

    let mut left = a.clone();
    {
        let mut bc = b.clone();
        bc.merge(&c);
        left.merge(&bc);
    }

    let mut right = a.clone();
    right.merge(&b);
    right.merge(&c);

    assert_eq!(left, right);
}

#[test]
fn value_is_sum_of_slots() {
    let g = counter(&[(1, 3), (2, 5), (3, 2)]);
    assert_eq!(g.value(), 10);
}

// ── property tests ────────────────────────────────────────────────────────────

proptest! {
    /// merge(A, B) == merge(B, A)
    #[test]
    fn prop_merge_is_commutative(
        slots_a in prop::collection::vec((0u64..10, 0u64..=100), 0..=5),
        slots_b in prop::collection::vec((0u64..10, 0u64..=100), 0..=5),
    ) {
        let a = counter(&slots_a);
        let b = counter(&slots_b);

        let mut ab = a.clone();
        ab.merge(&b);

        let mut ba = b.clone();
        ba.merge(&a);

        prop_assert_eq!(ab, ba);
    }

    /// merge(A, merge(B, C)) == merge(merge(A, B), C)
    #[test]
    fn prop_merge_is_associative(
        slots_a in prop::collection::vec((0u64..10, 0u64..=100), 0..=5),
        slots_b in prop::collection::vec((0u64..10, 0u64..=100), 0..=5),
        slots_c in prop::collection::vec((0u64..10, 0u64..=100), 0..=5),
    ) {
        let a = counter(&slots_a);
        let b = counter(&slots_b);
        let c = counter(&slots_c);

        let mut bc = b.clone();
        bc.merge(&c);
        let mut left = a.clone();
        left.merge(&bc);

        let mut ab = a.clone();
        ab.merge(&b);
        ab.merge(&c);

        prop_assert_eq!(left, ab);
    }

    /// merge(A, A) == A
    #[test]
    fn prop_merge_is_idempotent(
        slots in prop::collection::vec((0u64..10, 0u64..=100), 0..=5),
    ) {
        let a = counter(&slots);
        let mut merged = a.clone();
        merged.merge(&a);
        prop_assert_eq!(merged, a);
    }
}
