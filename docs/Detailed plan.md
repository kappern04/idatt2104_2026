# Assignment Translation (English)

## Assignment Description

Implement and optionally use one or more Conflict-free Replicated Data Types (CRDTs) for either a client-server or peer-to-peer architecture in a programming language of your choice.

You may either:

* create a simple proof-of-concept application, or
* develop a CRDT software library.

For more information and inspiration, see the lecture *“CRDTs: The Hard Parts”*. However, the lecture is 5 years old, and much may have changed in this field since then.

You should prioritize which functionalities to include in your solution.

A complete solution is **not expected**.

Implementation in more challenging programming languages such as C++ or Rust may positively influence the grade.

Using existing CRDT libraries or software made by others is **not allowed**.

You must create a `README.md` file for the solution. Include:

* Name of the solution and optionally a link to the latest CI/CD run
* Introduction
* Implemented functionality
* Future work including current limitations/weaknesses
* External dependencies with a short description of each dependency and its purpose
* Installation instructions
* Instructions for using the solution
* How to run tests
* Optional link to API documentation
* Any use of external information/code must be documented

---

# Deep Blueprint to Achieve an A

The key to getting an A is **not building the biggest system**.

The key is demonstrating:

* strong distributed systems understanding
* correct CRDT implementation
* clean architecture
* concurrency reasoning
* deterministic conflict resolution
* good testing/documentation
* thoughtful engineering tradeoffs

A polished smaller project beats a huge unfinished one.

---

# Recommended Project

# “RustCRDT” — Peer-to-Peer Collaborative Text Editor

## Why this project?

This project demonstrates:

* real distributed systems concepts
* concurrency handling
* networking
* synchronization
* custom CRDT implementation
* eventual consistency
* conflict resolution
* practical application

This looks significantly more advanced than:

* a simple counter
* toy examples
* isolated CRDT demos

It also gives many opportunities for technical discussion in the report.

---

# Final Goal

You will build:

## A distributed collaborative editor where:

* multiple peers edit the same document
* edits happen concurrently
* no central authority is required
* all replicas eventually converge
* network partitions are tolerated
* reconnecting peers synchronize automatically

---

# Technology Stack (Ideal for A-grade)

## Language

# Rust

Why:

* difficult language
* memory safety
* concurrency support
* strong systems-programming reputation
* explicitly rewarded in assignment text

---

# Core Architecture

# Peer-to-Peer (NOT client-server)

This gives significantly more academic value.

Each node:

* stores local document state
* executes local operations immediately
* broadcasts operations to peers
* merges remote operations deterministically

---

# Core CRDT Strategy

You should implement THREE CRDTs.

This demonstrates progression and understanding.

---

# Phase 1 — G-Counter

## Purpose

Demonstrate understanding of:

* state-based CRDTs
* merge semantics
* monotonic growth

## Structure

```rust
HashMap<PeerId, u64>
```

## Merge Rule

```text
take max value for each peer
```

## Why Important?

Easy to implement.
Shows understanding of CRDT fundamentals.

---

# Phase 2 — OR-Set (Observed Remove Set)

## Purpose

Demonstrate:

* add/remove conflict resolution
* tombstones
* causal reasoning

## Structure

```rust
HashSet<(Element, UniqueTag)>
```

## Features

* concurrent add/remove handling
* deterministic merge
* idempotency

---

# Phase 3 — Sequence/Text CRDT (MAIN PART)

This is where A-level work happens.

---

# Recommended Design

DO NOT implement a full advanced CRDT like:

* Automerge
* Yjs
* RGA++
* LogootSplit

Too large for assignment scope.

Instead implement:

# Simplified RGA (Replicated Growable Array)

This is realistic and manageable.

---

# Internal Data Model

## Character Identifier

```rust
struct Id {
    peer_id: u64,
    counter: u64,
}
```

Globally unique.

---

## Character Entry

```rust
struct Char {
    id: Id,
    value: char,
    deleted: bool,
}
```

---

## Document

```rust
Vec<Char>
```

Simple.
Easy to reason about.
Good enough academically.

---

# Operations

## Insert

```rust
Insert {
    after: Id,
    new_char: Char,
}
```

Insert after another character.

---

## Delete

```rust
Delete {
    target: Id,
}
```

Mark deleted using tombstones.

---

# Why Tombstones?

Critical concept.

Without tombstones:

* concurrent deletes break consistency
* replicas diverge

Tombstones show:

* understanding of distributed ordering
* causal stability tradeoffs

Professors love this discussion.

---

# Networking Layer

# TCP-based Peer Communication

Use:

* Tokio
* async Rust

Each peer:

* listens on TCP port
* connects to peers
* sends operations as JSON

---

# Message Format

```json
{
  "op": "insert",
  "after": [1, 5],
  "id": [2, 9],
  "char": "A"
}
```

---

# Why This Architecture Gets High Grades

Because it demonstrates:

| Concept              | Demonstrated |
| -------------------- | ------------ |
| Distributed systems  | YES          |
| CRDT theory          | YES          |
| Networking           | YES          |
| Concurrency          | YES          |
| Deterministic merge  | YES          |
| Eventual consistency | YES          |
| Fault tolerance      | YES          |
| Systems programming  | YES          |
| Rust complexity      | YES          |

---

# Eventual Consistency Demonstration

This MUST be emphasized.

You need to prove:

```text
All replicas converge
```

even if:

* operations arrive out of order
* messages are duplicated
* peers disconnect temporarily

This is one of the most important grading points.

---

# Recommended Advanced Features

These are HIGH VALUE for grades.

---

# 1. Offline Support

Allow peers to:

* disconnect
* continue editing
* reconnect later
* synchronize

This is extremely good academically.

---

# 2. Duplicate Message Handling

Operations may arrive multiple times.

Your merge logic should be:

* idempotent

Very important CRDT property.

---

# 3. Simulated Network Delay

Add artificial delay:

```rust
tokio::sleep(Duration::from_millis(500))
```

Then show:

* replicas still converge

Excellent demonstration.

---

# 4. Persistence

Save operations to disk.

Can be simple:

```json
operations.log
```

Shows engineering maturity.

---

# 5. Vector Clocks (BONUS)

Not required.

But VERY impressive if partially implemented.

Use for:

* causal ordering
* operation tracking

Even explaining why you avoided full vector clocks can score points.

---

# Suggested Folder Structure

```text
rust-crdt/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs
│   ├── crdt/
│   │   ├── gcounter.rs
│   │   ├── orset.rs
│   │   ├── sequence.rs
│   │   └── mod.rs
│   ├── network/
│   │   ├── peer.rs
│   │   ├── protocol.rs
│   │   └── mod.rs
│   ├── storage/
│   │   └── persistence.rs
│   └── ui/
│       └── cli.rs
├── tests/
│   ├── gcounter_tests.rs
│   ├── orset_tests.rs
│   ├── sequence_tests.rs
│   └── integration_tests.rs
└── docs/
    ├── architecture.md
    └── report.md
```

---

# The MOST IMPORTANT PART

# Testing

This is where many students lose grades.

You need strong tests.

---

# Required Tests

## 1. Merge Commutativity

```text
merge(A,B) == merge(B,A)
```

---

## 2. Merge Idempotency

```text
merge(A,A) == A
```

---

## 3. Associativity

```text
merge(A, merge(B,C))
==
merge(merge(A,B), C)
```

These are CORE CRDT properties.

---

# Integration Tests

Simulate:

* 3 peers
* concurrent edits
* reconnecting peer
* delayed messages

Then verify:

```text
all documents identical
```

THIS is A-level material.

---

# CLI Demo

Keep UI simple.

Example:

```bash
cargo run -- --port 9001
```

Then type:

```text
insert hello
```

Other peers synchronize automatically.

---

# What To Show During Presentation

## Demo Sequence

### Step 1

Start 3 peers.

### Step 2

Connect them.

### Step 3

Edit simultaneously.

### Step 4

Disconnect one peer.

### Step 5

Continue editing.

### Step 6

Reconnect peer.

### Step 7

Show automatic convergence.

This is extremely strong.

---

# README Quality

Most students underestimate this.

A-grade README should include:

* architecture diagrams
* CRDT explanation
* merge logic explanation
* examples
* screenshots
* known limitations
* future work
* testing explanation

---

# Report Topics That Impress Professors

---

# 1. Why CRDTs Instead of Locks?

Discuss:

* scalability
* offline editing
* availability
* distributed systems

---

# 2. Tradeoffs

Discuss:

* tombstone memory growth
* operation ordering
* synchronization cost
* eventual vs strong consistency

---

# 3. Why Simplified RGA?

Explain:

* assignment scope
* engineering tradeoffs
* complexity management

This shows maturity.

---

# 4. CAP Theorem Discussion

Very impressive if included briefly.

Explain:

* system prioritizes availability
* eventual consistency over strong consistency

---

# Timeline
Need to be done by 26th mai


---

# Biggest Mistakes To Avoid

## DO NOT:

* build GUI first
* overcomplicate algorithm
* attempt full Google Docs clone
* use existing CRDT libraries
* ignore testing
* skip networking
* skip documentation

---

# Final Recommendation

The strongest possible submission is:

## Rust + Peer-to-Peer + Sequence CRDT + Strong Tests

That combination hits:

* theoretical depth
* systems complexity
* practical engineering
* distributed systems concepts

which is exactly what typically earns top grades in this type of assignment.
