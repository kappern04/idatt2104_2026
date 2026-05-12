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
}
