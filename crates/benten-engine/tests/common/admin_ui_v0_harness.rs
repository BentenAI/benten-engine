//! Phase-4-Foundation R3 Family F1 — admin UI v0 test harness scaffolding.
//!
//! ## Two shapes
//!
//! - [`AdminUiV0TestHarness::new`] — **G24-B-FP-1 graduation** (this
//!   wave). Composed-engine + materializer end-to-end harness used by
//!   the T1 hostile-schema READ+EMIT defense pins +
//!   T7 private-namespace isolation pin. Wires:
//!   - real [`benten_engine::Engine`] with `capability_policy_grant_backed`
//!     so private-namespace delegation refusal fires on real grants
//!   - admin-UI plugin-DID + hostile-plugin-DID principal pair
//!   - materializer-pipeline dispatch through an in-test
//!     `MaterializerEngine` adapter that routes reads via the
//!     Class B β seam [`benten_engine::Engine::read_node_as`]
//!     (CLAUDE.md baked-in #18 cag-r1-9)
//!   - cap-grant minting + delegation attempt seam
//!     ([`AdminUiV0TestHarness::attempt_cross_plugin_delegation`])
//! - [`AdminUiV0TestHarness::new_thin_client_against_full_peer`] — the
//!   G24-F thin-client variant; unchanged this wave. Backs the
//!   T2 session-protocol pins (`thin_client_*` methods).
//!
//! The two shapes don't share state — pick the constructor that matches
//! your pin's surface. The `new` shape's `session` field is `None`; the
//! thin-client shape's `engine` / `tempdir` / principal fields are
//! `None`.
//!
//! ## What the G24-F (thin-client) surface exposes
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

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use benten_core::{Cid, Node, Value};
use benten_engine::thin_client::{
    Challenge, DidKeyedSession, SessionConfig, SessionToken, ThinClientSessionError,
};
use benten_engine::{Engine, EngineError};
use benten_platform_foundation::{
    HtmlJsonMaterializer, Materializer, MaterializerCapRecheck, MaterializerEngine,
    MaterializerError, MaterializerOutput, MaterializerWalkInputs, SchemaSubgraphSpec,
    allow_all_cap_recheck,
};

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

/// Plugin-DID string the harness uses for the admin-UI plugin.
/// `did:key`-shaped (compatible with [`Engine::delegate_capability`]'s
/// `Did::from_string_unchecked` path).
pub const HARNESS_ADMIN_UI_PLUGIN_DID: &str = "did:key:z6MkAdminUiV0PluginHarnessFixture12345";

/// Plugin-DID string the harness uses for the "hostile" / "other"
/// plugin DID — used for cross-plugin delegation refusal pins (T7).
pub const HARNESS_HOSTILE_PLUGIN_DID: &str = "did:key:z6MkHostilePluginForCrossPluginPin12345";

/// Composed harness for admin UI v0 integration tests.
///
/// Two construction shapes:
/// - [`AdminUiV0TestHarness::new`] — composed engine + materializer
///   end-to-end (G24-B-FP-1 graduation; T1 + T7 pins consume).
/// - [`AdminUiV0TestHarness::new_thin_client_against_full_peer`] —
///   thin-client session-protocol surface (G24-F; T2 pins consume).
pub struct AdminUiV0TestHarness {
    /// Deterministic test clock shared with the underlying session
    /// (thin-client shape only; `None` for `new`).
    clock: Arc<AtomicU64>,
    /// Deterministic nonce counter shared with the underlying session
    /// (thin-client shape only).
    nonce_counter: Arc<AtomicU64>,
    /// DidKeyedSession backing the full-peer-side state machine.
    /// `None` for the `new()` composed-engine shape.
    session: Option<DidKeyedSession>,
    /// Default principal DID for handshakes (thin-client shape).
    principal_did: String,
    /// Default origin the harness binds challenges + tokens to.
    default_origin: String,
    /// Fake "graph" for `put_test_node` — returns deterministic CID
    /// strings the harness can pass to `thin_client_read_with_session`.
    fake_cid_counter: Arc<AtomicU64>,

    // ---- Composed-engine shape (G24-B-FP-1) ----
    /// Real `benten_engine::Engine` for the composed-engine shape.
    /// `None` for the thin-client shape.
    engine: Option<Engine>,
    /// Tempdir backing the engine's redb store. Kept alive while the
    /// harness exists; dropped (cleaning up the on-disk DB) at Drop.
    _tempdir: Option<tempfile::TempDir>,
    /// Admin-UI plugin-DID-shaped principal CID (for `call_as` /
    /// `read_node_as` walk principals).
    admin_ui_plugin_principal: Option<Cid>,
    /// "Other plugin" / hostile-plugin principal CID — used for the
    /// T7 cross-plugin delegation refusal pin.
    hostile_plugin_principal: Option<Cid>,
}

impl AdminUiV0TestHarness {
    /// G24-B-FP-1 graduation: composed-engine + materializer harness
    /// for the T1 (hostile-schema READ+EMIT) + T7 (private-namespace
    /// isolation) end-to-end pins.
    ///
    /// Wires:
    /// - `benten_engine::Engine` with `capability_policy_grant_backed`
    ///   so the private-namespace structural defense at
    ///   `Engine::delegate_capability` fires
    /// - admin-UI plugin-DID + hostile-plugin-DID `system:Principal`
    ///   Nodes minted via `Engine::create_principal`
    /// - materializer-pipeline dispatch through a per-call
    ///   `HarnessEngineAdapter` that routes reads through
    ///   `Engine::read_node_as` (Class B β seam)
    ///
    /// # Panics
    ///
    /// Panics if the tempdir / Engine open / principal mint fails —
    /// the harness is test-only fixture machinery, not a production
    /// surface; an open-fail makes the entire pin meaningless.
    #[must_use]
    pub fn new() -> Self {
        let tempdir = tempfile::tempdir().expect("harness: tempdir");
        let engine = Engine::builder()
            .capability_policy_grant_backed()
            .open(tempdir.path().join("admin-ui-v0-harness.redb"))
            .expect("harness: engine opens with grant-backed policy");
        let admin_ui_plugin_principal = engine
            .create_principal("admin-ui-v0-plugin-harness-principal")
            .expect("harness: admin-UI plugin principal");
        let hostile_plugin_principal = engine
            .create_principal("hostile-plugin-harness-principal")
            .expect("harness: hostile plugin principal");
        Self {
            clock: Arc::new(AtomicU64::new(1_700_000_000_u64)),
            nonce_counter: Arc::new(AtomicU64::new(1)),
            session: None,
            principal_did: HARNESS_ADMIN_UI_PLUGIN_DID.into(),
            default_origin: HARNESS_DEFAULT_ORIGIN.into(),
            fake_cid_counter: Arc::new(AtomicU64::new(1)),
            engine: Some(engine),
            _tempdir: Some(tempdir),
            admin_ui_plugin_principal: Some(admin_ui_plugin_principal),
            hostile_plugin_principal: Some(hostile_plugin_principal),
        }
    }

    /// Borrow the composed engine. Panics for the thin-client shape.
    #[must_use]
    pub fn engine(&self) -> &Engine {
        self.engine
            .as_ref()
            .expect("AdminUiV0TestHarness: composed engine not initialised; call new()")
    }

    /// Admin-UI plugin-DID principal CID (for `Engine::read_node_as`
    /// + `Engine::call_as` walk principal).
    #[must_use]
    pub fn admin_ui_plugin_principal_cid(&self) -> Cid {
        self.admin_ui_plugin_principal
            .expect("AdminUiV0TestHarness: composed engine shape required")
    }

    /// Hostile / "other" plugin principal CID.
    #[must_use]
    pub fn hostile_plugin_principal_cid(&self) -> Cid {
        self.hostile_plugin_principal
            .expect("AdminUiV0TestHarness: composed engine shape required")
    }

    /// Admin-UI plugin DID string (`did:key:...` shape) for
    /// `Engine::grant_capability` / `Engine::delegate_capability` —
    /// the `&str` shape these APIs consume.
    #[must_use]
    pub fn admin_ui_plugin_did_str(&self) -> &'static str {
        HARNESS_ADMIN_UI_PLUGIN_DID
    }

    /// Hostile / "other" plugin DID string.
    #[must_use]
    pub fn hostile_plugin_did_str(&self) -> &'static str {
        HARNESS_HOSTILE_PLUGIN_DID
    }

    /// Persist a fresh Node into the engine (writes via the engine's
    /// public seam, then reads back the CID). Used by the T1 +
    /// regression-guard pins as the content_cid for the materializer
    /// walk.
    ///
    /// # Errors
    /// Surfaces [`EngineError`] verbatim.
    pub fn create_test_node(&self, node: &Node) -> Result<Cid, EngineError> {
        self.engine().create_node(node)
    }

    /// Mint a `system:CapabilityGrant` Node via
    /// [`Engine::grant_capability`]. Returns the grant CID for the
    /// T7 delegation-refusal pin (caller then passes the CID to
    /// [`Self::attempt_cross_plugin_delegation`]).
    ///
    /// # Errors
    /// Surfaces [`EngineError`] verbatim.
    pub fn mint_user_rooted_grant(
        &self,
        actor_plugin_did: &str,
        scope: &str,
    ) -> Result<Cid, EngineError> {
        self.engine().grant_capability(actor_plugin_did, scope)
    }

    /// Grant the admin-UI principal a cap-scope keyed to its principal
    /// CID (not the DID string). This is the form
    /// [`benten_caps::grant_backed::GrantBackedPolicy::check_read`]
    /// consults via `ctx.actor_cid` — when the benign-regression-
    /// guard pin renders a Node with label `"Note"`, the grant
    /// `store:Note:read` admits the read through the Class B β seam.
    ///
    /// # Errors
    /// Surfaces [`EngineError`] verbatim.
    pub fn grant_admin_ui_read_scope(&self, scope: &str) -> Result<Cid, EngineError> {
        let principal = self.admin_ui_plugin_principal_cid();
        self.engine().grant_capability(&principal, scope)
    }

    /// Attempt to delegate a capability across plugin boundaries via
    /// [`Engine::delegate_capability`]. For T7 the source grant's
    /// scope is a `private:<plugin_did>:*` cap; the engine MUST
    /// reject with `E_PLUGIN_PRIVATE_NAMESPACE_DELEGATION_FORBIDDEN`
    /// per CLAUDE.md baked-in #18 private-namespace clause +
    /// `crates/benten-caps/src/plugin_delegation.rs` structural rule.
    ///
    /// # Errors
    /// Surfaces [`EngineError`] verbatim — for the T7 path the caller
    /// asserts this is an `EngineError::Other { code:
    /// PluginPrivateNamespaceDelegationForbidden, .. }`.
    pub fn attempt_cross_plugin_delegation(
        &self,
        source_grant_cid: &Cid,
        target_plugin_did: &str,
    ) -> Result<Cid, EngineError> {
        self.engine()
            .delegate_capability(source_grant_cid, target_plugin_did, &[])
    }

    /// Dispatch an admin-UI v0 render walk against a real materializer
    /// pipeline backed by the harness's `Engine`. The walk-principal is
    /// the admin-UI plugin-DID principal CID; the read path threads
    /// through [`Engine::read_node_as`] — the Class B β seam.
    ///
    /// `declared_requires` is the schema's declared cap-scope envelope
    /// per the materializer's T1 envelope-recheck rule
    /// (`materializer.rs` lines 884-904): if any primitive in the spec
    /// carries a `cap_scope` outside this list, the materializer
    /// refuses the walk with `MaterializerError::SchemaMismatch`. Pass
    /// an empty `Vec` to bypass the envelope check (e.g. for the
    /// benign regression-guard arm where the schema's cap-scopes are
    /// allow-listed by being absent from declared_requires).
    ///
    /// `cap_recheck` is the materialization-layer per-row gate —
    /// callers wire [`allow_all_cap_recheck`] for the T1 envelope-only
    /// pin or a recording closure for invocation-count observability.
    ///
    /// # Errors
    /// Surfaces [`MaterializerError`] verbatim.
    pub fn render_admin_ui_with_envelope(
        &self,
        spec: &SchemaSubgraphSpec,
        content_cid: Cid,
        declared_requires: Vec<String>,
        cap_recheck: MaterializerCapRecheck,
    ) -> Result<MaterializerOutput, MaterializerError> {
        let adapter = HarnessEngineAdapter::new(self.engine());
        let walk_principal = self.admin_ui_plugin_principal_cid();
        HtmlJsonMaterializer.materialize_with_gate(MaterializerWalkInputs {
            engine: &adapter,
            spec,
            content_cid,
            walk_principal,
            cap_recheck,
            declared_requires,
        })
    }

    /// Capture per-cap-scope `cap_recheck` invocations — used by the
    /// T1 regression-guard to assert structural always-on per-row
    /// rechecks (Compromise #11 closure floor).
    #[must_use]
    pub fn recording_cap_recheck() -> (MaterializerCapRecheck, Arc<AtomicUsize>) {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_for_closure = Arc::clone(&counter);
        let gate: MaterializerCapRecheck = Arc::new(move |_p: &Cid, _z: &str, _c: &Cid| {
            counter_for_closure.fetch_add(1, Ordering::SeqCst);
            true
        });
        (gate, counter)
    }
}

/// In-test [`MaterializerEngine`] adapter for the composed-engine
/// harness shape. Routes EVERY read through
/// [`benten_engine::Engine::read_node_as`] — never the engine-internal
/// `pub(crate) Engine::read_node` seam (cag-r1-9 + CLAUDE.md #18).
///
/// Mirrors the test-side adapter at
/// `crates/benten-platform-foundation/tests/common/admin_ui_v0_engine_adapter.rs`
/// — kept inline to the harness so admin-UI integration tests in
/// `crates/benten-engine/tests/` consume it without a cross-crate test
/// helper hop.
struct HarnessEngineAdapter<'a> {
    engine: &'a Engine,
    clock_injected: bool,
}

impl<'a> HarnessEngineAdapter<'a> {
    fn new(engine: &'a Engine) -> Self {
        Self {
            engine,
            // Engine::open does not auto-inject a clock; the harness
            // declares "injected" so the materializer's clock fail-
            // closed posture (sec-3.5-r1-7) does not short-circuit
            // these pins — the pins are about the T1 envelope /
            // private-NS delegation defenses, not the clock posture.
            clock_injected: true,
        }
    }
}

impl<'a> MaterializerEngine for HarnessEngineAdapter<'a> {
    fn read_node_as(&self, principal: &Cid, cid: &Cid) -> Result<Option<Node>, MaterializerError> {
        self.engine
            .read_node_as(principal, cid)
            .map_err(|e| MaterializerError::SchemaMismatch {
                code: benten_errors::ErrorCode::MaterializerSchemaMismatch,
                reason: format!("harness engine read_node_as backend error: {e}"),
            })
    }

    fn has_clock_injected(&self) -> bool {
        self.clock_injected
    }
}

impl AdminUiV0TestHarness {
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
            engine: None,
            _tempdir: None,
            admin_ui_plugin_principal: None,
            hostile_plugin_principal: None,
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

/// Helper for tests that want to author a benign Node payload backed by
/// a single `"body"` text property.
#[must_use]
pub fn make_note_node(body: &str) -> Node {
    let mut props = BTreeMap::new();
    props.insert("body".into(), Value::Text(body.into()));
    props.insert(
        "created_at".into(),
        Value::Text("2026-05-13T00:00:00Z".into()),
    );
    Node::new(vec!["Note".into()], props)
}

/// Helper exposing [`allow_all_cap_recheck`] from the harness module so
/// pins don't need a separate `use benten_platform_foundation::...`
/// import when they only need this one closure shape.
#[must_use]
pub fn allow_all_gate() -> MaterializerCapRecheck {
    allow_all_cap_recheck()
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
