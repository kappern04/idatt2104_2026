//! CRDT implementations.
//!
//! Each submodule exposes a self-contained data type that satisfies the three
//! mathematical CRDT properties tested under `tests/`:
//!
//! * **commutativity** — `merge(a, b) == merge(b, a)`
//! * **associativity** — `merge(a, merge(b, c)) == merge(merge(a, b), c)`
//! * **idempotency**  — `merge(a, a) == a`
//!
//! These properties are what guarantee *strong eventual consistency*: any two
//! replicas that have observed the same set of updates are byte-for-byte equal,
//! regardless of delivery order or duplication.

pub mod gcounter;
pub mod orset;
pub mod sequence;
