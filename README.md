# RustCRDT — Peer-to-Peer Collaborative Text Editor

> IDATT2104 — Networked & Distributed Programming, spring 2026

[![CI](https://github.com/kappern04/idatt2104_2026/actions/workflows/ci.yml/badge.svg)](https://github.com/kappern04/idatt2104_2026/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/kappern04/idatt2104_2026/branch/main/graph/badge.svg)](https://codecov.io/gh/kappern04/idatt2104_2026)
[![Release](https://github.com/kappern04/idatt2104_2026/actions/workflows/release.yml/badge.svg)](https://github.com/kappern04/idatt2104_2026/actions/workflows/release.yml)

- **Latest CI run:** <https://github.com/kappern04/idatt2104_2026/actions/workflows/ci.yml>
- **Latest release / CD run:** <https://github.com/kappern04/idatt2104_2026/actions/workflows/release.yml>
- **API documentation:** <https://kappern04.github.io/idatt2104_2026/>

---

## Introduction

RustCRDT is a peer-to-peer collaborative text editor where multiple nodes edit
the same document concurrently, disconnect and reconnect freely, and always
converge to identical state — with no central server and no coordination
required. It is implemented in Rust and built around three hand-written
Conflict-free Replicated Data Types (CRDTs).

**Why CRDTs instead of locks or a central server?**
Traditional approaches to shared mutable state (mutexes, databases, operational
transformation) require either coordination or a single authority. Both
properties are hard to preserve under network partitions. CRDTs sidestep the
problem entirely: every operation is designed so that merging two replicas
always produces the same result regardless of the order or number of times
operations arrive. This gives *strong eventual consistency* — a weaker but more
available guarantee than strong consistency.

**CAP theorem positioning.** RustCRDT favours *availability* and *partition
tolerance* over strong consistency. Each node accepts writes immediately and
never blocks waiting for other peers. Convergence is guaranteed eventually once
messages are delivered, but two peers reading concurrently may briefly see
different states. This is the right trade-off for a collaborative editor where
offline editing and resilience to disconnects matter more than instantaneous
agreement.

**Why three CRDTs?** The implementation progresses from simple to complex to
illustrate the design space: the G-Counter demonstrates state-based merge
semantics; the OR-Set introduces tombstones and the add-wins conflict resolution
strategy; the Replicated Growable Array (RGA) applies the same ideas to an
ordered sequence, which is what a text document actually requires.

---

## Implemented Functionality

- **G-Counter** — grow-only counter with per-peer slots and element-wise max
  merge. Demonstrates state-based CRDT fundamentals.
  (`backend/src/crdt/gcounter.rs`)

- **OR-Set** — observed-remove set with unique UUID tags and tombstones.
  Concurrent add-wins over remove; merge is a union of tag sets.
  (`backend/src/crdt/orset.rs`)

- **RGA sequence CRDT** — insert-after with tombstone deletes and
  deterministic tie-breaking for concurrent inserts at the same anchor. This is
  the CRDT that backs the collaborative document.
  (`backend/src/crdt/sequence.rs`)

- **Async TCP peer transport** — each node listens for incoming connections and
  dials out to peers supplied via `--connect`. Disconnected peers reconnect
  automatically with exponential backoff.
  (`backend/src/network/peer.rs`)

- **Sync catch-up** — when a peer reconnects it sends all ops it holds so the
  remote can apply any it missed while the connection was down.
  (`backend/src/network/protocol.rs`)

- **JSON-Lines op-log persistence** — every applied operation is appended to
  `operations.log`. On startup the node replays the log to restore state, making
  crash recovery identical to normal op delivery.
  (`backend/src/storage/persistence.rs`)

- **WebSocket bridge** — a second TCP port exposes a WebSocket endpoint for the
  browser frontend. The backend pushes a full `State { text }` snapshot after
  every op; the browser is intentionally kept dumb and never runs CRDT logic.
  (`backend/src/ui/ws.rs`)

- **CLI interface** — `insert`, `delete`, `text`, `peers`, `quit` commands over
  stdin. (`backend/src/ui/cli.rs`)

- **Static browser frontend** — `frontend/index.html` connects to any running
  node over WebSocket and shows live collaborative editing.

- **CI/CD pipeline** — `rustfmt`, `clippy -D warnings`, cross-platform tests
  (Ubuntu + Windows), branch coverage via `grcov` uploaded to Codecov,
  cross-platform release binaries, and auto-deployed rustdoc on GitHub Pages.

---

## Installation

Requires Rust 1.85 or newer. Install via [rustup](https://rustup.rs).

```pwsh
git clone https://github.com/kappern04/idatt2104_2026
cd idatt2104_2026
cargo build --workspace
```

---

## Usage

### Starting the nodes

Open three terminals and start one peer node each:

```pwsh
# terminal 1
cargo run -p rustcrdt-node -- --port 9001 --ui-port 8001 --peer-id 1

# terminal 2
cargo run -p rustcrdt-node -- --port 9002 --ui-port 8002 --peer-id 2 --connect 127.0.0.1:9001

# terminal 3
cargo run -p rustcrdt-node -- --port 9003 --ui-port 8003 --peer-id 3 --connect 127.0.0.1:9001 --connect 127.0.0.1:9002
```

Each node accepts CLI commands on stdin:

```text
insert hello world   # append text to the document
delete 0 5           # delete 5 characters starting at position 0
text                 # print the current document
peers                # show number of connected peers
quit                 # exit cleanly
```

### Terminal-only demo (no browser required)

This shows P2P convergence using only two terminals:

```pwsh
# terminal 1 — start peer 1
cargo run -p rustcrdt-node -- --port 9001 --ui-port 8001 --peer-id 1
```

```pwsh
# terminal 2 — start peer 2, connect to peer 1
cargo run -p rustcrdt-node -- --port 9002 --ui-port 8002 --peer-id 2 --connect 127.0.0.1:9001
```

In terminal 1, type:
```text
insert hello
```

In terminal 2, type:
```text
text
```

Terminal 2 will print `hello` — the document has converged without any browser
involved. Type `insert` on either side and run `text` on the other to keep
verifying convergence.

### Browser frontend — localhost

Serve the frontend folder and open one tab per node:

```pwsh
py      -m http.server 5173 --bind 127.0.0.1 --directory frontend
python3 -m http.server 5173 --bind 127.0.0.1 --directory frontend
```

Open `http://localhost:5173/index.html` and connect each tab to its node's
WebSocket: `ws://127.0.0.1:8001`, `ws://127.0.0.1:8002`, `ws://127.0.0.1:8003`.

### Browser frontend — LAN (laptop + phone on the same Wi-Fi)

**Step 1 — find your laptop's LAN IP**

```pwsh
# Windows
ipconfig | findstr "IPv4"

# macOS / Linux
ip route get 1 | awk '{print $7}'
```

Note the address starting with `192.168.x.x` or `10.x.x.x` — call it
`<LAPTOP_IP>`. On a phone Personal Hotspot the laptop gets a `172.20.10.x`
address — use that instead.

**Step 2 — serve the frontend on the LAN**

```pwsh
py      -m http.server 5173 --bind 0.0.0.0 --directory frontend
python3 -m http.server 5173 --bind 0.0.0.0 --directory frontend
# Node.js alternative
npx http-server frontend -a 0.0.0.0 -p 5173
```



| Device | URL to open |
|--------|-------------|
| Laptop | `http://127.0.0.1:5173/index.html` |
| Phone  | `http://<LAPTOP_IP>:5173/index.html` |

The WebSocket URL field auto-fills on the phone — click **Connect**.

---

## How to Run Tests

```pwsh
cargo test --workspace
```

This runs:

- **G-Counter tests** (`backend/tests/gcounter_tests.rs`) — commutativity,
  associativity, and idempotency verified by hand-crafted examples, plus a check
  that `value()` equals the sum of all peer slots.

- **OR-Set tests** (`backend/tests/orset_tests.rs`) — basic add/contains, the
  concurrent-add-wins-over-remove scenario, merge idempotency, and full
  `proptest`-based coverage of commutativity, associativity, and idempotency
  across randomised inputs.

- **Sequence (RGA) tests** (`backend/tests/sequence_tests.rs`) — 8 targeted
  unit tests (empty document, sequential inserts, tombstone anchors, convergence
  in all six orderings of three concurrent inserts) plus 3 `proptest` properties
  for commutativity, associativity, and idempotency. All enabled; none ignored.

- **Multi-peer integration tests** (`backend/tests/integration_tests.rs`) — 6
  tests covering: three-peer concurrent edits, duplicate op delivery, offline
  peer re-syncing via `Sync`, concurrent delete-vs-insert, network delay
  simulation, and a full disconnect-edit-reconnect scenario.

---

## External Dependencies

| Crate | Purpose |
|-------|---------|
| `tokio` | Async runtime — all networking and I/O runs on Tokio tasks |
| `serde` + `serde_json` | Serialise and deserialise CRDT ops to JSON for the wire protocol and the on-disk op-log |
| `tokio-tungstenite` | WebSocket transport between the node and the browser frontend |
| `clap` | CLI argument parsing (`--port`, `--connect`, `--peer-id`, etc.) |
| `tracing` + `tracing-subscriber` | Structured, levelled logging; log level controlled via `RUST_LOG` |
| `anyhow` | Ergonomic error propagation throughout the async code |
| `uuid` | Generate unique tags for OR-Set elements to distinguish concurrent adds |
| `futures-util` | Stream and sink utilities for the WebSocket connection handler |
| `proptest` *(dev)* | Property-based testing — generates hundreds of random inputs to verify CRDT axioms |

No CRDT algorithm is taken from an existing library — every implementation in
`backend/src/crdt/` is written from scratch for this assignment.

---

## API Documentation

Rustdoc for the latest tagged release is auto-deployed to GitHub Pages:

> <https://kappern04.github.io/idatt2104_2026/>

Build locally:

```pwsh
cargo doc --workspace --no-deps --open
```

---

## Future Work / Known Limitations

- **Unbounded tombstones.** Deleted entries in OR-Set and RGA are never removed.
  Long-running systems need causal stability tracking to know when a tombstone is
  safe to discard.
- **Fully-connected topology required.** Ops are not relayed between peers — each
  node must connect directly to every other node, or it will miss their edits.
  A gossip layer would remove this constraint.
- **No authentication or encryption** between peers. Acceptable for a local demo;
  out of scope for this assignment.
- **No conflict-free cursors.** Only the document text converges; cursor positions
  are not tracked.
- **Manual peer discovery.** Peers are supplied via `--connect`; automatic
  discovery (e.g. mDNS) is a natural extension.
- **No vector clocks.** The RGA design uses `after` anchors and per-peer monotonic
  counters, which is sufficient for convergence but does not expose full causal
  history.
- **PN-Counter not implemented.** A natural extension of the G-Counter; omitted
  because it adds no new theoretical insight beyond the three CRDTs already
  implemented.
- **Clean shutdown clears the document.** On a normal exit (`quit` or Ctrl-C)
  the op-log is truncated so the next session starts empty. Only a crash
  (unclean exit) preserves the log for replay on restart. This is intentional
  for the demo but a real editor would persist state across clean shutdowns.
- **No frontend tests.** The browser client is exercised manually only.

---

## External Information / References

No third-party code is copied into this repository. CRDT theory and algorithm
design draw on the following public material — all references are conceptual,
not code imports:

- Marc Shapiro, Nuno Preguiça, Carlos Baquero, Marek Zawirski —
  *Conflict-free Replicated Data Types*, INRIA Research Report RR-7687 (2011).
- Martin Kleppmann — *CRDTs: The Hard Parts* (2020 video lecture), referenced
  by the assignment brief.
- Hyun-Gul Roh, Myeongjae Jeon, Jin-Soo Kim, Joonwon Lee —
  *Replicated Abstract Data Types: Building Blocks for Collaborative
  Applications*, Journal of Parallel and Distributed Computing (2011) — the
  original RGA paper.
- The [Rust source-based coverage documentation](https://doc.rust-lang.org/rustc/instrument-coverage.html)
  for the CI coverage setup.
