//! OR-Set CRDT property tests.

use proptest::prelude::*;
use rustcrdt::crdt::orset::OrSet;

// ── helpers ───────────────────────────────────────────────────────────────────

/// Build an `OrSet<u8>` by replaying a sequence of (add?, value) operations.
fn build(ops: &[(bool, u8)]) -> OrSet<u8> {
    let mut s = OrSet::new();
    for &(is_add, val) in ops {
        if is_add {
            s.add(val);
        } else {
            s.remove(&val);
        }
    }
    s
}

// ── targeted unit tests ───────────────────────────────────────────────────────

#[test]
fn add_then_contains() {
    let mut s: OrSet<&'static str> = OrSet::new();
    s.add("a");
    assert!(s.contains(&"a"));
}

#[test]
fn concurrent_add_wins_over_remove() {
    // Replica A adds "x" and removes it. Replica B independently adds "x".
    // After merge "x" should still be in the set: B's tag was never observed
    // by A's remove.
    let mut a: OrSet<&'static str> = OrSet::new();
    a.add("x");
    a.remove(&"x");
    assert!(!a.contains(&"x"));

    let mut b: OrSet<&'static str> = OrSet::new();
    b.add("x");

    a.merge(&b);
    assert!(a.contains(&"x"));
}

#[test]
fn merge_is_idempotent() {
    let mut a: OrSet<&'static str> = OrSet::new();
    a.add("a");
    a.add("b");
    a.remove(&"a");
    let snapshot = a.clone();
    a.merge(&snapshot);
    assert_eq!(a.contains(&"a"), snapshot.contains(&"a"));
    assert_eq!(a.contains(&"b"), snapshot.contains(&"b"));
}

// ── property tests ────────────────────────────────────────────────────────────

proptest! {
    /// merge(A, B) == merge(B, A)
    ///
    /// `OrSet::merge` is a per-element union of UUID tag sets, so the result
    /// is the same regardless of which side is merged into which.
    #[test]
    fn prop_merge_is_commutative(
        ops_a in prop::collection::vec((any::<bool>(), any::<u8>()), 0..=5),
        ops_b in prop::collection::vec((any::<bool>(), any::<u8>()), 0..=5),
    ) {
        let a = build(&ops_a);
        let b = build(&ops_b);

        let mut ab = a.clone();
        ab.merge(&b);

        let mut ba = b.clone();
        ba.merge(&a);

        prop_assert_eq!(ab, ba);
    }

    /// merge(A, merge(B, C)) == merge(merge(A, B), C)
    #[test]
    fn prop_merge_is_associative(
        ops_a in prop::collection::vec((any::<bool>(), any::<u8>()), 0..=5),
        ops_b in prop::collection::vec((any::<bool>(), any::<u8>()), 0..=5),
        ops_c in prop::collection::vec((any::<bool>(), any::<u8>()), 0..=5),
    ) {
        let a = build(&ops_a);
        let b = build(&ops_b);
        let c = build(&ops_c);

        // merge(A, merge(B, C))
        let mut bc = b.clone();
        bc.merge(&c);
        let mut left = a.clone();
        left.merge(&bc);

        // merge(merge(A, B), C)
        let mut ab = a.clone();
        ab.merge(&b);
        ab.merge(&c);

        prop_assert_eq!(left, ab);
    }

    /// merge(A, A) == A
    #[test]
    fn prop_merge_is_idempotent(
        ops in prop::collection::vec((any::<bool>(), any::<u8>()), 0..=5),
    ) {
        let a = build(&ops);
        let mut merged = a.clone();
        merged.merge(&a);
        prop_assert_eq!(merged, a);
    }
}
