//! OR-Set CRDT property tests.

use rustcrdt::crdt::orset::OrSet;

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
