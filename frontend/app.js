// RustCRDT — thin browser client.
//
// The browser never runs RGA logic. It only:
//   1. Sends keystroke intents to the node as { type, offset, ch }.
//   2. Receives { type: "state", text } frames and renders them.
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
  // Keep the log from growing unbounded in long sessions.
  while (logEl.children.length > 200) logEl.removeChild(logEl.lastChild);
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
  ws.onopen = () => {
    setState(true);
    log(`open ${url}`);
  };
  ws.onclose = () => {
    setState(false);
    log("close");
  };
  ws.onerror = (e) => log(`error ${e.message ?? ""}`);
  ws.onmessage = (e) => {
    let msg;
    try {
      msg = JSON.parse(e.data);
    } catch {
      log(`recv (unparseable) ${e.data}`);
      return;
    }
    if (msg.type === "state") {
      // Preserve cursor position across remote updates.
      const start = editor.selectionStart;
      const end = editor.selectionEnd;
      editor.value = msg.text;
      editor.selectionStart = Math.min(start, msg.text.length);
      editor.selectionEnd = Math.min(end, msg.text.length);
      log(`state len=${msg.text.length}`);
    } else {
      log(`recv ${msg.type}`);
    }
  };
});

// Local intent → ops.
// Naive single-char diff; the Rust node converts (offset, char) into a
// proper CRDT Op with Ids — the browser never generates Ids itself.
let prev = "";
editor.addEventListener("input", () => {
  const next = editor.value;
  if (next.length === prev.length + 1) {
    for (let i = 0; i < next.length; i++) {
      if (i >= prev.length || next[i] !== prev[i]) {
        send({ type: "local_insert", offset: i, ch: next[i] });
        break;
      }
    }
  } else if (next.length === prev.length - 1) {
    for (let i = 0; i < prev.length; i++) {
      if (i >= next.length || prev[i] !== next[i]) {
        send({ type: "local_delete", offset: i });
        break;
      }
    }
  }
  prev = next;
});
