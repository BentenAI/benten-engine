//! Phase-4-Foundation R6-FP-E `benten-admin-shell` integrator-binary
//! library surface.
//!
//! Closes r6-r1-browser-runtime finding `br-r6-r1-3` MAJOR: "Tauri 2.x
//! renderer crate has no production caller — `TauriRenderer` /
//! `InProcessSessionBridge` constructed only in tests; no integrator-
//! binary shell exists." After this commit lands the `TauriRenderer`
//! constructor + `InProcessSessionBridge::new` + `dispatch_ipc` are
//! wired into production paths under `tools/benten-admin-shell/` and
//! pinned by `tests/e2e_admin_shell_ipc.rs`.
//!
//! # Architecture (deployment shape (c) per CLAUDE.md baked-in #17)
//!
//! - The native shell is a **full peer** (shape a internally) holding
//!   the [`benten_engine::Engine`].
//! - The shell embeds a webview that loads the same
//!   `wasm32-unknown-unknown` admin UI v0 bundle a shape (b) browser tab
//!   loads.
//! - Webview <-> engine communication runs over **in-process IPC**
//!   instead of `fetch`; the cryptographic contract is the same
//!   [`benten_engine::thin_client::DidKeyedSession`] used by shape (b)
//!   per br-r1-14 closure at G24-E.
//!
//! # Trust model (CLAUDE.md baked-in #19)
//!
//! `benten-admin-shell` is an **engine-level extension** — compile-time
//! linked Rust crate; trust boundary is `cargo` + code review; out of
//! scope for Class B β `read_node_as` gating. The integrator binary IS
//! the engine. The webview running inside the integrator binary is a
//! thin-client surface (shape b internally) and IS gated through the
//! T2 + T3 defenses + the three rungs in [`benten_renderer_tauri`].
//!
//! # Two-mode compilation
//!
//! 1. **Default mode** (no `tauri` feature). Production build path
//!    pre-Tauri-runtime-vendor. Builds the IPC dispatch wiring without
//!    pulling the Tauri 2.x dep tree. Exercised end-to-end via
//!    `tests/e2e_admin_shell_ipc.rs`.
//! 2. **`tauri` feature mode** (opt-in). Links the real Tauri 2.x
//!    runtime, loads the webview, hooks Tauri command handlers into
//!    [`AdminShellState::dispatch`]. See `src/main.rs` for the boot
//!    sequence.
//!
//! See `Cargo.toml` header for the v1-assessment-window deferral
//! rationale on the webview-driven tauri-driver smoke test (half (ii)
//! of br-r6-r1-3).

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(clippy::module_name_repetitions)]

use std::sync::Arc;

use benten_engine::thin_client::{DidKeyedSession, SessionConfig, SessionToken};
use benten_renderer_tauri::{
    AdminUiManifest, InProcessSessionBridge, IpcError, IpcRequest, IpcResponse, TauriRenderer,
    WEBVIEW_CSP_HEADER,
};

/// Canonical cap-grant set the admin UI v0 plugin's `requires` envelope
/// MUST publish for the IPC method-cap-binding map (per
/// [`benten_renderer_tauri::ipc_method_cap_bindings`]) to admit every
/// allowlisted method.
///
/// Closes the secondary half of `br-r6-r1-8` MINOR: "No production
/// `admin_ui_v0_manifest()` constructor in benten-platform-foundation"
/// — the integrator binary is the named NOW destination + per-test
/// drift is asserted by `tests/canonical_manifest_matches_ipc_binding`.
///
/// The empty-cap sentinel `""` (used by `ui.notify`) is intentionally
/// NOT in this list — manifests don't need to publish the no-op cap.
///
/// The 6 cap scopes are the union of distinct non-empty values in
/// [`benten_renderer_tauri::IPC_METHOD_CAP_BINDING`].
pub const ADMIN_UI_V0_CANONICAL_CAPS: &[&str] = &[
    "graph:read",
    "graph:write",
    "caps:read",
    "identity:read",
    "plugin:read",
    "plugin:install",
];

/// Build the canonical admin-UI-v0 manifest envelope — granting exactly
/// the six cap scopes the IPC method-cap-binding map references.
///
/// Use this constructor at every production caller (integrator binary
/// boot + integration tests). Hand-rolled per-test manifests drift; the
/// drift is asserted-against in `canonical_manifest_matches_ipc_binding`.
#[must_use]
pub fn admin_ui_v0_canonical_manifest() -> AdminUiManifest {
    AdminUiManifest::with_caps(ADMIN_UI_V0_CANONICAL_CAPS.iter().copied())
}

/// State the Tauri 2.x command-invoke pipeline (or the equivalent
/// default-mode E2E test driver) holds across the process lifetime.
///
/// Owns:
///
/// - The [`TauriRenderer`] (allowlist + manifest + bridge composed).
/// - The [`DidKeyedSession`] state machine (challenge mint + handshake
///   completion + per-request resolve).
/// - The synthetic origin pinned at handshake (`"tauri://localhost"`
///   per [`InProcessSessionBridge`] doc).
///
/// The integrator-binary command handlers borrow `&AdminShellState`;
/// the dispatch flow is purely synchronous + thread-safe (the
/// underlying `DidKeyedSession` is internally `Mutex`-protected).
pub struct AdminShellState {
    renderer: TauriRenderer,
    session: Arc<DidKeyedSession>,
}

impl std::fmt::Debug for AdminShellState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdminShellState")
            .field("renderer", &self.renderer)
            .field("session", &self.session)
            .finish_non_exhaustive()
    }
}

impl AdminShellState {
    /// Construct the shell state with a production-shaped
    /// [`DidKeyedSession`] (real verifier + OS CSPRNG + wallclock).
    ///
    /// Build steps:
    ///
    /// 1. Mint a [`DidKeyedSession`] via [`DidKeyedSession::new`].
    /// 2. Wrap it in an [`InProcessSessionBridge`] (transport =
    ///    [`benten_engine::thin_client::Transport::Ipc`]).
    /// 3. Build a [`TauriRenderer`] with the canonical manifest +
    ///    bridge.
    ///
    /// The integrator binary's `main.rs` calls this once at boot.
    #[cfg(not(target_arch = "wasm32"))]
    #[must_use]
    pub fn new_production() -> Self {
        let session = Arc::new(DidKeyedSession::new(SessionConfig::default()));
        Self::from_session(session)
    }

    /// Construct with a caller-supplied [`DidKeyedSession`] — used by
    /// the integration-test E2E driver to inject deterministic hooks
    /// ([`DidKeyedSession::with_hooks`]) so the test can mint
    /// reproducible challenges + complete the handshake.
    #[must_use]
    pub fn from_session(session: Arc<DidKeyedSession>) -> Self {
        let bridge = InProcessSessionBridge::new(Arc::clone(&session));
        let renderer =
            TauriRenderer::new_with_manifest(admin_ui_v0_canonical_manifest()).with_bridge(bridge);
        Self { renderer, session }
    }

    /// Dispatch an IPC request through the renderer's three T3 defense
    /// rungs + the session-resolve step.
    ///
    /// The integrator binary's Tauri command handler calls this; the
    /// default-mode E2E test driver calls this directly.
    ///
    /// # Errors
    ///
    /// Returns the same [`IpcError`] envelope the renderer surfaces;
    /// callers map the error to a Tauri response shape (the wire
    /// framing is the integrator's responsibility — this crate stays
    /// transport-agnostic).
    pub fn dispatch(&self, request: IpcRequest) -> Result<IpcResponse, IpcError> {
        self.renderer.dispatch_ipc(request)
    }

    /// Borrow the underlying [`DidKeyedSession`]. The boot path uses
    /// this to mint challenges + complete handshakes; the integration
    /// test driver uses it to advance the deterministic clock.
    #[must_use]
    pub fn session(&self) -> &Arc<DidKeyedSession> {
        &self.session
    }

    /// Borrow the underlying [`TauriRenderer`]. Production callers use
    /// this for the [`TauriRenderer::webview_csp_header`] surface at
    /// webview-load time; tests use it for allowlist + manifest
    /// introspection.
    #[must_use]
    pub fn renderer(&self) -> &TauriRenderer {
        &self.renderer
    }

    /// Canonical locked CSP header to wire into Tauri's
    /// `WebviewWindowBuilder::with_csp()` (or equivalent) at webview
    /// boot. T3 defense rung 3.
    #[must_use]
    pub fn webview_csp_header(&self) -> &'static str {
        self.renderer.webview_csp_header()
    }
}

/// Synthetic origin the embedded webview presents to the
/// [`DidKeyedSession`] state machine. Per
/// [`InProcessSessionBridge::resolve`] doc: shape (c) presents
/// `"tauri://localhost"` for every request; the challenge MUST be
/// minted against the same value at handshake time.
pub const ADMIN_SHELL_BOUND_ORIGIN: &str = "tauri://localhost";

/// Convenience: build an [`IpcRequest`] envelope for a method + payload
/// + (optional) session token. Used by the integrator binary command
/// handlers + the integration tests.
#[must_use]
pub fn ipc_request(
    method: impl Into<String>,
    payload: serde_json::Value,
    session: Option<SessionToken>,
) -> IpcRequest {
    IpcRequest {
        method: method.into(),
        payload,
        session,
    }
}

/// Re-export the canonical webview CSP header at the crate top so the
/// integrator binary's `main.rs` does not also need to import the
/// `benten-renderer-tauri` crate name directly.
pub const ADMIN_SHELL_WEBVIEW_CSP_HEADER: &str = WEBVIEW_CSP_HEADER;

// ---------------------------------------------------------------------
// Sanity tests (compile-time + light unit pins)
// ---------------------------------------------------------------------

/// Compile-time pin: the integrator binary's canonical cap-grant set
/// equals the distinct non-empty cap-values in the IPC method-cap-
/// binding map (per `IPC_METHOD_CAP_BINDING` at
/// `crates/benten-renderer-tauri/src/lib.rs`). Drift on either side is
/// caught at unit-test time.
#[cfg(test)]
mod tests {
    use super::*;
    use benten_renderer_tauri::{IPC_METHOD_CAP_BINDING, IPC_METHOD_NAME_ALLOWLIST};
    use std::collections::BTreeSet;

    #[test]
    fn canonical_manifest_matches_ipc_binding() {
        let from_binding: BTreeSet<&str> = IPC_METHOD_CAP_BINDING
            .iter()
            .map(|(_, cap)| *cap)
            .filter(|c| !c.is_empty())
            .collect();
        let from_canonical: BTreeSet<&str> = ADMIN_UI_V0_CANONICAL_CAPS.iter().copied().collect();
        assert_eq!(
            from_binding, from_canonical,
            "ADMIN_UI_V0_CANONICAL_CAPS drift vs IPC_METHOD_CAP_BINDING distinct non-empty values"
        );
    }

    #[test]
    fn manifest_grants_every_allowlisted_methods_cap() {
        let manifest = admin_ui_v0_canonical_manifest();
        for method in IPC_METHOD_NAME_ALLOWLIST {
            let cap = IPC_METHOD_CAP_BINDING
                .iter()
                .find_map(|(m, c)| if m == method { Some(*c) } else { None })
                .unwrap_or("");
            assert!(
                manifest.grants_cap(cap),
                "canonical manifest must grant cap {cap:?} for method {method}"
            );
        }
    }

    #[test]
    fn bound_origin_is_tauri_localhost() {
        // The challenge MUST be minted against this exact string so
        // `InProcessSessionBridge::resolve`'s presented_origin matches.
        assert_eq!(ADMIN_SHELL_BOUND_ORIGIN, "tauri://localhost");
    }
}
