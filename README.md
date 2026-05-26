# RustCRDT — Peer-to-Peer Collaborative Text Editor

> IDATT2104 — Networked programming, spring 2026

[![CI](https://github.com/kappern04/idatt2104_2026/actions/workflows/ci.yml/badge.svg)](https://github.com/kappern04/idatt2104_2026/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/kappern04/idatt2104_2026/branch/main/graph/badge.svg)](https://codecov.io/gh/kappern04/idatt2104_2026)
[![Release](https://github.com/kappern04/idatt2104_2026/actions/workflows/release.yml/badge.svg)](https://github.com/kappern04/idatt2104_2026/actions/workflows/release.yml)

- **Latest CI run:** <https://github.com/kappern04/idatt2104_2026/actions/workflows/ci.yml>
- **Latest release / CD run:** <https://github.com/kappern04/idatt2104_2026/actions/workflows/release.yml>
- **API documentation:** <https://kappern04.github.io/idatt2104_2026/>

---
## Preface
IDATT2104 network programming voluntary project. This project is intended only as a demonstration of CRDT in the course idatt2104.

Developer:
Kasper Østerlie Gladsøy

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
  (`backend/src/network/peer.rs`)

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

- **Static browser frontend** — `frontend/index.html` + `frontend/app.js` +
  `frontend/styles.css`. Connects to any running node over WebSocket and
  renders the shared document in real time. Features: connected/disconnected
  status indicator, an event log sidebar (logs connection lifecycle events and
  the document length on each received state update), and a WebSocket URL field
  that auto-fills from the page hostname so the app works on mobile without
  editing the URL manually.

- **CI/CD pipeline** — `rustfmt`, `clippy -D warnings`, cross-platform tests
  (Ubuntu + Windows), branch coverage via `grcov` uploaded to Codecov,
  cross-platform release binaries, and auto-deployed rustdoc on GitHub Pages.

---

## Installation

Requires Rust 1.85 or newer. Install via [rustup](https://rustup.rs).

The browser frontend is plain static HTML/JS — no build step required.
To serve it you need either the
[VS Code Live Server extension](https://marketplace.visualstudio.com/items?itemName=ritwickdey.LiveServer)
(local use only) **or** Python 3 (for LAN / mobile access). Python 3 ships
with macOS and most Linux distros; on Windows install it from
[python.org](https://www.python.org/downloads/) or the Microsoft Store.

```pwsh
git clone https://github.com/kappern04/idatt2104_2026
cd idatt2104_2026
cargo build --workspace
```

---

## Usage

### Running the nodes

Start each node in its own terminal. Use `--port` for peer-to-peer TCP
traffic and `--connect` to dial another peer on startup:

```pwsh
# terminal 1 — first peer, listens on port 9001
cargo run -p rustcrdt-node -- --port 9001 --peer-id 1 --log-path operations-1.log

# terminal 2 — second peer, connects to the first
cargo run -p rustcrdt-node -- --port 9002 --peer-id 2 --log-path operations-2.log --connect 127.0.0.1:9001

# terminal 3 — third peer, connects to both
cargo run -p rustcrdt-node -- --port 9003 --peer-id 3 --log-path operations-3.log --connect 127.0.0.1:9001 --connect 127.0.0.1:9002
```

> **Important:** each node must use a distinct `--log-path`. Without it all three
> processes write to the same `operations.log`, which corrupts replay on restart.

To run peers across multiple machines on the same network, replace
`127.0.0.1` in `--connect` with the LAN IP of the target machine
(e.g. `--connect 192.168.1.10:9001`).

### CLI commands

Each node reads commands from stdin:

```text
insert hello world   # append text to the end of the document
delete 0 5           # delete 5 characters starting at visible position 0
text                 # print the current document text
peers                # show connected peers and op counts per peer (G-Counter)
quit                 # exit cleanly
```

Type `insert hello` in terminal 1, then `text` in terminal 2 — both replicas
converge to the same document with no browser required.

---

### Optional — browser frontend

The `frontend/` folder contains a static web client that visualises the
document in real time. It connects to a node over WebSocket using the
`--ui-port` flag.

Start the nodes with `--ui-port` added:

```pwsh
cargo run -p rustcrdt-node -- --port 9001 --ui-port 8001 --peer-id 1 --log-path operations-1.log
cargo run -p rustcrdt-node -- --port 9002 --ui-port 8002 --peer-id 2 --log-path operations-2.log --connect 127.0.0.1:9001
cargo run -p rustcrdt-node -- --port 9003 --ui-port 8003 --peer-id 3 --log-path operations-3.log --connect 127.0.0.1:9001 --connect 127.0.0.1:9002
```

Serve the frontend — pick one option:

#### Option A — VS Code Live Server (local only, no Python needed)

Open the `frontend/` folder in VS Code, right-click `index.html`, and choose
**Open with Live Server**. The browser opens automatically on
`http://127.0.0.1:5500/index.html`. This option only works on the machine
running VS Code; other devices on the network cannot reach it.

#### Option B — Python (local + LAN / mobile)

```pwsh
# Windows
py -m http.server 5173 --bind 0.0.0.0 --directory frontend
```

```bash
# macOS / Linux
python3 -m http.server 5173 --bind 0.0.0.0 --directory frontend
```

**Opening the frontend:**

On the machine serving the frontend:
```text
# Python
http://localhost:5173/index.html
# Live Server   
http://127.0.0.1:5500/index.html  
```

On any other device on the same network, replace `localhost` with the LAN IP
of the machine serving the frontend. To find that IP, run on the host machine:

```pwsh
# Windows
ipconfig | findstr "IPv4"

# macOS / Linux
ip route get 1 | awk '{print $7}'
```


```text
http://192.168.1.10:5173/index.html
```

You will see a WebSocket field — enter the address of the node you want to
connect to.

**Is the node you are connecting to running on this device?**
Use `ws://127.0.0.1` followed by its `--ui-port`:
```text
ws://127.0.0.1:8001
```

**Is the node running on a different device on the network?**
Use that device's LAN IP followed by its `--ui-port`:
```text
ws://192.168.1.10:8001
```

---

## How to Run Tests

```pwsh
cargo test --workspace
```

This runs:

- **G-Counter tests** (`backend/tests/gcounter_tests.rs`) — commutativity,
  associativity, and idempotency verified by hand-crafted examples and 3
  `proptest` properties, plus a check that `value()` equals the sum of all peer
  slots.

- **OR-Set tests** (`backend/tests/orset_tests.rs`) — basic add/contains, the
  concurrent-add-wins-over-remove scenario, merge idempotency, and full
  `proptest`-based coverage of commutativity, associativity, and idempotency
  across randomised inputs.

- **Sequence (RGA) tests** (`backend/tests/sequence_tests.rs`) — 10 targeted
  unit tests (empty document, single and sequential inserts, insert idempotency,
  delete tombstoning, delete idempotency, tombstone-anchor stability, concurrent
  inserts at the same anchor, convergence in all six orderings of three
  concurrent inserts, and missing-anchor handling) plus 3 `proptest` properties
  for insert idempotency, commutativity, and associativity. All enabled; none
  ignored.

- **Multi-peer integration tests** (`backend/tests/integration_tests.rs`) — 10
  tests covering: three-peer concurrent edits, duplicate op delivery, offline
  peer re-syncing via `Sync`, concurrent delete-vs-insert, network delay
  simulation, a full disconnect-edit-reconnect scenario, insert-after-tombstone
  anchor stability, op-log replay with out-of-order entries, Sync buffering
  until a missing anchor arrives, and multiple ops per peer converging across
  three replicas.

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

## External Information / Code Use

No external CRDT libraries or copied CRDT implementations were used. The
CRDT implementations in `backend/src/crdt/` were written from scratch for this
assignment.

Conceptual background and inspiration:

- Course assignment text for IDATT2104, spring 2026.
- Martin Kleppmann, **CRDTs: The Hard Parts**:
  <https://martin.kleppmann.com/2020/07/06/crdt-hard-parts-hydra.html>

The Rust crate documentation for the dependencies listed above was used as API
reference only.

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

- **Unbounded tombstones.** Deleted entries in the RGA are never removed; the
  sequence grows monotonically. The OR-Set has a `compact()` method that
  discards fully-dead elements (every add-tag tombstoned), but it is not called
  automatically during normal operation. Compaction of partially-live elements
  in either structure requires causal stability tracking to be safe.
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
- **Log persists across all shutdowns.** The op-log is never truncated on exit
  (clean or otherwise), so every restart replays the full history. A real
  editor would add an explicit "reset" command and separate session state from
  durable history. `Peer::clear_log()` exists for this purpose but is not wired
  to any CLI command yet.
- **Offline mode.** A disconnected node already accepts local edits (ops are
  applied immediately and queued in the op-log); on reconnection the Sync
  mechanism delivers all accumulated ops so replicas converge. What is missing
  is an explicit offline-mode switch and better UX (e.g. a UI indicator when
  no peers are connected). There is a subtle interaction worth noting: if a
  peer receives a `Sync` that includes ops it originally generated in a
  previous session, its local ID counter must be advanced past those old values
  — otherwise new inserts silently produce duplicate IDs and have no visible
  effect. This is handled in `peer.rs` via `id_seq` advancement in `remote_op`.
- **Better frontend.** The browser client is exercised manually only. Pasting or deleting many
  characters at once makes weird animation and can offset characters. More functionality to the frontend such as disconnect button.

---

## CI/CD Pipeline

- **CI workflow:** <https://github.com/kappern04/idatt2104_2026/blob/main/.github/workflows/ci.yml>
- **Release workflow:** <https://github.com/kappern04/idatt2104_2026/blob/main/.github/workflows/release.yml>
