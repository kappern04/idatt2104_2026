//! WebSocket bridge to the browser frontend.
//!
//! Frames are JSON, same `Message` enum as the peer protocol — that means the
//! frontend can be thought of as "just another peer that only renders".

// TODO(week 2): serve on `ui_port`, forward local ops out, forward incoming
// ops from the node into the socket as JSON `Message::Op`.

