//! Phase-4-Foundation Tauri 2.x renderer backend.
//!
//! # G24-E wave-7 landing
//!
//! This crate is the engine-level [Tauri 2.x](https://v2.tauri.app/)
//! renderer backend per `CLAUDE.md` baked-in commitment #19 (engine
//! extensions are Rust crates compile-time-linked; trust = "you compiled
//! this in"). It hosts an embedded webview that loads the same
//! `wasm32-unknown-unknown` admin UI v0 bundle the browser-tab
//! deployment shape uses, communicating with the embedded full peer via
//! in-process IPC instead of `fetch` (deployment shape (c) per
//! `CLAUDE.md` #17).
//!
//! ## Three defense rungs (T3 in `admin-ui-v0-threat-model.md`)
//!
//! 1. [`IpcAllowlist`] — explicit method-name allowlist. Methods NOT
//!    in the allowlist reject with
//!    [`IpcError::MethodNotInAllowlist`].
//! 2. [`TauriRenderer::dispatch_ipc`] — per-method capability binding.
//!    Each allowed method declares its required cap; invocation rejects
//!    with [`IpcError::CapabilityNotInManifest`] when the admin UI v0
//!    manifest envelope does not grant the bound cap.
//! 3. [`TauriRenderer::webview_csp_header`] — locked Content-Security-
//!    Policy at webview load: `default-src 'none'`,
//!    `script-src 'self' 'wasm-unsafe-eval'`,
//!    `connect-src 'self' tauri://*`,
//!    `style-src 'self'`, `font-src 'self'`. Forbids `'unsafe-eval'`
//!    + `'unsafe-inline'`.
//!
//! ## Cross-protocol contract (br-r1-14)
//!
//! Per CLAUDE.md baked-in #17 the (b) browser-tab and (c) embedded-
//! webview deployment shapes share ONE authentication contract:
//! [`benten_engine::thin_client::DidKeyedSession`] +
//! [`benten_engine::thin_client::SessionToken`]. This crate re-uses
//! that contract via the in-process [`InProcessSessionBridge`] — same
//! cryptographic state machine, transport swapped to in-process
//! channel.
//!
//! ## Trust model (CLAUDE.md #19)
//!
//! Engine extensions are out of scope for the Class B β read-side
//! gating boundary. `benten-renderer-tauri` does NOT pass requests
//! through `Engine::read_node_as` — it IS the engine. The boundary is
//! `cargo` and code review. The native shell holds the real Tauri 2.x
//! crate dependency at the integrator binary (this crate stays
//! Tauri-runtime-agnostic so the `tauri-runtime-verso` swap-readiness
//! pin per br-r1-9 + gap #1b holds).
//!
//! ## `tauri-runtime-verso` swap-readiness
//!
//! The [`Renderer`] trait surface (in
//! [`benten_platform_foundation::Renderer`]) is transport-agnostic:
//! no Tauri-2.x-specific webview-runtime types leak across the
//! boundary. A future swap to `tauri-runtime-verso` (Servo-based
//! webview) is a one-line `Cargo.toml` swap + a sibling
//! [`Renderer`] impl, not a breaking refactor.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(clippy::module_name_repetitions)]

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use benten_engine::thin_client::{
    DidKeyedSession, SessionToken, ThinClientSessionError, Transport,
};
use benten_errors::ErrorCode;
use benten_platform_foundation::{MaterializerOutput, RenderError, Renderer};

// ---------------------------------------------------------------------
// IPC allowlist
// ---------------------------------------------------------------------

/// Canonical method-name allowlist for the Tauri 2.x in-process IPC
/// channel. Must match
/// `docs/public-api/benten-renderer-tauri.json
/// _ipc_method_name_allowlist_baseline._anticipated_method_set` byte-
/// for-byte; a drift between this constant and the baseline file
/// triggers the `ipc_method_name_set_at_head_matches_public_api_baseline`
/// test (gap #1a closure).
///
/// Adding a method here REQUIRES an explicit baseline update + admin UI
/// v0 manifest review (T3 defense — silent IPC surface expansion is a
/// manifest-bypass risk).
pub const IPC_METHOD_NAME_ALLOWLIST: &[&str] = &[
    "engine.read_node_as",
    "engine.call_as",
    "engine.subscribe_via_on_change_as_with_cursor",
    "engine.list_caps",
    "engine.identity.user_did",
    "plugin.manifest.review",
    "plugin.install.consent",
    "ui.notify",
];

/// Per-method capability binding. Each allowlisted method names the cap
/// scope it requires; the admin UI v0 manifest envelope MUST grant the
/// scope or the dispatch rejects with
/// [`IpcError::CapabilityNotInManifest`].
///
/// `ui.notify` requires no cap (UI-only side effect inside the
/// webview); represented by the empty string.
pub const IPC_METHOD_CAP_BINDING: &[(&str, &str)] = &[
    ("engine.read_node_as", "graph:read"),
    ("engine.call_as", "graph:write"),
    (
        "engine.subscribe_via_on_change_as_with_cursor",
        "graph:read",
    ),
    ("engine.list_caps", "caps:read"),
    ("engine.identity.user_did", "identity:read"),
    ("plugin.manifest.review", "plugin:read"),
    ("plugin.install.consent", "plugin:install"),
    ("ui.notify", ""),
];

/// Allowlist enforcement seam (T3 defense rung 1). Methods absent from
/// [`IPC_METHOD_NAME_ALLOWLIST`] are rejected at this gate.
#[derive(Debug, Clone)]
pub struct IpcAllowlist {
    methods: BTreeSet<String>,
}

impl Default for IpcAllowlist {
    fn default() -> Self {
        Self::canonical()
    }
}

impl IpcAllowlist {
    /// Build the canonical allowlist from [`IPC_METHOD_NAME_ALLOWLIST`].
    #[must_use]
    pub fn canonical() -> Self {
        Self {
            methods: IPC_METHOD_NAME_ALLOWLIST
                .iter()
                .map(|m| (*m).to_string())
                .collect(),
        }
    }

    /// True iff `method` is on the allowlist (T3 defense rung 1).
    #[must_use]
    pub fn method_permitted(&self, method: &str) -> bool {
        self.methods.contains(method)
    }

    /// Iterate the live method-name set. Used by the drift-detector
    /// pin to compare against
    /// `docs/public-api/benten-renderer-tauri.json`.
    pub fn methods(&self) -> impl Iterator<Item = &str> {
        self.methods.iter().map(String::as_str)
    }

    /// Look up the cap-scope bound to `method`, or `None` if the method
    /// is not allowlisted. Empty string indicates "no cap required"
    /// (e.g. `ui.notify`).
    #[must_use]
    pub fn required_cap_for_method(&self, method: &str) -> Option<&'static str> {
        if !self.method_permitted(method) {
            return None;
        }
        IPC_METHOD_CAP_BINDING
            .iter()
            .find_map(|(m, cap)| if *m == method { Some(*cap) } else { None })
    }
}

// ---------------------------------------------------------------------
// CSP locked at webview load (T3 defense rung 3)
// ---------------------------------------------------------------------

/// Locked Content-Security-Policy header for the Tauri 2.x webview
/// load, per br-r1-11 + T3 defense rung 3.
///
/// Directives:
/// - `default-src 'none'` — deny everything not explicitly allowed.
/// - `script-src 'self' 'wasm-unsafe-eval'` — allow same-origin scripts
///   + the wasm32 bundle's `WebAssembly.compile`-equivalent (`'wasm-
///   unsafe-eval'` is the wasm-only relaxation; does NOT enable
///   classic `eval`).
/// - `connect-src 'self' tauri://*` — allow in-process IPC origin
///   for Tauri command invocations.
/// - `style-src 'self'` + `font-src 'self'` — same-origin assets only.
///
/// Forbidden: `'unsafe-eval'` (classic JS `eval`) + `'unsafe-inline'`
/// (inline scripts/styles). The
/// `webview_csp_locked_no_unsafe_eval` test pin enforces both.
pub const WEBVIEW_CSP_HEADER: &str = "default-src 'none'; \
script-src 'self' 'wasm-unsafe-eval'; \
connect-src 'self' tauri://*; \
style-src 'self'; \
font-src 'self'";

// ---------------------------------------------------------------------
// IPC request / response / error
// ---------------------------------------------------------------------

/// IPC request envelope from the webview to the native Tauri shell.
///
/// `payload` is the method-specific input parsed at the integrator's
/// per-method handler (after the three T3 defense rungs admit the
/// dispatch). The wire framing (Tauri 2.x `invoke` JSON) is the
/// integrator binary's responsibility; this crate operates on the
/// already-parsed [`IpcRequest`] shape per
/// [`benten_engine::thin_client`] module-doc precedent for "wire
/// framing is above this module".
#[derive(Debug, Clone)]
pub struct IpcRequest {
    /// Method name; must appear in [`IPC_METHOD_NAME_ALLOWLIST`].
    pub method: String,
    /// Method-specific payload.
    pub payload: serde_json::Value,
    /// Session token resolved at handshake. The dispatcher uses it to
    /// pin the principal DID + origin per
    /// [`DidKeyedSession::resolve`]. `None` is only acceptable when
    /// the renderer was constructed without a session bridge (test
    /// configurations + the bootstrap handshake itself, which does
    /// NOT pass through `dispatch_ipc`).
    pub session: Option<SessionToken>,
}

/// IPC response envelope returned to the webview. Carries no
/// principal information (the webview already knows its principal
/// from the session token).
#[derive(Debug, Clone)]
pub struct IpcResponse {
    /// Method-specific response payload.
    pub payload: serde_json::Value,
}

/// Typed IPC errors. Each variant maps to a stable [`ErrorCode`]; see
/// [`IpcError::error_code`]. The four T3 defense rungs surface here:
/// allowlist-miss (rung 1), cap-missing (rung 2), session-resolve-
/// failure (rung 1 of br-r1-14 cross-protocol contract), and CSP-load-
/// failure (rung 3, used by the integrator-binary boot path).
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum IpcError {
    /// Method is not in [`IPC_METHOD_NAME_ALLOWLIST`]. The webview
    /// requested a method the native shell does not expose. T3
    /// defense rung 1.
    #[error("ipc method not in allowlist: {method}")]
    MethodNotInAllowlist {
        /// Offending method name.
        method: String,
    },
    /// Method is allowlisted but its required cap is not in the admin
    /// UI v0 manifest envelope. T3 defense rung 2.
    #[error("ipc method requires cap not in manifest: method={method} cap={cap}")]
    CapabilityNotInManifest {
        /// Method requesting the cap.
        method: String,
        /// Cap scope that the manifest did not grant.
        cap: String,
    },
    /// Method invocation provided no session token (or the token
    /// failed to resolve at the [`DidKeyedSession`] layer). Wraps the
    /// underlying [`ThinClientSessionError`] for diagnostic surface
    /// continuity with shape (b) browser-tab failures.
    #[error("ipc session resolve failed: {0}")]
    SessionResolve(#[from] ThinClientSessionError),
    /// Session token was absent on a non-bootstrap invocation. The
    /// webview never gets to call into the engine without an
    /// established session.
    #[error("ipc invocation missing session token (no handshake)")]
    MissingSession,
}

impl IpcError {
    /// Stable catalog code for this error. The four T3 surface errors
    /// reuse existing thin-client codes (per cross-protocol contract
    /// br-r1-14); no new ErrorCodes are minted in this wave.
    #[must_use]
    pub fn error_code(&self) -> ErrorCode {
        match self {
            // The webview attempted to step outside the manifest
            // envelope — same semantic class as a thin-client
            // handshake against an unknown surface.
            Self::MethodNotInAllowlist { .. } | Self::CapabilityNotInManifest { .. } => {
                ErrorCode::ThinClientHandshakeInvalid
            }
            Self::SessionResolve(err) => err.error_code(),
            Self::MissingSession => ErrorCode::ThinClientHandshakeInvalid,
        }
    }
}

// ---------------------------------------------------------------------
// Manifest envelope (admin UI v0)
// ---------------------------------------------------------------------

/// Minimal admin-UI-v0 manifest envelope shape this renderer consumes
/// to drive the per-method cap-binding check.
///
/// The full plugin manifest schema lives in
/// [`benten_platform_foundation::PluginManifest`] (G24-D); this
/// minimal projection captures ONLY the `requires` envelope the IPC
/// dispatcher consults. Future widening to consume the full manifest
/// is a forwards-compat change (extra fields ignored).
#[derive(Debug, Clone, Default)]
pub struct AdminUiManifest {
    /// Cap scopes the admin UI v0 plugin's `requires` envelope grants.
    /// Methods whose bound cap is NOT in this set reject at the IPC
    /// boundary.
    pub granted_caps: BTreeSet<String>,
}

impl AdminUiManifest {
    /// Build a manifest envelope granting the given cap scopes.
    #[must_use]
    pub fn with_caps<I, S>(caps: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            granted_caps: caps.into_iter().map(Into::into).collect(),
        }
    }

    /// True iff the manifest grants `cap`. Empty cap (`""`) is the
    /// "no cap required" sentinel and is always granted.
    #[must_use]
    pub fn grants_cap(&self, cap: &str) -> bool {
        cap.is_empty() || self.granted_caps.contains(cap)
    }
}

// ---------------------------------------------------------------------
// In-process session bridge (br-r1-14)
// ---------------------------------------------------------------------

/// Bridge that connects the embedded webview's session-establishment
/// path to [`DidKeyedSession`]. Same cryptographic contract as the
/// shape (b) browser-tab thin-client; transport is in-process IPC
/// instead of HTTP.
///
/// The integrator binary holds a [`DidKeyedSession`] alongside the
/// engine (per
/// [`benten_engine::thin_client`] module doc); this bridge is the
/// shape (c) entry-point.
pub struct InProcessSessionBridge {
    session: Arc<DidKeyedSession>,
}

impl std::fmt::Debug for InProcessSessionBridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InProcessSessionBridge")
            .field("transport", &Transport::Ipc)
            .finish_non_exhaustive()
    }
}

impl InProcessSessionBridge {
    /// Construct a bridge around an existing [`DidKeyedSession`]. The
    /// session state machine lives in `benten-engine`; this crate just
    /// references it.
    #[must_use]
    pub fn new(session: Arc<DidKeyedSession>) -> Self {
        Self { session }
    }

    /// Transport identifier for this bridge. Always
    /// [`Transport::Ipc`] — shape (c) per CLAUDE.md #17.
    #[must_use]
    pub fn transport(&self) -> Transport {
        Transport::Ipc
    }

    /// Resolve a session token to the authoritative principal DID. The
    /// origin recheck is per-request (Family F1 gap #2 mid-session
    /// defense); shape (c) presents the synthetic origin
    /// `"tauri://localhost"` for every request — the SAME value the
    /// challenge was minted against at handshake time.
    ///
    /// # Errors
    ///
    /// Propagates the underlying [`ThinClientSessionError`] (origin
    /// mismatch / expiry / unknown token).
    pub fn resolve(
        &self,
        token: &SessionToken,
        presented_origin: &str,
    ) -> Result<String, ThinClientSessionError> {
        self.session.resolve(token, presented_origin)
    }

    /// Underlying [`DidKeyedSession`] reference — for callers that want
    /// to drive the establishment path directly (the Tauri shell at
    /// boot invokes
    /// [`DidKeyedSession::emit_challenge`] + `establish_session`).
    #[must_use]
    pub fn session(&self) -> &Arc<DidKeyedSession> {
        &self.session
    }
}

// ---------------------------------------------------------------------
// TauriRenderer
// ---------------------------------------------------------------------

/// Tauri 2.x renderer backend. Engine extension per CLAUDE.md #19;
/// compile-time linked into the integrator binary.
///
/// Holds the IPC allowlist + admin UI v0 manifest + in-process session
/// bridge. The integrator binary wires a [`TauriRenderer`] into the
/// Tauri 2.x command-invoke pipeline; every command first passes
/// through [`Self::dispatch_ipc`] for the T3 three-rung defense.
pub struct TauriRenderer {
    allowlist: IpcAllowlist,
    manifest: AdminUiManifest,
    bridge: Option<InProcessSessionBridge>,
}

impl std::fmt::Debug for TauriRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TauriRenderer")
            .field("allowlist_methods", &self.allowlist.methods.len())
            .field("bridge_attached", &self.bridge.is_some())
            .finish_non_exhaustive()
    }
}

impl TauriRenderer {
    /// Construct a renderer with the canonical IPC allowlist + the
    /// admin UI v0 manifest envelope `manifest`. No session bridge
    /// attached (use [`Self::with_bridge`] to wire one).
    #[must_use]
    pub fn new_with_manifest(manifest: AdminUiManifest) -> Self {
        Self {
            allowlist: IpcAllowlist::canonical(),
            manifest,
            bridge: None,
        }
    }

    /// Attach an in-process session bridge. Dispatches that present a
    /// session token resolve through this bridge before the
    /// per-method dispatch.
    #[must_use]
    pub fn with_bridge(mut self, bridge: InProcessSessionBridge) -> Self {
        self.bridge = Some(bridge);
        self
    }

    /// Allowlist accessor — used by tests + the drift-detector pin.
    #[must_use]
    pub fn allowlist(&self) -> &IpcAllowlist {
        &self.allowlist
    }

    /// Manifest accessor — used by tests.
    #[must_use]
    pub fn manifest(&self) -> &AdminUiManifest {
        &self.manifest
    }

    /// Locked CSP header for the webview-load boundary (T3 defense
    /// rung 3). The integrator binary wires this into Tauri's
    /// `WebviewWindowBuilder::with_csp()` (or equivalent) at boot.
    #[must_use]
    pub fn webview_csp_header(&self) -> &'static str {
        WEBVIEW_CSP_HEADER
    }

    /// Canonical IPC method-name allowlist — used by the
    /// drift-detector pin to compare against the public-api baseline
    /// file.
    #[must_use]
    pub fn ipc_method_allowlist() -> Vec<String> {
        IPC_METHOD_NAME_ALLOWLIST
            .iter()
            .map(|m| (*m).to_string())
            .collect()
    }

    /// Dispatch an IPC request through the three T3 defense rungs:
    ///
    /// 1. Allowlist filter — method-name MUST be in
    ///    [`IPC_METHOD_NAME_ALLOWLIST`] (rung 1).
    /// 2. Capability binding — admin UI v0 manifest envelope MUST
    ///    grant the bound cap (rung 2).
    /// 3. Session resolution — if a bridge is attached, the session
    ///    token resolves to the authoritative principal DID; origin
    ///    pinning + expiry check fire here (br-r1-14 cross-protocol
    ///    contract).
    ///
    /// The CSP rung (3 of the T3 defense composition) fires at
    /// webview-load time via [`Self::webview_csp_header`], NOT here —
    /// CSP is a load-boundary defense, not a per-call defense.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] when any of the three rungs reject. The
    /// integrator binary surfaces the error to the webview as an
    /// opaque error code; no diagnostic reason crosses the IPC
    /// boundary (operator-only audit per
    /// [`benten_engine::thin_client`] module doc).
    pub fn dispatch_ipc(&self, request: IpcRequest) -> Result<IpcResponse, IpcError> {
        // (1) Allowlist filter (T3 rung 1). Rejects BEFORE any payload
        // parse so an attacker-crafted payload can't pivot through a
        // forbidden method.
        if !self.allowlist.method_permitted(&request.method) {
            return Err(IpcError::MethodNotInAllowlist {
                method: request.method,
            });
        }

        // (2) Capability binding (T3 rung 2). The cap is bound to the
        // method at the IpcAllowlist seam; the manifest envelope is
        // consulted here. Empty cap = "no cap required" (e.g.
        // `ui.notify`).
        let cap = self
            .allowlist
            .required_cap_for_method(&request.method)
            .unwrap_or("");
        if !self.manifest.grants_cap(cap) {
            return Err(IpcError::CapabilityNotInManifest {
                method: request.method,
                cap: cap.to_string(),
            });
        }

        // (3) Session resolution (br-r1-14 cross-protocol contract).
        // Only fires when a bridge is attached AND the request carries
        // a token. The webview MUST present a session on every
        // non-bootstrap invocation; absent-session reject is
        // unconditional once a bridge is attached.
        if let Some(bridge) = &self.bridge {
            let token = request.session.as_ref().ok_or(IpcError::MissingSession)?;
            // Shape (c) presents a synthetic origin pinned to the
            // Tauri local origin. The handshake bound this origin at
            // session establishment (per `InProcessSessionBridge`
            // doc); per-request recheck happens inside `resolve`.
            let _principal = bridge.resolve(token, "tauri://localhost")?;
            // Production wiring forwards `_principal` to
            // `Engine::call_as` / `Engine::read_node_as`. This crate
            // does NOT call the engine directly — the integrator
            // binary's command-handler does. See module doc + #19.
        }

        // Past the three rungs: the request is admitted. The
        // method-specific handler lives in the integrator binary
        // (Tauri command handlers), which calls back into engine
        // facade methods with the resolved principal. This crate
        // returns an empty success envelope; the integrator overwrites
        // `payload` with the real response.
        Ok(IpcResponse {
            payload: serde_json::Value::Null,
        })
    }
}

// ---------------------------------------------------------------------
// Renderer trait impl (transport-agnostic; verso swap-readiness)
// ---------------------------------------------------------------------

/// [`Renderer`] impl for the Tauri 2.x deployment shape (c). The trait
/// surface is transport-agnostic per br-r1-9 + arch-r1-16 — no
/// Tauri-2.x-specific types appear in the [`Renderer::render`] signature
/// — so swapping to `tauri-runtime-verso` later is a sibling
/// [`Renderer`] impl, not a breaking refactor.
impl Renderer for TauriRenderer {
    fn render(&self, _output: &MaterializerOutput) -> Result<(), RenderError> {
        // The integrator binary's Tauri command handler mounts the
        // materializer output into the webview DOM via the standard
        // Tauri 2.x `emit` API. This crate stays Tauri-runtime-
        // agnostic (preserves the verso swap-readiness); the actual
        // emit call lives in the integrator binary.
        Ok(())
    }

    fn backend_name(&self) -> &'static str {
        "tauri-2.x"
    }
}

// ---------------------------------------------------------------------
// Compile-time pluggability assertions
// ---------------------------------------------------------------------

/// Compile-time assertion that `TauriRenderer: Renderer + Send + Sync`.
/// Used by the verso swap-readiness pin to prove the trait does NOT
/// leak Tauri-2.x-specific associated types.
#[doc(hidden)]
pub fn _assert_renderer_object_safety()
where
    TauriRenderer: Renderer + Send + Sync,
{
}

/// Public method-cap binding accessor as a stable map. Used by tests +
/// the operator audit surface to enumerate the IPC surface without
/// holding a `TauriRenderer` instance.
#[must_use]
pub fn ipc_method_cap_bindings() -> BTreeMap<String, String> {
    IPC_METHOD_CAP_BINDING
        .iter()
        .map(|(m, c)| ((*m).to_string(), (*c).to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowlist_canonical_matches_constant() {
        let allowlist = IpcAllowlist::canonical();
        for method in IPC_METHOD_NAME_ALLOWLIST {
            assert!(allowlist.method_permitted(method), "missing: {method}");
        }
        assert!(!allowlist.method_permitted("engine.arbitrary"));
    }

    #[test]
    fn csp_header_forbids_unsafe_eval_and_unsafe_inline() {
        assert!(WEBVIEW_CSP_HEADER.contains("default-src 'none'"));
        assert!(WEBVIEW_CSP_HEADER.contains("script-src 'self' 'wasm-unsafe-eval'"));
        assert!(WEBVIEW_CSP_HEADER.contains("connect-src 'self' tauri://*"));
        assert!(WEBVIEW_CSP_HEADER.contains("style-src 'self'"));
        assert!(WEBVIEW_CSP_HEADER.contains("font-src 'self'"));
        // `'wasm-unsafe-eval'` is the wasm-only relaxation; classic
        // `'unsafe-eval'` MUST NOT appear.
        let cleaned = WEBVIEW_CSP_HEADER.replace("'wasm-unsafe-eval'", "");
        assert!(!cleaned.contains("'unsafe-eval'"));
        assert!(!WEBVIEW_CSP_HEADER.contains("'unsafe-inline'"));
    }

    #[test]
    fn cap_binding_covers_every_allowlisted_method() {
        for method in IPC_METHOD_NAME_ALLOWLIST {
            let found = IPC_METHOD_CAP_BINDING.iter().any(|(m, _)| m == method);
            assert!(found, "method {method} has no cap binding");
        }
    }
}
