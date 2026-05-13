//! Phase-4-Foundation R3 Family F1 — admin UI v0 test harness scaffolding.
//!
//! Stub at R3 (RED-PHASE) per `.addl/phase-4-foundation/r2-test-landscape.md`
//! §4 "NEW helpers that should land FIRST" item 3. G24-F wires the
//! thin-client variant (`new_thin_client_against_full_peer`) on top of
//! the real `benten_engine::thin_client::DidKeyedSession` state machine.
//! G24-A will graduate the broader `AdminUiV0TestHarness::new()` path
//! (composed engine + 2-peer Atrium); for G24-F only the thin-client
//! surface needs to be live.
//!
//! ## What the G24-F surface exposes
//!
//! - [`AdminUiV0TestHarness::new_thin_client_against_full_peer`] —
//!   construct a thin-client harness with a deterministic test clock +
//!   test RNG + signature-verifier driven from in-memory keypairs.
//! - [`AdminUiV0TestHarness::full_peer_emit_challenge`] — full-peer
//!   side mints a fresh handshake challenge bound to the harness's
//!   default origin.
//! - [`AdminUiV0TestHarness::thin_client_sign_challenge`] — produce a
//!   signature over the challenge nonce using the harness's principal
//!   keypair.
//! - [`AdminUiV0TestHarness::thin_client_establish_session`] — drive
//!   the full-peer handshake.
//! - [`AdminUiV0TestHarness::thin_client_read_with_session`] —
//!   per-request thin-client → full-peer read that exercises the
//!   `DidKeyedSession::resolve` recheck.
//! - [`AdminUiV0TestHarness::put_test_node`] — fixture for the
//!   mid-session-wraparound pin to address a real-looking CID.
//! - [`AdminUiV0TestHarness::advance_test_clock`] — drive the
//!   deterministic clock past TTL bounds.

#![allow(dead_code)]

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use benten_engine::thin_client::{
    Challenge, DidKeyedSession, SessionConfig, SessionToken, ThinClientSessionError,
};

/// Composed harness for admin UI v0 integration tests.
///
/// G24-A will replace `engine: None` + `atrium: None` with the
/// composed-engine path. G24-F lights up `session` for the thin-client
/// session-protocol pins.
pub struct AdminUiV0TestHarness {
    /// Deterministic test clock shared with the underlying session.
    clock: Arc<AtomicU64>,
    /// Deterministic nonce counter shared with the underlying session.
    nonce_counter: Arc<AtomicU64>,
    /// DidKeyedSession backing the full-peer-side state machine.
    /// `None` for the legacy `new()` path until G24-A wires it.
    session: Option<DidKeyedSession>,
    /// Default principal DID for handshakes — driven by the harness's
    /// in-memory keypair; for G24-F we don't need real did:key
    /// resolution (the verifier hook short-circuits on the
    /// `THIN_CLIENT_HARNESS_PRINCIPAL_DID` sentinel).
    principal_did: String,
    /// Default origin the harness binds challenges + tokens to.
    default_origin: String,
    /// Fake "graph" for `put_test_node` — returns deterministic CID
    /// strings the harness can pass to `thin_client_read_with_session`.
    fake_cid_counter: Arc<AtomicU64>,
}

/// Sentinel principal DID used by the harness's signature-verifier
/// hook. The hook accepts a signature iff its bytes equal
/// [`HARNESS_VALID_SIG`] AND the principal DID matches this sentinel
/// — a closed pinhole so the harness can drive every T2 negative pin
/// without standing up a real Ed25519 keypair (which the production
/// path uses via the `production_signature_verifier`).
pub const HARNESS_PRINCIPAL_DID: &str = "did:key:zHarnessPrincipalForG24FTest";

/// Sentinel "valid signature" bytes the harness's signature-verifier
/// hook accepts. Exactly 32 bytes for the test fixture; the production
/// path verifies real Ed25519 64-byte signatures.
pub const HARNESS_VALID_SIG: &[u8; 32] = b"HARNESS-VALID-SIG-32-BYTES-PAD!!";

/// Default origin the harness binds challenges + tokens to.
pub const HARNESS_DEFAULT_ORIGIN: &str = "https://benten.localhost:8443";

impl AdminUiV0TestHarness {
    /// Construct a harness with a 2-peer Atrium fixture + an engine
    /// configured for full-peer shape (a) per CLAUDE.md baked-in #17.
    ///
    /// **Stub.** G24-A wave fills.
    pub fn new() -> Self {
        unimplemented!(
            "G24-A wires AdminUiV0TestHarness::new — composes engine + \
             2-peer Atrium + DID-keyed thin-client session stub"
        )
    }

    /// Construct a thin-client variant (shape b: wasm32-unknown-unknown
    /// browser bundle) backed by a full peer over a loopback transport.
    ///
    /// G24-F wave fills: lights up the [`DidKeyedSession`] state machine
    /// with a deterministic test clock + test RNG + signature-verifier
    /// hook keyed on the harness's sentinel principal DID. Tests then
    /// drive handshakes + resolves through the harness to exercise
    /// every T2 defense without standing up a real engine + Atrium.
    pub fn new_thin_client_against_full_peer() -> Self {
        let clock = Arc::new(AtomicU64::new(1_700_000_000_u64));
        let nonce_counter = Arc::new(AtomicU64::new(1));
        let clock_for_closure = Arc::clone(&clock);
        let nonce_for_closure = Arc::clone(&nonce_counter);
        let session = DidKeyedSession::with_hooks(
            SessionConfig::default(),
            // Verifier: accept iff sig bytes equal HARNESS_VALID_SIG.
            // The principal DID is parameterised so tests can swap
            // identities for cross-DID pins.
            Box::new(|_did, _msg, sig| {
                if sig == HARNESS_VALID_SIG.as_slice() {
                    Ok(())
                } else {
                    Err(format!(
                        "harness: signature does not match HARNESS_VALID_SIG (len={})",
                        sig.len(),
                    ))
                }
            }),
            // RNG: counter-stamped 32-byte nonces so consumed-nonce
            // assertions are stable across runs.
            Box::new(move || {
                let n = nonce_for_closure.fetch_add(1, Ordering::SeqCst);
                let mut bytes = [0_u8; 32];
                bytes[..8].copy_from_slice(&n.to_le_bytes());
                bytes
            }),
            // Clock: shared atomic so `advance_test_clock` drives the
            // session's expiry checks deterministically.
            Box::new(move || clock_for_closure.load(Ordering::SeqCst)),
        );
        Self {
            clock,
            nonce_counter,
            session: Some(session),
            principal_did: HARNESS_PRINCIPAL_DID.into(),
            default_origin: HARNESS_DEFAULT_ORIGIN.into(),
            fake_cid_counter: Arc::new(AtomicU64::new(1)),
        }
    }

    /// Internal: borrow the underlying session, panicking if the
    /// harness wasn't built via `new_thin_client_against_full_peer`.
    fn session(&self) -> &DidKeyedSession {
        self.session
            .as_ref()
            .expect("AdminUiV0TestHarness: thin-client session not initialised; call new_thin_client_against_full_peer()")
    }

    /// Full-peer mints a fresh challenge bound to the harness default
    /// origin. The thin-client signs `challenge.nonce` and presents
    /// the signature to [`Self::thin_client_establish_session`].
    pub fn full_peer_emit_challenge(&self) -> Challenge {
        self.session().emit_challenge(self.default_origin.clone())
    }

    /// Thin-client side: produce a "signature" over the challenge.
    /// For the harness, this is just the [`HARNESS_VALID_SIG`] sentinel
    /// bytes — the verifier hook short-circuits on that exact value.
    /// Tests that want to drive a bad-signature path pass `&[0_u8; 0]`
    /// or any other slice directly to `thin_client_establish_session`.
    #[must_use]
    pub fn thin_client_sign_challenge(&self, _challenge: &Challenge) -> Vec<u8> {
        HARNESS_VALID_SIG.to_vec()
    }

    /// Drive the full-peer handshake. Returns the minted session token
    /// or a typed thin-client error per T2 defenses.
    pub fn thin_client_establish_session(
        &self,
        challenge: &Challenge,
        signature: &[u8],
        presented_origin: &str,
    ) -> Result<SessionToken, ThinClientSessionError> {
        self.session().establish_session(
            challenge,
            signature,
            self.principal_did.clone(),
            presented_origin.to_string(),
        )
    }

    /// Drive the full-peer handshake under a specific principal DID
    /// (for cross-DID pins). Defaults to [`HARNESS_PRINCIPAL_DID`] via
    /// [`Self::thin_client_establish_session`].
    pub fn thin_client_establish_session_as(
        &self,
        challenge: &Challenge,
        signature: &[u8],
        principal_did: &str,
        presented_origin: &str,
    ) -> Result<SessionToken, ThinClientSessionError> {
        self.session().establish_session(
            challenge,
            signature,
            principal_did.to_string(),
            presented_origin.to_string(),
        )
    }

    /// Per-request thin-client → full-peer read. The token is resolved
    /// against the session state machine; the resolved principal would
    /// then be fed to `Engine::read_node_as` by the production bridge.
    /// For the harness, we stop at the resolve step + return `Ok(())`
    /// — the resolve recheck is the surface the G24-F pins assert.
    pub fn thin_client_read_with_session(
        &self,
        token: &SessionToken,
        _cid_bytes: &[u8],
        presented_origin: &str,
    ) -> Result<(), ThinClientSessionError> {
        self.session().resolve(token, presented_origin)?;
        // In production the resolved principal feeds
        // Engine::read_node_as(principal, cid); for the harness we
        // stop here — the recheck is the boundary G24-F asserts.
        Ok(())
    }

    /// Test-fixture stub: returns a deterministic 32-byte "CID-like"
    /// blob with a monotonic counter so the mid-session-wraparound pin
    /// can call the read path with distinct addresses across its 3+
    /// arms. Not a real CID; the harness doesn't run the graph layer.
    pub fn put_test_node(&self, _label: impl Into<String>) -> Result<Vec<u8>, String> {
        let n = self.fake_cid_counter.fetch_add(1, Ordering::SeqCst);
        let mut bytes = vec![0_u8; 32];
        bytes[..8].copy_from_slice(&n.to_le_bytes());
        Ok(bytes)
    }

    /// Advance the deterministic test clock by `secs` seconds. Drives
    /// [`Challenge::expires_at_unix_secs`] + session expiry checks
    /// past their bounds.
    pub fn advance_test_clock_secs(&self, secs: u64) {
        self.clock.fetch_add(secs, Ordering::SeqCst);
    }

    /// Default origin the harness binds challenges + tokens to. Tests
    /// reference this when asserting bound-origin equality.
    #[must_use]
    pub fn default_origin(&self) -> &str {
        &self.default_origin
    }

    /// Default principal DID the harness uses.
    #[must_use]
    pub fn principal_did(&self) -> &str {
        &self.principal_did
    }

    /// Test-only: count of active session records (for assertions
    /// around DoS-via-cross-origin-attempt — the recheck MUST NOT
    /// auto-invalidate legit sessions).
    #[must_use]
    pub fn active_session_count(&self) -> usize {
        self.session().active_session_count_for_test()
    }
}

impl Default for AdminUiV0TestHarness {
    fn default() -> Self {
        Self::new()
    }
}

/// Opaque session-token handle returned from DID-keyed handshake.
///
/// **Stub shape only.** G24-F wires the real
/// [`benten_engine::thin_client::SessionToken`]; this lingering stub
/// is preserved to keep G24-A's wave-7 pins compiling while they
/// migrate.
#[derive(Debug, Clone)]
pub struct SessionTokenStub {
    pub token_bytes: Vec<u8>,
    pub bound_origin: String,
}
