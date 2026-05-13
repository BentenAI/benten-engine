// benten-admin-shell webview bootstrap.
//
// Phase-4-Foundation R6-FP-E. This file lives in the webview-assets
// directory + is loaded by the embedded webview when the integrator
// binary boots with the `tauri` feature enabled. Under the default
// build mode (no `tauri` feature) the binary never loads this file —
// the integration tests at `tests/` exercise the same IPC dispatch
// pipeline directly through the Rust API.
//
// Wire-framing contract (the webview ↔ integrator boundary):
//
//   1. On boot, the webview presents `tauri://localhost` as its origin.
//   2. The webview obtains a `SessionToken` via the integrator's
//      `engine.session.handshake` command (not in the allowlist; this
//      command is the bootstrap path that does NOT pass through
//      `dispatch_ipc` — the renderer's three rungs admit requests
//      AFTER the session is established).
//   3. Every subsequent click invokes a Tauri command name from
//      `IPC_METHOD_NAME_ALLOWLIST` with the session token attached.
//      The Tauri command handler in `src/main.rs::tauri_boot` calls
//      `AdminShellState::dispatch` with the request envelope.
//
// With the `tauri` cargo feature enabled the boot path wires a real
// Tauri 2.x runtime via `tauri_boot::run` and this script invokes
// `window.__TAURI__.core.invoke` against the registered command
// handler. Without the feature, the default-mode binary prints a
// placeholder + the helpful "feature disabled" message below.

(function () {
  "use strict";

  var responseEl = document.getElementById("response");
  var button = document.getElementById("ipc-roundtrip");

  function setText(node, text) {
    while (node.firstChild) { node.removeChild(node.firstChild); }
    node.appendChild(document.createTextNode(text));
  }

  if (!button) { return; }

  button.addEventListener("click", function () {
    setText(responseEl, "(dispatching engine.plugin.manifest.review ...)");

    // With the `tauri` feature enabled the runtime is present and
    // `window.__TAURI__.core.invoke` is wired by `tauri_boot::run`.
    var tauri = window.__TAURI__;
    if (!tauri || !tauri.core || typeof tauri.core.invoke !== "function") {
      setText(responseEl, "tauri runtime not present in this build — `tauri` feature disabled (default-mode binary).");
      return;
    }

    tauri.core.invoke("dispatch_ipc", {
      method: "plugin.manifest.review",
      payload: {},
    }).then(function (resp) {
      setText(responseEl, JSON.stringify(resp));
    }).catch(function (err) {
      setText(responseEl, "ipc error: " + String(err));
    });
  });
})();
