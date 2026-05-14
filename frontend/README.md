# Frontend

Static web client for a `rustcrdt-node`. Open `index.html` directly in a
browser, or serve the folder with any static file server, e.g.:

```pwsh
# Run from the repo root. --bind 127.0.0.1 prevents Windows from binding
# to [::] (IPv6 any-address) which causes ERR_ADDRESS_INVALID in the browser.
py      -m http.server 5173 --bind 127.0.0.1 --directory frontend
python3 -m http.server 5173 --bind 127.0.0.1 --directory frontend

# Or cd into the frontend folder first, then omit --directory:
cd frontend
py -m http.server 5173 --bind 127.0.0.1
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

