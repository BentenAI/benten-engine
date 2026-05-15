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
//! 1. [`IPC_METHODS`] — explicit method-name allowlist. Methods NOT
//!    in the slice reject with
//!    [`IpcError::MethodNotInAllowlist`].
//! 2. [`TauriRenderer::dispatch_ipc`] — per-method capability binding.
//!    Each allowed method declares its [`CapRequirement`]; invocation
//!    rejects with [`IpcError::CapabilityNotInManifest`] when the admin
//!    UI v0 manifest envelope does not grant the required cap.
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

use std::collections::BTreeSet;
use std::sync::Arc;

use benten_engine::thin_client::{
    DidKeyedSession, SessionToken, ThinClientSessionError, Transport,
};
use benten_errors::ErrorCode;
use benten_platform_foundation::{MaterializerOutput, RenderError, Renderer};

// ---------------------------------------------------------------------
// IPC allowlist
// ---------------------------------------------------------------------

/// Capability requirement for an IPC method. A typed sum replaces the
/// former empty-string-as-sentinel idiom: `None` is structurally
/// distinct from `Required(scope)`, so the "binding missing" case is
/// unrepresentable at the call site and the fail-OPEN drift hazard
/// (formerly `unwrap_or("")` in [`TauriRenderer::dispatch_ipc`]) is
/// eliminated by construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapRequirement {
    /// Method is invokable without any cap (e.g. `ui.notify`, a
    /// UI-only side effect inside the webview).
    None,
    /// Method requires the named cap scope; the admin UI v0 manifest
    /// envelope MUST grant it or the dispatch rejects with
    /// [`IpcError::CapabilityNotInManifest`].
    Required(&'static str),
}

/// One IPC method's complete binding: its name + the cap it requires.
///
/// This typed slice element replaces the former parallel-array shape
/// (`IPC_METHOD_NAME_ALLOWLIST: &[&str]` +
/// `IPC_METHOD_CAP_BINDING: &[(&str, &str)]`). Co-locating the name
/// and cap in one struct makes the cross-array synchronization
/// invariant — every allowlisted method has exactly one cap binding,
/// and every binding entry is on the allowlist — a structural
/// guarantee rather than a runtime test pin. There is no longer a
/// "reverse-completeness" direction to enforce: the bijection is the
/// data shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IpcMethod {
    /// Method name; the webview MUST request a name in [`IPC_METHODS`].
    pub name: &'static str,
    /// Cap scope this method requires (or [`CapRequirement::None`]).
    pub cap: CapRequirement,
}

/// Canonical IPC method binding set for the Tauri 2.x in-process IPC
/// channel — the single source of truth for the T3 defense rung 1
/// (allowlist) AND rung 2 (per-method cap binding).
///
/// The method-name set (the `name` field of each entry) must match
/// `docs/public-api/benten-renderer-tauri.json
/// ._ipc_method_name_allowlist_baseline._anticipated_method_set`
/// byte-for-byte; a drift triggers the
/// `ipc_method_name_set_at_head_matches_public_api_baseline` test
/// (gap #1a closure).
///
/// Adding a method here REQUIRES an explicit baseline update + admin UI
/// v0 manifest review (T3 defense — silent IPC surface expansion is a
/// manifest-bypass risk). `ui.notify` carries
/// [`CapRequirement::None`] (UI-only side effect; no cap gate).
pub const IPC_METHODS: &[IpcMethod] = &[
    IpcMethod {
        name: "engine.read_node_as",
        cap: CapRequirement::Required("graph:read"),
    },
    IpcMethod {
        name: "engine.call_as",
        cap: CapRequirement::Required("graph:write"),
    },
    IpcMethod {
        name: "engine.subscribe_via_on_change_as_with_cursor",
        cap: CapRequirement::Required("graph:read"),
    },
    IpcMethod {
        name: "engine.list_caps",
        cap: CapRequirement::Required("caps:read"),
    },
    IpcMethod {
        name: "engine.identity.user_did",
        cap: CapRequirement::Required("identity:read"),
    },
    IpcMethod {
        name: "plugin.manifest.review",
        cap: CapRequirement::Required("plugin:read"),
    },
    IpcMethod {
        name: "plugin.install.consent",
        cap: CapRequirement::Required("plugin:install"),
    },
    IpcMethod {
        name: "ui.notify",
        cap: CapRequirement::None,
    },
];

/// Look up the [`IpcMethod`] binding for `name`, or `None` if the
/// method is not on the allowlist.
///
/// This is the single lookup that serves BOTH T3 defense rungs: the
/// presence of a result is rung 1 (allowlist membership); the result's
/// [`CapRequirement`] is rung 2 (cap binding). The former shape forced
/// three traversals per dispatch (a `BTreeSet::contains`, a redundant
/// repeat inside `required_cap_for_method`, and a parallel-array linear
/// scan); the typed slice collapses that to one linear scan over a
/// cache-friendly `&'static` slice — allocation-free.
#[must_use]
pub fn ipc_method(name: &str) -> Option<&'static IpcMethod> {
    IPC_METHODS.iter().find(|m| m.name == name)
}

/// Iterate the canonical method-name set. Used by the drift-detector
/// pin to compare against `docs/public-api/benten-renderer-tauri.json`.
pub fn ipc_method_names() -> impl Iterator<Item = &'static str> {
    IPC_METHODS.iter().map(|m| m.name)
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
    /// Method name; must appear in [`IPC_METHODS`].
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

/// Typed IPC errors. Each variant maps to a stable [`ErrorCode`]; see
/// [`IpcError::error_code`]. The four T3 defense rungs surface here:
/// allowlist-miss (rung 1), cap-missing (rung 2), session-resolve-
/// failure (rung 1 of br-r1-14 cross-protocol contract), and CSP-load-
/// failure (rung 3, used by the integrator-binary boot path).
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum IpcError {
    /// Method is not in [`IPC_METHODS`]. The webview requested a
    /// method the native shell does not expose. T3 defense rung 1.
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

    /// True iff the manifest satisfies `req`.
    ///
    /// [`CapRequirement::None`] is always satisfied (no cap gate —
    /// e.g. `ui.notify`). [`CapRequirement::Required`]`(scope)` is
    /// satisfied iff `scope` is in [`Self::granted_caps`]. The former
    /// empty-string-as-"no cap" sentinel is gone: the two states are
    /// now structurally distinct, so a missing binding can never be
    /// confused with an intentional no-cap method.
    #[must_use]
    pub fn grants(&self, req: &CapRequirement) -> bool {
        match req {
            CapRequirement::None => true,
            CapRequirement::Required(scope) => self.granted_caps.contains(*scope),
        }
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
    manifest: AdminUiManifest,
    bridge: Option<InProcessSessionBridge>,
}

impl std::fmt::Debug for TauriRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TauriRenderer")
            .field("allowlist_methods", &IPC_METHODS.len())
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
        ipc_method_names().map(str::to_string).collect()
    }

    /// Dispatch (gate) an IPC request through the three T3 defense
    /// rungs:
    ///
    /// 1. Allowlist filter — method-name MUST be in
    ///    [`IPC_METHODS`] (rung 1).
    /// 2. Capability binding — admin UI v0 manifest envelope MUST
    ///    grant the method's [`CapRequirement`] (rung 2).
    /// 3. Session resolution — if a bridge is attached, the session
    ///    token resolves to the authoritative principal DID; origin
    ///    pinning + expiry check fire here (br-r1-14 cross-protocol
    ///    contract).
    ///
    /// The CSP rung (3 of the T3 defense composition) fires at
    /// webview-load time via [`Self::webview_csp_header`], NOT here —
    /// CSP is a load-boundary defense, not a per-call defense.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the request passes all three rungs. This crate's
    /// job is purely to *gate* the request — it carries no response
    /// data. The method-specific handler lives in the integrator
    /// binary's Tauri command handler, which owns the response shape
    /// entirely (it calls back into engine facade methods with the
    /// resolved principal and returns the real payload to the webview).
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] when any of the three rungs reject. The
    /// integrator binary surfaces the error to the webview as an
    /// opaque error code; no diagnostic reason crosses the IPC
    /// boundary (operator-only audit per
    /// [`benten_engine::thin_client`] module doc).
    pub fn dispatch_ipc(&self, request: IpcRequest) -> Result<(), IpcError> {
        // (1) Allowlist filter (T3 rung 1) + (2) cap binding (T3 rung
        // 2) share ONE lookup over the typed `IPC_METHODS` slice. The
        // presence of a result IS rung 1 (allowlist membership); the
        // result's `CapRequirement` IS rung 2. Rejects BEFORE any
        // payload parse so an attacker-crafted payload can't pivot
        // through a forbidden method.
        let Some(method) = ipc_method(&request.method) else {
            return Err(IpcError::MethodNotInAllowlist {
                method: request.method,
            });
        };

        // (2) Capability binding (T3 rung 2). The manifest envelope is
        // consulted against the method's typed `CapRequirement`. There
        // is no fail-OPEN fallback: a `CapRequirement::None` admits by
        // construction (the `ui.notify` case), and a missing binding is
        // unrepresentable — `ipc_method` either returns a fully-bound
        // `IpcMethod` or `None` (rung 1 reject above). The former
        // `unwrap_or("")` drift hazard cannot occur.
        if !self.manifest.grants(&method.cap) {
            let cap = match method.cap {
                CapRequirement::Required(scope) => scope.to_string(),
                CapRequirement::None => String::new(),
            };
            return Err(IpcError::CapabilityNotInManifest {
                method: request.method,
                cap,
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
        // facade methods with the resolved principal and owns the
        // response shape entirely.
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipc_method_lookup_covers_every_entry_and_rejects_unknown() {
        for m in IPC_METHODS {
            assert_eq!(
                ipc_method(m.name).map(|found| found.name),
                Some(m.name),
                "missing: {}",
                m.name
            );
        }
        assert!(ipc_method("engine.arbitrary").is_none());
    }

    #[test]
    fn cap_requirement_none_only_for_ui_notify() {
        // Structural invariant: `ui.notify` is the sole no-cap method;
        // every other method carries `Required(scope)`. The former
        // empty-string sentinel made this checkable only by string
        // comparison; the typed sum makes it a `matches!`.
        for m in IPC_METHODS {
            match m.cap {
                CapRequirement::None => assert_eq!(m.name, "ui.notify"),
                CapRequirement::Required(scope) => {
                    assert!(!scope.is_empty(), "{} has empty required scope", m.name);
                }
            }
        }
    }

    #[test]
    fn dispatch_rung2_fails_closed_for_required_cap_not_granted() {
        // The structural fail-CLOSED guarantee (formerly Safe-1 #499's
        // `unwrap_or("")` fail-OPEN hazard): a method with a required
        // cap the manifest does NOT grant rejects, never admits.
        let renderer = TauriRenderer::new_with_manifest(AdminUiManifest::default());
        let result = renderer.dispatch_ipc(IpcRequest {
            method: "engine.call_as".to_string(),
            payload: serde_json::Value::Null,
            session: None,
        });
        assert!(
            matches!(result, Err(IpcError::CapabilityNotInManifest { .. })),
            "rung-2 must fail CLOSED on ungranted required cap, got {result:?}"
        );
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

}
