// RustCRDT — thin browser client.
//
// This file is intentionally tiny: it opens a WebSocket to a local node and
// forwards keystrokes as CRDT ops, then renders ops that arrive back. The real
// logic — Ids, RGA, convergence — lives in the Rust node. The browser never
// makes up an Id of its own; it only describes intent (e.g. "insert 'a' at
// visible offset 3") and lets the node turn that into a CRDT op.
//
// Wire format mirrors `backend/src/network/protocol.rs::Message`.

const $ = (id) => document.getElementById(id);
const editor = $("editor");
const logEl = $("log");
const stateEl = $("state");

let ws = null;

function setState(connected) {
  stateEl.textContent = connected ? "connected" : "disconnected";
  stateEl.className = "state " + (connected ? "connected" : "disconnected");
}

function log(line) {
  const div = document.createElement("div");
  div.className = "entry";
  div.textContent = line;
  logEl.prepend(div);
}

function send(msg) {
  if (ws && ws.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify(msg));
  }
}

$("connect").addEventListener("click", () => {
  const url = $("ws-url").value.trim();
  if (ws) ws.close();
  ws = new WebSocket(url);
  ws.onopen    = () => { setState(true);  log(`open ${url}`); };
  ws.onclose   = () => { setState(false); log("close"); };
  ws.onerror   = (e) => log(`error ${e.message ?? ""}`);
  ws.onmessage = (e) => {
    log(`recv ${e.data}`);
    // TODO: parse Message::Op and apply to editor view.
  };
});

// Local intent → ops. For now we just diff naively against the previous value;
// the Rust node converts (offset, char) into a proper CRDT Op with Ids.
let prev = "";
editor.addEventListener("input", () => {
  const next = editor.value;
  // Naive single-char diff suitable for demo; replace with a proper diff later.
  if (next.length === prev.length + 1) {
    for (let i = 0; i < next.length; i++) {
      if (next[i] !== prev[i]) {
        send({ type: "local_insert", offset: i, ch: next[i] });
        break;
      }
    }
  } else if (next.length === prev.length - 1) {
    for (let i = 0; i < prev.length; i++) {
      if (prev[i] !== next[i]) {
        send({ type: "local_delete", offset: i });
        break;
      }
    }
  }
  prev = next;
});

