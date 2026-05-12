# RustCRDT

> **IDATT2104 — Networked & Distributed Programming, spring 2026**
> Peer-to-peer collaborative text editor built around custom Conflict-free
> Replicated Data Types (CRDTs) in Rust.

[![CI](https://github.com/kappern04/idatt2104_2026/actions/workflows/ci.yml/badge.svg)](https://github.com/kappern04/idatt2104_2026/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/kappern04/idatt2104_2026/branch/main/graph/badge.svg)](https://codecov.io/gh/kappern04/idatt2104_2026)
[![Release](https://github.com/kappern04/idatt2104_2026/actions/workflows/release.yml/badge.svg)](https://github.com/kappern04/idatt2104_2026/actions/workflows/release.yml)

- **Name of the solution:** *RustCRDT — Peer-to-Peer Collaborative Text Editor*
- **Repository:** <https://github.com/kappern04/idatt2104_2026>
- **Latest CI run:** <https://github.com/kappern04/idatt2104_2026/actions/workflows/ci.yml>
- **Latest release / CD run:** <https://github.com/kappern04/idatt2104_2026/actions/workflows/release.yml>
- **API documentation (rustdoc, auto-deployed):** <https://kappern04.github.io/idatt2104_2026/>

### Assignment requirement checklist

| Required README section | Where it lives |
|---|---|
| Name of the solution + link to latest CI/CD run | Top of this file (above) |
| Introduction | [`#introduction`](#introduction) |
| Implemented functionality | [`#implemented-functionality`](#implemented-functionality) |
| Future work / limitations | [`#future-work--known-limitations`](#future-work--known-limitations) |
| External dependencies (with purpose) | [`#external-dependencies`](#external-dependencies) |
| Installation instructions | [`#installation`](#installation) |
| Instructions for using the solution | [`#usage--instructions-for-using-the-solution`](#usage--instructions-for-using-the-solution) |
| How to run tests | [`#how-to-run-tests`](#how-to-run-tests) |
| API documentation link | [`#api-documentation`](#api-documentation) |
| Documentation of external information / code | [`#external-information--references`](#external-information--references) |

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

## Usage / Instructions for using the solution

A typical demo brings up three peer nodes on `localhost` and points one browser
tab at each:

```pwsh
# terminal 1
cargo run -p rustcrdt-node -- --port 9001 --ui-port 8001 --peer-id 1

# terminal 2
cargo run -p rustcrdt-node -- --port 9002 --ui-port 8002 --peer-id 2 --connect 127.0.0.1:9001

# terminal 3
cargo run -p rustcrdt-node -- --port 9003 --ui-port 8003 --peer-id 3 --connect 127.0.0.1:9001 --connect 127.0.0.1:9002
```

Open `frontend/index.html` three times and point each tab to a node's
`ws://127.0.0.1:80xx` URL to see live convergence as you type.

Once the network layer lands (week 2), the CLI also accepts commands on
stdin:

```text
insert hello world      # insert text at the end of the document
delete 0 5              # delete the first 5 characters
peers                   # list connected peers
text                    # print current document text
quit
```

## How to run tests

```pwsh
cargo test --workspace
```

This runs:

- **G-Counter property tests** (`backend/tests/gcounter_tests.rs`) —
  commutativity, associativity, idempotency, plus a sanity check that
  `value()` equals the sum of all slots.
- **OR-Set tests** (`backend/tests/orset_tests.rs`) — basic add/contains, the
  concurrent-add-wins-over-remove scenario, and merge idempotency. Full
  commutativity / associativity coverage is **planned** alongside the next
  CRDT iteration.
- **Sequence (RGA) test stub** (`backend/tests/sequence_tests.rs`) — currently
  `#[ignore]`-d; will be enabled once `Rga::apply` is implemented in phase 3.
- **Multi-peer integration test stub** (`backend/tests/integration_tests.rs`)
  — currently `#[ignore]`-d; will be filled in once the networking layer
  lands in phase 2.

Run everything including the ignored stubs with:

```pwsh
cargo test --workspace -- --include-ignored
```

## Branching model

The repo uses a lightweight GitFlow:

| Branch | Purpose |
|---|---|
| `main` | Release-ready code. Only updated via PR from `dev`. Tagged for releases. |
| `dev`  | Integration branch. All feature branches merge here via PR. |
| `<n>-<topic>` | Short-lived feature branches (e.g. `3-cicd`, `4-rga-apply`). |

**CI runs on every push and PR to both `main` and `dev`.** **CD (release
binaries + Pages deploy) runs only on tag pushes from `main`** — i.e. after a
`dev → main` PR is merged and a `vX.Y.Z` tag is pushed.

## Continuous integration & code coverage

CI runs on every push and pull request via GitHub Actions
([`.github/workflows/ci.yml`](.github/workflows/ci.yml)) and consists of:

1. **`rustfmt`** — formatting gate.
2. **`clippy`** — `-D warnings`, fails on any lint.
3. **`test`** — `cargo test --workspace` on Ubuntu *and* Windows.
4. **`test-analytics`** — `cargo nextest` run on Ubuntu that emits a JUnit XML
   report; uploaded to **Codecov Test Analytics** so flaky/slow tests surface
   in PR comments.
5. **`coverage`** — instrumented build + `grcov` aggregation, with line *and*
   branch coverage uploaded to Codecov and stored as a build artefact.

The choice between `cargo-tarpaulin` and `grcov` is documented as an ADR in
[`docs/ci-coverage-decision.md`](docs/ci-coverage-decision.md). Short version:
`grcov` wins because we need branch coverage on heavily-branching CRDT merge
code, the project is async/Tokio-heavy (where ptrace-based tools struggle),
and `grcov` works identically on the Windows dev machine and the Linux CI
runner.

## Continuous delivery

CD lives in [`.github/workflows/release.yml`](.github/workflows/release.yml)
and is **tag-driven** — pushing a tag like `v0.1.0` triggers:

1. **Cross-platform builds** of `rustcrdt-node` for
   `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`, and
   `x86_64-apple-darwin`.
2. **GitHub Release** publication with the three archives attached and
   auto-generated release notes.
3. **GitHub Pages** deployment of `cargo doc --workspace --no-deps`, so the
   latest API documentation is always reachable at
   `https://kappern04.github.io/idatt2104_2026/`.

Why tag-driven instead of per-commit? Building three OS targets on every
push would burn CI minutes for no benefit; tags are the conventional signal
that "this commit is a deployable version". The workflow also has a
`workflow_dispatch` trigger so it can be run manually from the Actions tab.

To cut a release:

```pwsh
git tag v0.1.0
git push origin v0.1.0
```

To reproduce the coverage report locally (matches the CI invocation):

```pwsh
$env:RUSTFLAGS = "-Cinstrument-coverage"
$env:LLVM_PROFILE_FILE = "rustcrdt-%p-%m.profraw"
rustup component add llvm-tools-preview
cargo install grcov --locked
cargo test --workspace
grcov . --binary-path ./target/debug/deps/ -s . -t html --branch `
  --ignore-not-existing --ignore 'target/*' --ignore '/*' -o coverage
start coverage/index.html
```

The `--ignore '/*'` pattern filters out absolute paths (rustc-installed
stdlib and registry crates that occasionally leak into source-based coverage
reports) so the percentage reflects this workspace only.

## Future work / known limitations

- **Unbounded tombstones.** OR-Set and RGA never garbage-collect deletes;
  long-running systems need causal stability tracking.
- **No authentication / encryption** between peers — out of scope.
- **No conflict-free cursors** — only document text converges, not selections.
- **Peer discovery is manual** (via `--connect`); mDNS would be a nice extension.
- **No vector clocks**: the RGA design uses `after` anchors + per-peer
  monotonic counters, which is sufficient for convergence but doesn't expose
  full causal history.
- **PN-Counter not implemented.** Natural extension of G-Counter; left as
  future work because it doesn't add new theoretical insight beyond the three
  CRDTs already covered.
- **No GUI tests.** The frontend is exercised manually only.

## API documentation

The rustdoc for the latest tagged release is auto-deployed to GitHub Pages:

> <https://kappern04.github.io/idatt2104_2026/>

To build it locally:

```pwsh
cargo doc --workspace --no-deps --open
```

Design notes live alongside the code in `docs/`:

- [`docs/Detailed plan.md`](docs/Detailed%20plan.md) — the original project plan.

## External information / references

No third-party code is copied into this repository. CRDT theory and design
draw on the following public material; everything cited is conceptual, not a
code import:

- Marc Shapiro, Nuno Preguiça, Carlos Baquero, Marek Zawirski — *Conflict-free
  Replicated Data Types*, INRIA Research Report (2011).
- Martin Kleppmann — *CRDTs: The Hard Parts* (2020 video lecture), referenced
  by the assignment brief.
- Hyun-Gul Roh et al. — *Replicated Abstract Data Types: Building Blocks for
  Collaborative Applications* (2011) — original RGA paper.
- The [Rust source-based coverage chapter](https://doc.rust-lang.org/rustc/instrument-coverage.html)
  for the CI coverage setup.

External *tools* (compilers, runtimes, CI actions) are listed under
[External dependencies](#external-dependencies); their licenses are
compatible with this project's MIT license.

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
