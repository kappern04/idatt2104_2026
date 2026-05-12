# RustCRDT

> **IDATT2104 — Networked & Distributed Programming, spring 2026**
> Peer-to-peer collaborative text editor built around custom Conflict-free
> Replicated Data Types (CRDTs) in Rust.

## Introduction

`rustcrdt` is a small distributed system that demonstrates strong eventual
consistency. Multiple peers each hold a local replica of the same document;
they edit it concurrently, exchange operations over the network, tolerate
disconnections, and provably converge to identical state — without any leader
or central server.

## Repository layout

The repo is split into two top-level folders by responsibility. The **backend
is the graded artefact**; the frontend is an optional visual demo that talks
to a node over WebSocket.

```
idatt2104_2026/
├── Cargo.toml          # workspace manifest (Rust)
├── README.md
├── docs/
│   └── Detailed plan.md
│
├── backend/            # ── PRIMARY DELIVERABLE ──
│   │                   #    Rust peer node: CRDTs, networking, persistence.
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs         # CLI entry point
│   │   ├── lib.rs          # public crate root
│   │   ├── crdt/           # G-Counter, OR-Set, RGA
│   │   ├── network/        # peer transport + wire protocol
│   │   ├── storage/        # append-only op-log persistence
│   │   └── ui/             # CLI + WebSocket-to-frontend bridge
│   └── tests/              # property + integration tests
│
└── frontend/           # ── OPTIONAL DEMO CLIENT ──
    │                   #    Static HTML/JS, no build step.
    │                   #    Connects to a node over WebSocket and renders ops.
    ├── index.html
    ├── app.js
    └── styles.css
```

The two halves are deliberately decoupled: the backend has no notion of the
frontend, and the frontend treats the node as just another peer endpoint over
the wire protocol defined in `backend/src/network/protocol.rs`.

## Implemented functionality

> Items marked ⏳ are scaffolded but not yet implemented.

- ✅ Cargo workspace, module layout, dependency wiring
- ✅ G-Counter CRDT + commutativity / associativity / idempotency tests
- ✅ OR-Set CRDT with tombstones + concurrent add/remove test
- ⏳ RGA sequence CRDT
- ⏳ Async TCP/WebSocket peer transport with reconnection
- ⏳ JSON-Lines persistence + replay on startup
- ⏳ WebSocket bridge to the browser frontend
- ⏳ CLI demo loop
- ⏳ Multi-peer integration tests with simulated delay

## Roadmap (deadline 26 May)

| Week | Goal |
|------|------|
| 1    | Finish RGA; all three CRDTs green with property tests |
| 2    | Networking + reconnection + integration tests |
| 3    | Persistence, frontend polish, write-up, demo recording |

## External dependencies

| Crate | Purpose |
|-------|---------|
| `tokio` | Async runtime for networking and concurrency |
| `serde` + `serde_json` | (De)serialise CRDT ops to JSON for the wire and the on-disk log |
| `tokio-tungstenite` | WebSocket transport between peers and toward the UI |
| `clap` | CLI argument parsing |
| `tracing` + `tracing-subscriber` | Structured logs for the demo |
| `anyhow` + `thiserror` | Ergonomic error handling |
| `uuid` | Unique tags inside the OR-Set |
| `proptest` (dev) | Property-based CRDT axiom tests |

No CRDT logic is borrowed from existing crates — every algorithm in `crdt/` is
hand-written for this assignment.

## Installation

Requires Rust 1.75+ (install via [rustup](https://rustup.rs)).

```pwsh
git clone <repo-url>
cd idatt2104_2026
cargo build --workspace
```

## Running a 3-peer demo

```pwsh
# terminal 1
cargo run -p rustcrdt-node -- --port 9001 --ui-port 8001 --peer-id 1

# terminal 2
cargo run -p rustcrdt-node -- --port 9002 --ui-port 8002 --peer-id 2 --connect 127.0.0.1:9001

# terminal 3
cargo run -p rustcrdt-node -- --port 9003 --ui-port 8003 --peer-id 3 --connect 127.0.0.1:9001 --connect 127.0.0.1:9002
```

Open `frontend/index.html` three times and point each to a node's
`ws://127.0.0.1:80xx` URL to see live convergence.

## Running tests

```pwsh
cargo test --workspace
```

Integration tests that need the networking layer are `#[ignore]`-d until
week 2.

## Known limitations / future work

- **Unbounded tombstones.** OR-Set and RGA never garbage-collect deletes;
  long-running systems need causal stability tracking.
- **No authentication / encryption** between peers — out of scope.
- **No conflict-free cursors** — only document text converges, not selections.
- **Peer discovery is manual** (via `--connect`); mDNS would be a nice extension.
- **No vector clocks**: the RGA design uses `after` anchors + per-peer
  monotonic counters, which is sufficient for convergence but doesn't expose
  full causal history.

## Documentation

```pwsh
cargo doc --workspace --open
```

Design discussion lives in `docs/Detailed plan.md`, and later in
`docs/architecture.md` and `docs/report.md`.

## Acknowledgements

CRDT theory references (no code copied):
- Marc Shapiro et al., *Conflict-free Replicated Data Types* (2011).
- Martin Kleppmann, *CRDTs: The Hard Parts* (2020 lecture).

---

## Original Assignment Description

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
