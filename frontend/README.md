# Frontend

Static web client for a `rustcrdt-node`. Open `index.html` directly in a
browser, or serve the folder with any static file server, e.g.:

```pwsh
# --bind 127.0.0.1 is required on Windows — without it Python binds to [::] (IPv6)
# and the browser receives http://[::]:5173/ which is invalid (ERR_ADDRESS_INVALID).
python  -m http.server 5173 --bind 127.0.0.1
# On Windows, if `python` is not on PATH:
py      -m http.server 5173 --bind 127.0.0.1
# On macOS / Linux if `python` invokes Python 2:
python3 -m http.server 5173 --bind 127.0.0.1
```

Then connect to a running node's UI WebSocket port (default `ws://127.0.0.1:8001`).

Alternative options:

- Node.js (if `npm` is installed):

```pwsh
npx http-server . -p 5173
```

- VS Code Live Server extension:

Press "Go Live" in VS Code (Live Server) to serve the folder.

The client is intentionally dumb — it forwards keystrokes as *intent* messages
and renders whatever ops the node sends back. All CRDT logic stays in Rust.

