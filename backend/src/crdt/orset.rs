//! Phase 2 — Observed-Remove Set.
//!
//! Each `add(x)` attaches a unique tag to the element. `remove(x)` records the
//! set of tags currently observed for `x` as *tombstones*. The element is in
//! the set iff it has at least one tag not yet tombstoned. This resolves the
//! classic add/remove race: a concurrent add wins because its tag was never
//! observed by the removing replica.
//!
//! Trade-off: tombstones grow without bound. Real systems compact them via
//! causal stability — discussed in the report.

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrSet<T: Eq + Hash + Clone> {
    /// element -> set of live tags
    adds: HashMap<T, HashSet<Uuid>>,
    /// element -> set of tombstoned tags
    removes: HashMap<T, HashSet<Uuid>>,
}

impl<T: Eq + Hash + Clone> OrSet<T> {
    pub fn new() -> Self {
        Self {
            adds: HashMap::new(),
            removes: HashMap::new(),
        }
    }

    pub fn add(&mut self, element: T) -> Uuid {
        let tag = Uuid::new_v4();
        self.adds.entry(element).or_default().insert(tag);
        tag
    }

    /// Remove every currently-observed tag for `element` (tombstone them).
    pub fn remove(&mut self, element: &T) {
        if let Some(tags) = self.adds.get(element).cloned() {
            self.removes
                .entry(element.clone())
                .or_default()
                .extend(tags);
        }
    }

    pub fn contains(&self, element: &T) -> bool {
        let live = self.adds.get(element);
        let dead = self.removes.get(element);
        match (live, dead) {
            (Some(a), Some(d)) => a.difference(d).next().is_some(),
            (Some(a), None) => !a.is_empty(),
            _ => false,
        }
    }

    pub fn merge(&mut self, other: &Self) {
        for (k, tags) in &other.adds {
            self.adds
                .entry(k.clone())
                .or_default()
                .extend(tags.iter().copied());
        }
        for (k, tags) in &other.removes {
            self.removes
                .entry(k.clone())
                .or_default()
                .extend(tags.iter().copied());
        }
    }

    /// Remove entries for elements that have no live tags from *this replica's*
    /// perspective (every locally-known add-tag is tombstoned).
    ///
    /// **Warning — not safe for production use.** Another replica may hold a
    /// live add-tag for the same element that this replica has not yet received.
    /// If that tag arrives after compaction the element resurfaces without a
    /// corresponding tombstone, violating remove-wins semantics. Safe compaction
    /// requires causal stability (a guarantee that no future `add` ops for this
    /// element can arrive). Retained here as a demo-only memory management
    /// helper; do not call during live replication.
    pub fn compact(&mut self) {
        let dead: Vec<T> = self
            .adds
            .iter()
            .filter(|(elem, add_tags)| {
                self.removes
                    .get(*elem)
                    .is_some_and(|removed| add_tags.is_subset(removed))
            })
            .map(|(elem, _)| elem.clone())
            .collect();
        for elem in &dead {
            self.adds.remove(elem);
            self.removes.remove(elem);
        }
    }
}
