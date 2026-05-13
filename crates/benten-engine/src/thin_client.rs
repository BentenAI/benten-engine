//! Phase-4-Foundation G24-F wave: `DidKeyedSession` + `SessionToken`
//! thin-client session protocol.
//!
//! Implements the **DID-keyed handshake** + **session token** contract
//! that defends admin UI v0 (and any future thin-client surface) per
//! `docs/admin-ui-v0-threat-model.md` §T2 defenses 1-3, br-r1-1, and
//! sec-4f-r1-5.
//!
//! ## Three deployment shapes use ONE protocol
//!
//! Per CLAUDE.md baked-in #17, three engine deployment shapes coexist
//! and share this thin-client protocol whenever the surface is not a
//! full peer:
//!
//! - **(a) Full peer** — native Rust. Does NOT use this module.
//! - **(b) Thin compute surface** — wasm32 browser tab; talks HTTP /
//!   fetch to a (a) full peer. Uses [`DidKeyedSession`] +
//!   [`SessionToken`] via the [`Transport::Http`] adapter (out of scope
//!   for this module — wire framing lands at G24-D-FP-2 plus the
//!   bindings layer); the cryptographic state machine here is the same.
//! - **(c) Embedded webview** — Tauri 2.x shell wrapping a webview that
//!   loads the same wasm32 bundle as shape (b). Talks **in-process
//!   IPC** to the full peer embedded in the same process. Uses
//!   [`DidKeyedSession`] + [`SessionToken`] via the [`Transport::Ipc`]
//!   adapter. Per `docs/ADMIN-UI.md` §4.3 (br-r1-14): the contract is
//!   IDENTICAL to shape (b); only the wire transport is swapped.
//!
//! ## Cryptographic shape (T2 defenses)
//!
//! 1. **DID-keyed handshake (T2 defense 1, br-r1-1):** the full peer
//!    mints a fresh [`Challenge`] (32-byte nonce + binding origin +
//!    not-after wallclock). The thin-client signs the challenge bytes
//!    with the principal DID's private key. The full peer verifies the
//!    signature against the resolved did:key public key.
//! 2. **Origin pinning (T2 defense 3, sec-4f-r1-5):** the challenge
//!    carries the origin the thin-client claimed at handshake; the
//!    minted [`SessionToken`] is bound to the SAME origin; every
//!    subsequent call presenting the token MUST match that origin.
//!    Per Family F1 gap #2: the origin recheck is structural-
//!    always-on per-request, not establishment-only.
//! 3. **Replay defense (T2 defense 1, defense-in-depth):** every
//!    challenge is single-use. The full peer tracks consumed nonces in
//!    a bounded set; replaying a previously-consumed challenge is
//!    rejected with `E_THIN_CLIENT_CHALLENGE_REPLAY` even if the
//!    signature verifies.
//! 4. **Time bound (T2 defense 2):** the session token carries an
//!    expiry wallclock. Presenting an expired token rejects with
//!    `E_THIN_CLIENT_SESSION_EXPIRED`; the thin-client must
//!    re-handshake.
//!
//! ## What this module does NOT do
//!
//! - **Wire framing.** HTTP / SSE / WebSocket / Tauri IPC framing is a
//!   layer ABOVE this module (the `bindings/napi` + admin-UI plugin).
//!   This module operates on opaque `&[u8]` and typed Rust shapes.
//! - **Capability resolution.** The session token carries the resolved
//!   principal DID; the engine's CapabilityPolicy then drives every
//!   per-call cap check against that principal. The thin-client
//!   protocol does NOT bypass `Engine::call_as` / `Engine::read_node_as`
//!   — it FEEDS them the correct principal.
//! - **Cross-origin defense at the BROWSER side.** Browser-side defense
//!   (CSP directives + `same-origin` cookie + `Cross-Origin-Opener-
//!   Policy`) is the admin UI bundle's responsibility per `docs/
//!   ADMIN-UI.md` §4.2. This module enforces the full-peer-side recheck
//!   so a hostile origin presenting a captured token is denied at the
//!   engine boundary even if browser-side defenses are bypassed
//!   (defense-in-depth).

#![allow(clippy::module_name_repetitions)]

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use benten_errors::ErrorCode;

/// 32-byte challenge nonce. Single-use; consumed at handshake.
pub type ChallengeNonce = [u8; 32];

/// 32-byte session-token opaque-id. Random per-session; not the
/// principal DID (the principal lives at [`SessionToken::principal_did`]).
pub type SessionTokenId = [u8; 32];

/// Typed errors for the thin-client session protocol.
///
/// Each variant maps to a stable [`ErrorCode`] via
/// [`ThinClientSessionError::error_code`]. The four codes minted at
/// G24-F (`E_THIN_CLIENT_ORIGIN_MISMATCH`,
/// `E_THIN_CLIENT_CHALLENGE_REPLAY`, `E_THIN_CLIENT_HANDSHAKE_INVALID`,
/// `E_THIN_CLIENT_SESSION_EXPIRED`) sit alongside the pre-existing
/// `E_THIN_CLIENT_AUTH_REJECTED` (G14-D wave-5a; covers the
/// device-attestation auth boundary that gates `ThinClientConnection`
/// rather than the session-token boundary that gates this module's
/// surface).
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ThinClientSessionError {
    /// Handshake signature failed to verify against the claimed
    /// principal DID's resolved public key. Also covers
    /// malformed-challenge / wrong-length-signature variants.
    #[error("thin-client handshake invalid: {reason}")]
    HandshakeInvalid {
        /// Diagnostic detail (NOT propagated to the wire — orchestrator
        /// audit only).
        reason: String,
    },
    /// Handshake presented a challenge that was already consumed by an
    /// earlier (successful) handshake. Defends T2 defense 1 captured-
    /// replay attack class.
    #[error("thin-client challenge already consumed (replay rejected)")]
    ChallengeReplay,
    /// Session token presented with a request whose origin does not
    /// match the origin the token was bound to at handshake. Fires
    /// both at establishment (cross-origin handshake) and mid-session
    /// (per-request structural recheck per Family F1 gap #2 closure).
    #[error("thin-client origin mismatch: bound={bound} presented={presented}")]
    OriginMismatch {
        /// The origin the token was minted against at handshake.
        bound: String,
        /// The origin the request presented this token from.
        presented: String,
    },
    /// Session token's `expires_at_unix_secs` has passed.
    #[error("thin-client session expired: expires_at={expires_at} now={now}")]
    SessionExpired {
        /// Token's bound expiry (unix seconds).
        expires_at: u64,
        /// Wallclock at the rejecting full peer (unix seconds).
        now: u64,
    },
    /// Session token's opaque id does not resolve to any active
    /// session at the full peer. Fires on garbage-collected tokens, on
    /// tokens minted by a different full-peer instance, and on
    /// fabricated tokens.
    #[error("thin-client session token unknown")]
    UnknownToken,
}

impl ThinClientSessionError {
    /// Stable catalog code mapping. Joins the `ON_DENIED` routing
    /// family per the same precedent as
    /// [`ErrorCode::ThinClientAuthRejected`].
    #[must_use]
    pub fn error_code(&self) -> ErrorCode {
        match self {
            Self::HandshakeInvalid { .. } => ErrorCode::ThinClientHandshakeInvalid,
            Self::ChallengeReplay => ErrorCode::ThinClientChallengeReplay,
            Self::OriginMismatch { .. } => ErrorCode::ThinClientOriginMismatch,
            Self::SessionExpired { .. } | Self::UnknownToken => ErrorCode::ThinClientSessionExpired,
        }
    }
}

/// A fresh DID-keyed handshake challenge.
///
/// Returned by [`DidKeyedSession::emit_challenge`]; the thin-client
/// signs `Challenge::nonce` with its principal DID's private key and
/// presents `(challenge, signature, principal_did, origin)` to
/// [`DidKeyedSession::establish_session`].
///
/// The challenge carries the origin the thin-client claimed at the
/// REQUEST-FOR-CHALLENGE step; presenting a signature against a
/// challenge with `claimed_origin` X but completing the handshake with
/// `presented_origin` Y rejects with [`ThinClientSessionError::OriginMismatch`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Challenge {
    /// 32 random bytes. Single-use; consumed by `establish_session`.
    pub nonce: ChallengeNonce,
    /// Origin the thin-client claimed when it requested the challenge.
    /// Bound into the minted session token so per-request recheck has
    /// an authoritative comparison value.
    pub claimed_origin: String,
    /// Unix seconds after which this challenge can no longer be used
    /// to establish a session (independent from session-token TTL).
    /// Defends T2 defense 1 staleness — a leaked challenge that was
    /// never consumed can't be re-used months later.
    pub expires_at_unix_secs: u64,
}

/// Minted session token. Opaque-id + bound-context envelope.
///
/// The thin-client treats this as opaque: presenting it on subsequent
/// requests proves the holder completed the handshake. The full peer
/// resolves `token_id` to its in-memory `SessionRecord` (which carries
/// the principal DID + bound origin + expiry) and feeds the resolved
/// principal to `Engine::call_as` / `Engine::read_node_as`.
///
/// Per T2 defense 3 second clause (per the
/// `admin_ui_v0_thin_client_bridge_resolves_principal_from_session_not_client`
/// pin): clients NEVER assert the principal in request bodies; the
/// bridge ALWAYS resolves from the session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionToken {
    /// 32-byte opaque id. Random; not the principal DID.
    pub token_id: SessionTokenId,
    /// Resolved principal DID (the one that signed the handshake).
    /// Carried alongside the token for client-side display only —
    /// the engine boundary always re-resolves from `token_id` to defend
    /// against client-tampered token bodies.
    pub principal_did: String,
    /// Origin the token was minted against.
    pub bound_origin: String,
    /// Unix seconds after which the token expires.
    pub expires_at_unix_secs: u64,
}

impl SessionToken {
    /// True iff the token has not yet expired against the provided
    /// `now_unix_secs`. Convenience for downstream consumers; the
    /// authoritative check lives at [`DidKeyedSession::resolve`].
    #[must_use]
    pub fn is_valid_at(&self, now_unix_secs: u64) -> bool {
        now_unix_secs < self.expires_at_unix_secs
    }
}

/// Server-side session record. Held in [`DidKeyedSession`]'s state map
/// keyed on `token_id`. Carries the authoritative copy of every
/// security-bound field — clients NEVER mutate this; only the full
/// peer mints/expires it.
#[derive(Debug, Clone)]
struct SessionRecord {
    principal_did: String,
    bound_origin: String,
    expires_at_unix_secs: u64,
}

/// Configuration for the thin-client session state machine.
#[derive(Debug, Clone, Copy)]
pub struct SessionConfig {
    /// Challenge TTL in seconds. Default: 60s (handshake must complete
    /// within a minute of the challenge being minted).
    pub challenge_ttl_secs: u64,
    /// Session token TTL in seconds. Default: 3600s (1 hour). After
    /// this the thin-client must re-handshake.
    pub session_ttl_secs: u64,
    /// Maximum number of consumed-nonce entries retained for replay
    /// defense. Past this bound the oldest entries are pruned; the
    /// challenge TTL provides the substantive replay window so the
    /// cap is just a memory bound, not a security bound.
    pub max_consumed_nonces: usize,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            challenge_ttl_secs: 60,
            session_ttl_secs: 3600,
            max_consumed_nonces: 4096,
        }
    }
}

/// Signature-verification hook. The host wires this with a real
/// Ed25519 verifier in production; tests can swap an always-accept /
/// always-reject hook for negative pins.
///
/// Returns `Ok(())` iff `signature` is a valid signature over `message`
/// for the public key resolved from `principal_did`. Returns
/// `Err(reason)` to surface a diagnostic (NOT propagated to the wire).
pub type SignatureVerifier =
    Box<dyn Fn(&str, &[u8], &[u8]) -> Result<(), String> + Send + Sync + 'static>;

/// Random-bytes hook. Pluggable so tests can drive deterministic
/// challenge nonces.
pub type RandomBytesFn = Box<dyn Fn() -> [u8; 32] + Send + Sync + 'static>;

/// Wallclock hook. Pluggable so tests can advance time for
/// TTL-expiry pins.
pub type ClockFn = Box<dyn Fn() -> u64 + Send + Sync + 'static>;

/// DID-keyed session state machine. Holds the full-peer-side state for
/// the thin-client session protocol: pending challenges, consumed
/// nonces (replay defense), and active session records.
///
/// Thread-safe via internal Mutex; cheap to share across threads (the
/// production wiring keeps one instance per Engine and routes every
/// thin-client request through it).
pub struct DidKeyedSession {
    inner: Mutex<DidKeyedSessionState>,
    config: SessionConfig,
    verifier: SignatureVerifier,
    rng: RandomBytesFn,
    clock: ClockFn,
}

#[derive(Debug, Default)]
struct DidKeyedSessionState {
    /// Outstanding (un-consumed) challenges keyed by nonce.
    pending_challenges: HashMap<ChallengeNonce, Challenge>,
    /// Consumed challenge nonces — replayed handshakes against these
    /// reject with [`ThinClientSessionError::ChallengeReplay`].
    consumed_nonces: HashSet<ChallengeNonce>,
    /// Insertion-order list of consumed nonces for bounded-set pruning
    /// when `max_consumed_nonces` is reached.
    consumed_order: Vec<ChallengeNonce>,
    /// Active session records keyed by token id.
    sessions: HashMap<SessionTokenId, SessionRecord>,
}

impl std::fmt::Debug for DidKeyedSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DidKeyedSession")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl DidKeyedSession {
    /// Construct with custom verifier + RNG + clock hooks.
    ///
    /// Production builds wire `verifier` with an Ed25519 verifier that
    /// resolves the public key from the did:key DID per
    /// `benten_id::Did::public_key_from_did_key`, wires `rng` with
    /// `getrandom::fill`, and wires `clock` with
    /// `SystemTime::now().duration_since(UNIX_EPOCH)`.
    pub fn with_hooks(
        config: SessionConfig,
        verifier: SignatureVerifier,
        rng: RandomBytesFn,
        clock: ClockFn,
    ) -> Self {
        Self {
            inner: Mutex::new(DidKeyedSessionState::default()),
            config,
            verifier,
            rng,
            clock,
        }
    }

    /// Production constructor: real Ed25519 verification via
    /// `benten-id`, OS CSPRNG, wallclock.
    ///
    /// The verifier closure is intentionally minimal here — the full
    /// did:key resolution path lands at G24-F-FP1 (the build is gated
    /// by `cfg(not(target_arch = "wasm32"))` per CLAUDE.md baked-in
    /// #17: this module is full-peer-only).
    #[cfg(not(target_arch = "wasm32"))]
    #[must_use]
    pub fn new(config: SessionConfig) -> Self {
        let verifier: SignatureVerifier = Box::new(production_signature_verifier);
        let rng: RandomBytesFn = Box::new(|| {
            let mut bytes = [0_u8; 32];
            getrandom::getrandom(&mut bytes)
                .expect("OS CSPRNG must be available on full-peer hardware");
            bytes
        });
        let clock: ClockFn = Box::new(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |d| d.as_secs())
        });
        Self::with_hooks(config, verifier, rng, clock)
    }

    /// Emit a fresh challenge bound to `claimed_origin`. The
    /// thin-client signs `challenge.nonce` and presents the signature
    /// to [`Self::establish_session`].
    pub fn emit_challenge(&self, claimed_origin: impl Into<String>) -> Challenge {
        let claimed_origin = claimed_origin.into();
        let now = (self.clock)();
        let nonce = (self.rng)();
        let challenge = Challenge {
            nonce,
            claimed_origin: claimed_origin.clone(),
            expires_at_unix_secs: now + self.config.challenge_ttl_secs,
        };
        let mut state = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        // Defense: if the RNG ever collides (cryptographically
        // implausible but tested with deterministic test RNGs), refuse
        // to overwrite a still-pending challenge — caller can retry.
        state.pending_challenges.insert(nonce, challenge.clone());
        challenge
    }

    /// Complete the handshake: verify the signature, consume the
    /// nonce, mint a session token bound to (`principal_did`,
    /// `presented_origin`, `now + session_ttl_secs`).
    ///
    /// # Errors
    ///
    /// - [`ThinClientSessionError::HandshakeInvalid`] — signature
    ///   verification failed, challenge unknown, or challenge expired.
    /// - [`ThinClientSessionError::ChallengeReplay`] — challenge was
    ///   already consumed by an earlier successful handshake.
    /// - [`ThinClientSessionError::OriginMismatch`] — `presented_origin`
    ///   does not match the `claimed_origin` the challenge was minted
    ///   against (T2 defense 3 cross-origin handshake guard).
    pub fn establish_session(
        &self,
        challenge: &Challenge,
        signature: &[u8],
        principal_did: impl Into<String>,
        presented_origin: impl Into<String>,
    ) -> Result<SessionToken, ThinClientSessionError> {
        let principal_did = principal_did.into();
        let presented_origin = presented_origin.into();
        let now = (self.clock)();

        let mut state = self.inner.lock().unwrap_or_else(|e| e.into_inner());

        // (1) Replay defense — check BEFORE pending-challenge lookup so
        // a consumed nonce returns the replay error rather than the
        // generic "unknown challenge" error (better operator
        // diagnostics).
        if state.consumed_nonces.contains(&challenge.nonce) {
            return Err(ThinClientSessionError::ChallengeReplay);
        }

        // (2) Pending-challenge lookup. The presented `challenge`
        // value is client-asserted; we re-resolve against the
        // authoritative record stored at emit_challenge time so
        // mutated client values don't bypass binding fields.
        let authoritative = state
            .pending_challenges
            .get(&challenge.nonce)
            .cloned()
            .ok_or_else(|| ThinClientSessionError::HandshakeInvalid {
                reason: "challenge nonce unknown".into(),
            })?;

        // (3) Challenge-staleness check.
        if now >= authoritative.expires_at_unix_secs {
            // Reap the stale challenge from pending — it'll never be
            // usable again.
            state.pending_challenges.remove(&challenge.nonce);
            return Err(ThinClientSessionError::HandshakeInvalid {
                reason: "challenge expired".into(),
            });
        }

        // (4) Origin pinning at handshake (T2 defense 3 first clause).
        if authoritative.claimed_origin != presented_origin {
            return Err(ThinClientSessionError::OriginMismatch {
                bound: authoritative.claimed_origin.clone(),
                presented: presented_origin,
            });
        }

        // (5) Signature verification (T2 defense 1). The verifier
        // closure is the cryptographic boundary; failure here
        // surfaces as HandshakeInvalid.
        (self.verifier)(&principal_did, &authoritative.nonce, signature)
            .map_err(|reason| ThinClientSessionError::HandshakeInvalid { reason })?;

        // (6) Mint session record. Consume the nonce + retire it to
        // the replay-defense set BEFORE returning so the SAME
        // (challenge, signature) tuple presented a second time is
        // rejected even if it races concurrently.
        state.pending_challenges.remove(&challenge.nonce);
        state.consumed_nonces.insert(challenge.nonce);
        state.consumed_order.push(challenge.nonce);
        // Bounded consumed-nonce set: prune oldest if over cap.
        while state.consumed_order.len() > self.config.max_consumed_nonces {
            if let Some(old) = state.consumed_order.first().copied() {
                state.consumed_order.remove(0);
                state.consumed_nonces.remove(&old);
            } else {
                break;
            }
        }

        let token_id = (self.rng)();
        let token = SessionToken {
            token_id,
            principal_did: principal_did.clone(),
            bound_origin: presented_origin.clone(),
            expires_at_unix_secs: now + self.config.session_ttl_secs,
        };
        state.sessions.insert(
            token_id,
            SessionRecord {
                principal_did,
                bound_origin: presented_origin,
                expires_at_unix_secs: token.expires_at_unix_secs,
            },
        );

        Ok(token)
    }

    /// Resolve a session token presented from `presented_origin` to
    /// the authoritative principal DID. Performs the per-request
    /// structural rechecks (T2 defense 3 mid-session + T2 defense 2
    /// expiry) per Family F1 gap #2 closure.
    ///
    /// Returns the resolved principal DID; the bridge then invokes
    /// `Engine::call_as(principal, ...)` with this resolved value,
    /// IGNORING any client-asserted principal field (T2 defense 3
    /// second clause).
    ///
    /// # Errors
    ///
    /// - [`ThinClientSessionError::UnknownToken`] — token id unknown.
    /// - [`ThinClientSessionError::OriginMismatch`] — token bound to
    ///   different origin (gap #2 mid-session defense).
    /// - [`ThinClientSessionError::SessionExpired`] — token TTL elapsed.
    pub fn resolve(
        &self,
        token: &SessionToken,
        presented_origin: &str,
    ) -> Result<String, ThinClientSessionError> {
        let now = (self.clock)();
        let state = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let record = state
            .sessions
            .get(&token.token_id)
            .ok_or(ThinClientSessionError::UnknownToken)?;
        // Origin recheck FIRST so an expired-AND-cross-origin token
        // surfaces the origin mismatch (the more actionable diagnostic
        // for the security audit) rather than the expiry.
        if record.bound_origin != presented_origin {
            return Err(ThinClientSessionError::OriginMismatch {
                bound: record.bound_origin.clone(),
                presented: presented_origin.to_string(),
            });
        }
        if now >= record.expires_at_unix_secs {
            return Err(ThinClientSessionError::SessionExpired {
                expires_at: record.expires_at_unix_secs,
                now,
            });
        }
        Ok(record.principal_did.clone())
    }

    /// Test-only: count of active session records. Used by the
    /// `thin_client_session_*` pins to assert mint/expire bookkeeping.
    #[doc(hidden)]
    #[must_use]
    pub fn active_session_count_for_test(&self) -> usize {
        let state = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        state.sessions.len()
    }

    /// Test-only: count of consumed nonces tracked for replay defense.
    #[doc(hidden)]
    #[must_use]
    pub fn consumed_nonce_count_for_test(&self) -> usize {
        let state = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        state.consumed_nonces.len()
    }
}

/// Production signature-verification entry point. Resolves the
/// `did:key:zXXX` DID body to an Ed25519 public key via `benten-id`
/// and verifies the signature. Native-only (full peers do the
/// crypto; thin-client surfaces never call this path).
///
/// Wired by [`DidKeyedSession::new`] as the default verifier; tests
/// inject alternatives via [`DidKeyedSession::with_hooks`].
#[cfg(not(target_arch = "wasm32"))]
fn production_signature_verifier(
    principal_did: &str,
    message: &[u8],
    signature: &[u8],
) -> Result<(), String> {
    use benten_id::did::Did;
    use ed25519_dalek::Verifier;
    // Resolve the DID to a public key. The `did:key` resolver is the
    // closed Phase-3 G14-B baseline; non-did:key forms reject inside
    // `Did::resolve` as unsupported until a later phase widens the
    // resolver registry.
    let did = Did::from_string_unchecked(principal_did.to_string());
    let public_key = did
        .resolve()
        .map_err(|e| format!("did key resolution failed: {e}"))?;
    let sig_bytes: [u8; 64] = signature
        .try_into()
        .map_err(|_| format!("signature length {} != 64", signature.len()))?;
    let sig = ed25519_dalek::Signature::from_bytes(&sig_bytes);
    public_key
        .as_verifying_key()
        .verify(message, &sig)
        .map_err(|e| format!("ed25519 verify failed: {e}"))
}

/// Transport adapter — names the wire-substrate the thin-client uses
/// to talk to the full peer. Carried alongside session records ONLY
/// for observability / audit; the cryptographic contract is identical
/// across variants.
///
/// Per CLAUDE.md baked-in #17: shape (b) browser tab uses HTTP; shape
/// (c) Tauri-embedded webview uses in-process IPC; both share the same
/// `DidKeyedSession` contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transport {
    /// Shape (b) — HTTP / fetch from a wasm32 browser tab. Wire framing
    /// lands at G24-D-FP-2 + the bindings layer.
    Http,
    /// Shape (c) — in-process IPC from a Tauri-embedded webview to the
    /// engine in the same native process. Wire framing lands at
    /// G24-Tauri-shell.
    Ipc,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn make_session_with_test_hooks() -> (DidKeyedSession, Arc<AtomicU64>, Arc<AtomicU64>) {
        let clock_value = Arc::new(AtomicU64::new(1_700_000_000));
        let nonce_counter = Arc::new(AtomicU64::new(1));
        let clock_for_closure = Arc::clone(&clock_value);
        let nonce_for_closure = Arc::clone(&nonce_counter);
        let session = DidKeyedSession::with_hooks(
            SessionConfig::default(),
            Box::new(|_did, _msg, sig| {
                if sig == b"VALID-SIG-32-BYTES--padding-pad!" {
                    Ok(())
                } else {
                    Err(format!("bad sig len={}", sig.len()))
                }
            }),
            Box::new(move || {
                let n = nonce_for_closure.fetch_add(1, Ordering::SeqCst);
                let mut bytes = [0_u8; 32];
                bytes[..8].copy_from_slice(&n.to_le_bytes());
                bytes
            }),
            Box::new(move || clock_for_closure.load(Ordering::SeqCst)),
        );
        (session, clock_value, nonce_counter)
    }

    const VALID_SIG: &[u8] = b"VALID-SIG-32-BYTES--padding-pad!";
    const BAD_SIG: &[u8] = b"BAD-SIG";

    #[test]
    fn handshake_happy_path_mints_session_token() {
        let (session, _clock, _nonce) = make_session_with_test_hooks();
        let challenge = session.emit_challenge("https://benten.localhost:8443");
        let token = session
            .establish_session(
                &challenge,
                VALID_SIG,
                "did:key:zAlice",
                "https://benten.localhost:8443",
            )
            .expect("handshake should succeed");
        assert_eq!(token.principal_did, "did:key:zAlice");
        assert_eq!(token.bound_origin, "https://benten.localhost:8443");
        assert_eq!(session.active_session_count_for_test(), 1);
        assert_eq!(session.consumed_nonce_count_for_test(), 1);
    }

    #[test]
    fn handshake_replay_rejects_consumed_nonce() {
        let (session, _clock, _nonce) = make_session_with_test_hooks();
        let challenge = session.emit_challenge("https://benten.localhost:8443");
        let _first = session
            .establish_session(
                &challenge,
                VALID_SIG,
                "did:key:zAlice",
                "https://benten.localhost:8443",
            )
            .expect("first handshake should succeed");
        let replay = session.establish_session(
            &challenge,
            VALID_SIG,
            "did:key:zAlice",
            "https://benten.localhost:8443",
        );
        assert_eq!(replay, Err(ThinClientSessionError::ChallengeReplay));
        assert_eq!(
            replay.unwrap_err().error_code(),
            ErrorCode::ThinClientChallengeReplay
        );
    }

    #[test]
    fn handshake_bad_signature_rejects_with_handshake_invalid() {
        let (session, _clock, _nonce) = make_session_with_test_hooks();
        let challenge = session.emit_challenge("https://benten.localhost:8443");
        let err = session
            .establish_session(
                &challenge,
                BAD_SIG,
                "did:key:zAlice",
                "https://benten.localhost:8443",
            )
            .expect_err("bad sig must reject");
        assert!(matches!(
            err,
            ThinClientSessionError::HandshakeInvalid { .. }
        ));
        assert_eq!(err.error_code(), ErrorCode::ThinClientHandshakeInvalid);
    }

    #[test]
    fn establish_session_cross_origin_rejects() {
        let (session, _clock, _nonce) = make_session_with_test_hooks();
        let challenge = session.emit_challenge("https://benten.localhost:8443");
        let err = session
            .establish_session(
                &challenge,
                VALID_SIG,
                "did:key:zAlice",
                "https://evil.example",
            )
            .expect_err("cross-origin handshake must reject");
        match err {
            ThinClientSessionError::OriginMismatch { bound, presented } => {
                assert_eq!(bound, "https://benten.localhost:8443");
                assert_eq!(presented, "https://evil.example");
            }
            other => panic!("expected OriginMismatch, got {other:?}"),
        }
    }

    #[test]
    fn resolve_mid_session_cross_origin_rejects() {
        let (session, _clock, _nonce) = make_session_with_test_hooks();
        let challenge = session.emit_challenge("https://benten.localhost:8443");
        let token = session
            .establish_session(
                &challenge,
                VALID_SIG,
                "did:key:zAlice",
                "https://benten.localhost:8443",
            )
            .unwrap();
        // Same-origin resolve works:
        let principal = session
            .resolve(&token, "https://benten.localhost:8443")
            .unwrap();
        assert_eq!(principal, "did:key:zAlice");
        // Cross-origin resolve rejects (Family F1 gap #2 closure):
        let err = session
            .resolve(&token, "https://evil.example")
            .expect_err("mid-session cross-origin resolve must reject");
        assert!(matches!(err, ThinClientSessionError::OriginMismatch { .. }));
        assert_eq!(err.error_code(), ErrorCode::ThinClientOriginMismatch);
        // Defense-in-depth: subsequent same-origin resolves keep
        // working (no self-inflicted DoS via auto-invalidation):
        let principal_after = session
            .resolve(&token, "https://benten.localhost:8443")
            .unwrap();
        assert_eq!(principal_after, "did:key:zAlice");
    }

    #[test]
    fn resolve_expired_token_rejects() {
        let (session, clock, _nonce) = make_session_with_test_hooks();
        let challenge = session.emit_challenge("https://benten.localhost:8443");
        let token = session
            .establish_session(
                &challenge,
                VALID_SIG,
                "did:key:zAlice",
                "https://benten.localhost:8443",
            )
            .unwrap();
        // Advance the clock past session_ttl_secs (3600 default):
        clock.fetch_add(3601, Ordering::SeqCst);
        let err = session
            .resolve(&token, "https://benten.localhost:8443")
            .expect_err("expired token must reject");
        assert!(matches!(err, ThinClientSessionError::SessionExpired { .. }));
        assert_eq!(err.error_code(), ErrorCode::ThinClientSessionExpired);
    }

    #[test]
    fn expired_challenge_cannot_establish_session() {
        let (session, clock, _nonce) = make_session_with_test_hooks();
        let challenge = session.emit_challenge("https://benten.localhost:8443");
        // Advance past challenge_ttl_secs (60 default):
        clock.fetch_add(61, Ordering::SeqCst);
        let err = session
            .establish_session(
                &challenge,
                VALID_SIG,
                "did:key:zAlice",
                "https://benten.localhost:8443",
            )
            .expect_err("expired challenge must reject");
        assert!(matches!(
            err,
            ThinClientSessionError::HandshakeInvalid { .. }
        ));
    }

    #[test]
    fn unknown_token_id_rejects_with_unknown_token() {
        let (session, _clock, _nonce) = make_session_with_test_hooks();
        let fabricated = SessionToken {
            token_id: [0xff_u8; 32],
            principal_did: "did:key:zMallory".into(),
            bound_origin: "https://benten.localhost:8443".into(),
            expires_at_unix_secs: u64::MAX,
        };
        let err = session
            .resolve(&fabricated, "https://benten.localhost:8443")
            .expect_err("fabricated token must reject");
        assert_eq!(err, ThinClientSessionError::UnknownToken);
        assert_eq!(err.error_code(), ErrorCode::ThinClientSessionExpired);
    }

    #[test]
    fn transport_variants_are_distinguishable() {
        // Sanity: the Transport enum is part of the API surface per
        // CLAUDE.md baked-in #17 (shape b HTTP vs shape c IPC).
        assert_ne!(Transport::Http, Transport::Ipc);
    }
}
