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

// Auto-fill WS URL from the hostname the page was served from so a phone on
// the same LAN connects to the right node without manual editing.
{
  const host = location.hostname === "127.0.0.1" || location.hostname === "localhost"
    ? "127.0.0.1"
    : location.hostname;
  $("ws-url").value = `ws://${host}:8001`;
}

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

// After inserting N chars we expect N state updates from the server before the
// cursor is final. Store the intended cursor position and the minimum text
// length at which it should be applied so that intermediate state updates
// (carrying partial results) don't clobber the cursor.
let intendedCursor = null;   // { pos, minLen }

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
      editor.value = msg.text;
      prev = msg.text; // sync diff baseline — prevents stale-offset deletes

      if (intendedCursor !== null && msg.text.length >= intendedCursor.minLen) {
        // All our inserts have been reflected — place cursor at the intended spot.
        const pos = Math.min(intendedCursor.pos, msg.text.length);
        editor.selectionStart = pos;
        editor.selectionEnd = pos;
        intendedCursor = null;
      } else if (intendedCursor !== null) {
        // Intermediate update — leave cursor wherever the browser put it.
      } else {
        // Remote update — preserve the user's current cursor position.
        const saved = Math.min(editor.selectionStart, msg.text.length);
        editor.selectionStart = saved;
        editor.selectionEnd = saved;
      }
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

// Capture the exact cursor position on keydown so that deletes of
// duplicate characters (e.g. the first 'l' in "hello") target the right
// CRDT entry instead of the first mismatch found by the string diff.
let pendingDeleteOffset = null;
editor.addEventListener("keydown", (e) => {
  if (e.key === "Backspace") pendingDeleteOffset = editor.selectionStart - 1;
  else if (e.key === "Delete") pendingDeleteOffset = editor.selectionStart;
  else pendingDeleteOffset = null;
});

editor.addEventListener("input", () => {
  const next = editor.value;
  const delta = next.length - prev.length;

  if (delta > 0) {
    // One or more characters inserted (single keystroke, paste, autocomplete).
    // Find where the new chars begin (first mismatch from the left).
    let start = 0;
    while (start < prev.length && next[start] === prev[start]) start++;
    for (let i = 0; i < delta; i++) {
      // Each previously sent insert shifts all subsequent visible offsets by +1,
      // so the next char must go at start + i (not a fixed offset).
      send({ type: "local_insert", offset: start + i, ch: next[start + i] });
    }
    // Record where the cursor should land after all state updates arrive.
    intendedCursor = { pos: start + delta, minLen: next.length };
  } else if (delta < 0) {
    // One or more characters deleted (backspace, select-all+delete, etc.).
    const deleteCount = -delta;
    let offset = (deleteCount === 1 && pendingDeleteOffset !== null && pendingDeleteOffset >= 0)
      ? pendingDeleteOffset
      : (() => { for (let i = 0; i < prev.length; i++) { if (i >= next.length || prev[i] !== next[i]) return i; } return 0; })();
    for (let i = 0; i < deleteCount; i++) {
      // Each delete removes the char at `offset`; remaining chars shift left.
      send({ type: "local_delete", offset });
    }
    intendedCursor = null;
    pendingDeleteOffset = null;
  }
  // delta === 0: autocorrect replaced same-length text — too ambiguous to handle safely.

  prev = next;
});
