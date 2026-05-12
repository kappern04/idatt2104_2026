//! RGA / sequence CRDT tests.
//!
//! These are intentionally `#[ignore]` until phase 3 is implemented so that CI
//! stays green during development. Re-enable when `Rga::apply` is filled in.

use rustcrdt::crdt::sequence::Rga;

#[test]
#[ignore]
fn empty_doc_renders_as_empty_string() {
    let r = Rga::new();
    assert_eq!(r.text(), "");
}

