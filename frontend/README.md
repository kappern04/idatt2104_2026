# Frontend

Static web client for a `rustcrdt-node`. Open `index.html` directly in a
browser, or serve the folder with any static file server.

## Localhost-only (laptop browser)

```pwsh
# From repo root. --bind 127.0.0.1 prevents Windows from binding to [::] (IPv6)
# which causes ERR_ADDRESS_INVALID in the browser.
py      -m http.server 5173 --bind 127.0.0.1 --directory frontend
python3 -m http.server 5173 --bind 127.0.0.1 --directory frontend
```

Open `http://localhost:5173/index.html` and connect to `ws://127.0.0.1:8001`.

---

## Serve to mobile / LAN (laptop + phone on the same Wi-Fi)

### Step 1 — find your laptop's LAN IP

```pwsh
# Windows
ipconfig | findstr "IPv4"

# macOS / Linux
ip route get 1 | awk '{print $7}'   # or: ifconfig | grep "inet "
```

Note the address that starts with `192.168.x.x` or `10.x.x.x` — call it
`<LAPTOP_IP>`.

### Step 2 — start the node

```pwsh
cargo run -p rustcrdt-node -- --port 9001 --ui-port 8001 --peer-id 1
```

The UI bridge already binds `0.0.0.0:8001`, so no backend change is needed.

### Step 3 — serve the frontend on the LAN

```pwsh
# Windows (Python) — from repo root
py -m http.server 5173 --bind 0.0.0.0 --directory frontend

# macOS / Linux
python3 -m http.server 5173 --bind 0.0.0.0 --directory frontend

# Node.js alternative
npx http-server frontend -a 0.0.0.0 -p 5173
```

### Step 4 — open on your phone

Make sure the phone is on the **same Wi-Fi** as the laptop, then open:

```
http://<LAPTOP_IP>:5173/index.html
```

The WS URL field auto-fills to `ws://<LAPTOP_IP>:8001` — just click
**Connect**.

> **Security note:** Binding to `0.0.0.0` exposes both the HTTP server and
> the node's WebSocket port to everyone on the LAN. This is fine for a local
> demo. For a public network (coffee shop, university) block the ports in your
> firewall first, or use a tunnelling tool like
> [ngrok](https://ngrok.com): `ngrok tcp 8001`.

### Windows Firewall — allow inbound on ports 5173 and 8001

If the phone cannot reach the laptop, Windows Firewall is the most likely
cause. Run once in an elevated PowerShell:

```pwsh
New-NetFirewallRule -DisplayName "RustCRDT HTTP"  -Direction Inbound -Protocol TCP -LocalPort 5173 -Action Allow
New-NetFirewallRule -DisplayName "RustCRDT WS"    -Direction Inbound -Protocol TCP -LocalPort 8001 -Action Allow
```

To remove the rules afterwards:

```pwsh
Remove-NetFirewallRule -DisplayName "RustCRDT HTTP"
Remove-NetFirewallRule -DisplayName "RustCRDT WS"
```

---

## Troubleshooting checklist

- **Phone shows "connection refused" or times out**
  - Confirm both devices are on the same Wi-Fi SSID.
  - Double-check `<LAPTOP_IP>` — run `ipconfig` again; the address can change
    if the laptop reconnects.
  - Add Windows Firewall rules (see above) and retry.
  - Ping from phone: open a browser and navigate to
    `http://<LAPTOP_IP>:5173` — if it times out, the HTTP port is blocked.

- **Frontend loads but WS says "disconnected" immediately**
  - Confirm the node is running (`cargo run …` output should show
    `WebSocket UI bridge on 0.0.0.0:8001`).
  - Firewall may allow HTTP (5173) but block WS (8001) — add the WS rule.
  - In the browser address bar check the auto-filled URL; edit it manually
    if needed.

- **Ghost characters / edits don't appear on the other device**
  - Reload the page on both devices and reconnect — the server pushes the
    full document state on every fresh connection.
  - Check the browser console (DevTools → Console) for `send_delete` /
    `send_insert` log lines; `prev` and `next` should each differ by one
    character per keystroke.
  - Open DevTools → Network → WS → Messages and verify `state` frames arrive
    after every edit.

- **`ERR_ADDRESS_INVALID` on laptop**
  - Use `--bind 127.0.0.1` (localhost only) or `--bind 0.0.0.0` (LAN).
  - Never omit `--bind`; without it Python on Windows binds `[::]` and the
    browser URL becomes `http://[::]:5173/`.

---

## Acceptance test — laptop ↔ phone sync

Follow these steps in order to confirm convergence works:

| # | Device | Action | Expected on **both** devices |
|---|--------|--------|-------------------------------|
| 1 | Laptop | Connect laptop browser to `ws://127.0.0.1:8001` | "connected" shown |
| 2 | Phone  | Open `http://<LAPTOP_IP>:5173/index.html`, connect to `ws://<LAPTOP_IP>:8001` | "connected" shown |
| 3 | Laptop | Type `hello` | Both show `hello` |
| 4 | Phone  | Type ` world` (space then world) | Both show `hello world` |
| 5 | Laptop | Position cursor after `o` in `hello`, press Backspace | Both show `hell world` |
| 6 | Phone  | Select all, Delete | Both show empty editor |
| 7 | Both   | Type one character each simultaneously | Both show the same 2-char string |

If step 7 shows different text on the two devices, paste the Node 1 backend
log (filtered for `rga_apply`) and the browser WS frames from both tabs.

---

## Capture commands (if something goes wrong)

```pwsh
# Backend — structured debug log to file
$env:RUST_LOG = "rustcrdt_node=debug"
cargo run -p rustcrdt-node -- --port 9001 --ui-port 8001 --peer-id 1 2>&1 | Tee-Object node1.log

# Filter log for apply events only
Select-String -Path node1.log -Pattern "rga_apply"

# Browser console — paste the output of:
#   DevTools → Console → filter "send_delete OR send_insert OR state_recv"
#
# Browser WS frames — DevTools → Network → WS →
#   click the ws:// connection → Messages tab → right-click → Copy all messages
```

---

## VS Code Live Server (localhost only)

Press "Go Live" in VS Code — this serves on `127.0.0.1` only and is not
reachable from other devices.

---

The client is intentionally dumb — it forwards keystrokes as *intent* messages
and renders whatever ops the node sends back. All CRDT logic stays in Rust.
