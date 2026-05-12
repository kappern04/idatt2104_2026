# Frontend

Static web client for a `rustcrdt-node`. Open `index.html` directly in a
browser, or serve the folder with any static file server, e.g.:

```pwsh
python -m http.server 5173
```

Then connect to a running node's UI WebSocket port (default `ws://127.0.0.1:8001`).

The client is intentionally dumb — it forwards keystrokes as *intent* messages
and renders whatever ops the node sends back. All CRDT logic stays in Rust.

