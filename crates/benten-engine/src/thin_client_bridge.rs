//! Phase-4-Meta-Core G-CORE-8 §4.22 — thin-client bridge that
//! resolves the acting principal from the authenticated DID-keyed
//! session (NEVER from client input).
//!
//! # What this seam does
//!
//! Per CLAUDE.md baked-in #17/#18: a thin-client (shape b/c — browser
//! tab or embedded webview) is a VIEW into a full peer via the
//! authenticated thin-client protocol; it is NOT a sync participant
//! and it does NOT hold cap tokens. The bridge resolves the acting
//! principal from the authenticated SESSION (the DID-keyed handshake)
//! — NOT from anything the client supplies in-band. A client that
//! asserts "I am principal X" through any in-band channel MUST NOT
//! thereby become X; the principal is whatever the session was
//! established as.
//!
//! # The structural defense
//!
//! [`ThinClientBridge::resolve_principal_for_request`] takes ONLY
//! `(session_token, presented_origin)` — there is no client-supplied
//! principal field on the API surface. A client cannot self-elevate
//! by asserting `principal = X` because the API has no such
//! parameter. The bridge resolves the session token against the
//! engine's [`crate::thin_client::DidKeyedSession`] (the G24-F
//! SHIPPED session surface) and returns either the bound principal-
//! DID OR a typed
//! [`benten_errors::ErrorCode::ThinClientBridgePrincipalUnresolved`]
//! reject when the session is unknown, expired, origin-mismatched, or
//! otherwise un-resolvable.
//!
//! # Engine ↔ host-shell runtime boundary (j)
//!
//! Per CLAUDE.md baked-in #17 + §8-C dual-runtime contract: the
//! engine owns its tokio runtime; the host shell owns its own; they
//! communicate across an explicit channel/IPC boundary. The bridge
//! NEVER threads a host-shell runtime handle through the engine — the
//! bridge surface holds NO runtime types (no `tokio::Handle`, no
//! `tokio::task::Spawner`, no `tauri::Runtime`-class shapes). The
//! bridge is async by construction (the session resolve call is
//! sync — it's an in-memory lookup against `DidKeyedSession` — but
//! the bridge entry point is `pub fn` not `async fn` so it composes
//! across either runtime's executor without ceremony). The j
//! constraint is satisfied by the API surface alone (no runtime
//! coupling possible).
//!
//! # No cap tokens cross the boundary
//!
//! The bridge consumes a [`crate::thin_client::SessionToken`] (an
//! opaque correlation handle bound to a server-side
//! principal-DID + origin + expiry) and returns ONLY the principal-
//! DID string. NO UCAN / capability-bundle / grant artifact crosses
//! the boundary. The Class-B-β `Engine::read_node_as` consumer on the
//! engine side takes the resolved principal-DID and walks the graph
//! "as" that principal — cap-tokens stay engine-internal.
//!
//! # G-CORE-8 deliverable scope
//!
//! Lands the [`ThinClientBridge`] type + the
//! `resolve_principal_for_request` API + the typed reject ErrorCode.
//! The actual wiring into a specific host-shell transport (Tauri IPC,
//! HTTP, WebSocket) is a host-shell-adapter concern that lives
//! OUTSIDE the engine (per CLAUDE.md #17 three deployment shapes —
//! the bridge is shape-agnostic; each host-shell adapter consumes
//! this bridge over its own transport).

use std::sync::Arc;

use benten_errors::ErrorCode;

use crate::EngineError;
use crate::thin_client::{DidKeyedSession, SessionToken, ThinClientSessionError};

/// The thin-client bridge — owns a reference to the full peer's
/// [`DidKeyedSession`] surface and resolves session tokens to their
/// bound principal-DIDs.
///
/// One instance per engine; the engine constructs it lazily from its
/// owned `DidKeyedSession`.
///
/// # Construction
///
/// Use [`Self::new`] passing an `Arc<DidKeyedSession>` (the engine's
/// owned session surface). The bridge does not own the session — it
/// references it through `Arc` so multiple host-shell adapters can
/// share one bridge instance over the same session state.
pub struct ThinClientBridge {
    session: Arc<DidKeyedSession>,
}

impl std::fmt::Debug for ThinClientBridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThinClientBridge")
            .field("session", &"<Arc<DidKeyedSession>>")
            .finish()
    }
}

impl ThinClientBridge {
    /// Construct a bridge over the engine's owned session surface.
    #[must_use]
    pub fn new(session: Arc<DidKeyedSession>) -> Self {
        Self { session }
    }

    /// Resolve the acting principal for a thin-client request.
    ///
    /// Takes ONLY `(session_token, presented_origin)` — no
    /// client-supplied principal field by construction. A client
    /// cannot assert "I am principal X" through this API; the
    /// principal is whatever the session was established as.
    ///
    /// # Errors
    ///
    /// - [`ErrorCode::ThinClientBridgePrincipalUnresolved`] when the
    ///   session is unknown / expired / origin-mismatched / otherwise
    ///   un-resolvable. The wrapped [`EngineError`] carries the
    ///   typed session-error reason in its message for forensic
    ///   correlation.
    pub fn resolve_principal_for_request(
        &self,
        token: &SessionToken,
        presented_origin: &str,
    ) -> Result<String, EngineError> {
        match self.session.resolve(token, presented_origin) {
            Ok(principal_did) => Ok(principal_did),
            Err(reason) => Err(EngineError::Other {
                code: ErrorCode::ThinClientBridgePrincipalUnresolved,
                message: format!(
                    "thin-client bridge: cannot resolve acting principal from \
                     authenticated session (token_id_hex='{}' presented_origin='{}' \
                     reason='{}'): G-CORE-8 §4.22 + CLAUDE.md baked-in #17/#18 — the bridge \
                     resolves the principal from the server-side session ONLY; no \
                     client-supplied principal field exists on this API",
                    short_hex(&token.token_id),
                    presented_origin,
                    SessionErrorRender(&reason),
                ),
            }),
        }
    }

    /// Read-only access to the underlying session surface (test
    /// observable + host-shell-adapter introspection).
    #[must_use]
    pub fn session(&self) -> &DidKeyedSession {
        &self.session
    }
}

/// Render the first 8 bytes of the token-id as hex for forensic
/// correlation in error messages (the full 32 bytes would be noisy +
/// the prefix is sufficient to distinguish minted tokens). Inline
/// implementation avoids adding a `hex` dep for one call site.
fn short_hex(bytes: &[u8; 32]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(16);
    for b in &bytes[..8] {
        let _ = write!(out, "{b:02x}");
    }
    out
}

/// Helper to render a [`ThinClientSessionError`] for the bridge's
/// error message without coupling to the session crate's `Display`
/// surface (which may carry more detail than the bridge wants to
/// expose to the client). Keeps the bridge message stable as the
/// session error variants grow.
struct SessionErrorRender<'a>(&'a ThinClientSessionError);

impl<'a> std::fmt::Display for SessionErrorRender<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Render the variant name only (not the full error chain).
        // `ThinClientSessionError` is `#[non_exhaustive]` across the
        // crate boundary; the wildcard arm is the forward-compat
        // fall-through (currently unreachable but defends against a
        // future variant addition without requiring a rebuild here).
        #[allow(unreachable_patterns)]
        match self.0 {
            ThinClientSessionError::HandshakeInvalid { .. } => f.write_str("HandshakeInvalid"),
            ThinClientSessionError::ChallengeReplay => f.write_str("ChallengeReplay"),
            ThinClientSessionError::OriginMismatch { .. } => f.write_str("OriginMismatch"),
            ThinClientSessionError::SessionExpired { .. } => f.write_str("SessionExpired"),
            ThinClientSessionError::UnknownToken => f.write_str("UnknownToken"),
            _ => f.write_str("SessionError"),
        }
    }
}

// `ThinClientBridge` MUST NOT thread any host-shell runtime handle/
// ownership through the engine (§8-C / §1.A.FROZEN item 13 / (j)
// constraint). The compile-time backstop: the struct's only field is
// `Arc<DidKeyedSession>` — no `tokio::Handle`, no `tokio::task::
// Spawner`, no `tauri::Runtime`-class shapes. The bridge entry point
// is `pub fn`, not `async fn`, so it composes across either runtime's
// executor without binding a `Handle`. This file imports NOTHING from
// tokio or any host-shell-runtime library; a regression that added
// such an import would fire on the workspace deny.toml's host-shell-
// banned-list (the cross-tool config mirror per §3.5g item #4).

#[cfg(test)]
mod tests {
    use super::*;
    use crate::thin_client::SessionConfig;
    use std::sync::atomic::{AtomicU64, Ordering};

    const VALID_SIG: &[u8; 4] = b"sig!";

    fn test_session() -> Arc<DidKeyedSession> {
        let clock = Arc::new(AtomicU64::new(1_700_000_000_u64));
        let nonce_counter = Arc::new(AtomicU64::new(1));
        let clock_for_closure = Arc::clone(&clock);
        let nonce_for_closure = Arc::clone(&nonce_counter);
        Arc::new(DidKeyedSession::with_hooks(
            SessionConfig::default(),
            Box::new(|_did, _msg, sig| {
                if sig == VALID_SIG.as_slice() {
                    Ok(())
                } else {
                    Err("bad sig".to_string())
                }
            }),
            Box::new(move || {
                let n = nonce_for_closure.fetch_add(1, Ordering::SeqCst);
                let mut bytes = [0_u8; 32];
                bytes[..8].copy_from_slice(&n.to_le_bytes());
                bytes
            }),
            Box::new(move || clock_for_closure.load(Ordering::SeqCst)),
        ))
    }

    fn establish_for(session: &DidKeyedSession, principal_did: &str, origin: &str) -> SessionToken {
        let challenge = session.emit_challenge(origin.to_string());
        session
            .establish_session(
                &challenge,
                VALID_SIG.as_slice(),
                principal_did.to_string(),
                origin.to_string(),
            )
            .expect("test session establishes")
    }

    #[test]
    fn bridge_resolves_to_session_bound_principal() {
        let session = test_session();
        let bridge = ThinClientBridge::new(Arc::clone(&session));
        let origin = "https://benten.localhost:8443";
        let token = establish_for(&session, "did:key:zUser", origin);
        let principal = bridge
            .resolve_principal_for_request(&token, origin)
            .expect("session bound to did:key:zUser must resolve");
        assert_eq!(principal, "did:key:zUser");
    }

    #[test]
    fn bridge_rejects_cross_origin_with_typed_code() {
        let session = test_session();
        let bridge = ThinClientBridge::new(Arc::clone(&session));
        let origin = "https://benten.localhost:8443";
        let token = establish_for(&session, "did:key:zUser", origin);
        let err = bridge
            .resolve_principal_for_request(&token, "https://attacker.example")
            .expect_err("cross-origin presentation must reject");
        assert_eq!(err.code(), ErrorCode::ThinClientBridgePrincipalUnresolved);
    }

    #[test]
    fn bridge_api_has_no_client_principal_parameter_structural_pin() {
        // Compile-time structural pin: `resolve_principal_for_request`
        // takes EXACTLY (session_token, presented_origin). If a
        // regression added a client-asserted-principal parameter the
        // call below would fail to compile. The call IS the assertion.
        let session = test_session();
        let bridge = ThinClientBridge::new(Arc::clone(&session));
        let origin = "https://benten.localhost:8443";
        let token = establish_for(&session, "did:key:zUser", origin);
        // Exactly TWO arguments — no client-principal slot exists.
        let _ = bridge.resolve_principal_for_request(&token, origin);
    }
}
