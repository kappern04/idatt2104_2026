//! User-facing interfaces.
//!
//! * [`cli`] — line-oriented commands on stdin, useful for the recorded demo.
//! * [`ws`]  — WebSocket bridge consumed by the static frontend in `/frontend`.

pub mod cli;
pub mod ws;
