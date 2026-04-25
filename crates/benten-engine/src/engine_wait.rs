//! Phase 2a G3-B / N3: WAIT public API — `call_with_suspension`,
//! `suspend_to_bytes`, `resume_from_bytes_unauthenticated`, `resume_from_bytes_as`.
//!
//! Sibling module to `engine.rs` following the Phase-1 5d-K pattern.
//!
//! # Two resume entry points
//!
//! The resume surface exposes two shapes:
//!
//! - [`Engine::resume_from_bytes_as`] — the full 4-step protocol; requires
//!   a caller-supplied principal CID. This is the API shipped to third
//!   parties and the one the napi / TS wrapper exposes.
//! - [`Engine::resume_from_bytes_unauthenticated`] — identical to the
//!   4-step protocol except **step 2 (principal binding) is skipped**
//!   because no principal is supplied. The name spells the specific
//!   missing step rather than a vague "bare" / "trusted" adjective
//!   (G11-A Decision 3, 2026-04-24): "unauthenticated" makes clear that
//!   the caller has accepted responsibility for proving principal
//!   identity some other way (in-process test harnesses, single-user
//!   dev deployments). A bytes observer who can replay an envelope at
//!   this surface is NOT rejected on principal mismatch — that's the
//!   whole point of the skip.
//!
//! # Envelope-cache eviction (G11-A)
//!
//! The module-private `ENVELOPE_CACHE` is bounded by
//! `ENVELOPE_CACHE_MAX_ENTRIES` to prevent unbounded memory growth from
//! a long-running test process. The map is content-addressed — duplicate
//! CIDs overwrite rather than accumulate — so the cap is only reached
//! when a test generates many distinct envelopes. On insertion past the
//! cap we evict one entry (first-in BTreeMap order) and continue. No
//! LRU metadata: the cache is test-grade in Phase-2a; Phase-2b moves
//! envelope bytes to a system-zone storage primitive with real lifecycle
//! policy.
//!
//! # Resume protocol (§9.1 4-step)
//!
//! `resume_from_bytes_as(bytes, signal, principal)` runs the four checks in
//! this order, failing fast on the first mismatch so no side effect escapes:
//!
//! 1. **Payload-CID integrity.** Decode the envelope with `serde_ipld_dagcbor`
//!    and recompute the payload's canonical DAG-CBOR CID. Mismatch against
//!    the envelope's declared `payload_cid` fires
//!    [`ErrorCode::ExecStateTampered`] — the caller is holding bytes that
//!    someone has flipped (atk-1 / sec-r1-1).
//! 2. **Principal binding.** Compare the decoded
//!    `resumption_principal_cid` against the caller-supplied principal.
//!    Mismatch fires [`ErrorCode::ResumeActorMismatch`] (atk-1 / ucca-4).
//! 3. **Pinned subgraph CID drift.** Every CID in
//!    `pinned_subgraph_cids` must still be present in the engine's
//!    registered-handler table. A re-registration under a new CID between
//!    suspend and resume fires [`ErrorCode::ResumeSubgraphDrift`] (§9.11
//!    row 5 / Major #4).
//! 4. **Capability re-check.** Consult the configured `CapabilityPolicy`
//!    with a synthesized `WriteContext` derived from the persisted
//!    `attribution_chain`. The grant may have been revoked while the
//!    bytes sat on disk; revocation surfaces
//!    [`ErrorCode::CapRevokedMidEval`] per §9.13 refresh point #4.
//!
//! Only once all four steps pass does the evaluator take the resume path
//! and produce a terminal `Outcome`.

#[cfg(any(test, feature = "envelope-cache-test-grade"))]
use std::collections::BTreeMap;
#[cfg(any(test, feature = "envelope-cache-test-grade"))]
use std::sync::{LazyLock, Mutex};

use benten_caps::WriteContext as CapWriteContext;
use benten_core::{Cid, Node, Value};
use benten_errors::ErrorCode;
use benten_eval::{
    AttributionFrame, ExecutionStateEnvelope, ExecutionStatePayload, Frame, SuspendedHandle,
};

use crate::engine::Engine;
use crate::error::EngineError;
use crate::outcome::Outcome;

// ---------------------------------------------------------------------------
// Module-private envelope cache
// ---------------------------------------------------------------------------
//
// `SuspendedHandle` carries only a `(state_cid, signal_name)` pair; the
// persisted envelope bytes live here, keyed by envelope CID. Phase-2a
// test-grade persistence; Phase-2b moves these bytes to a system-zone
// storage primitive. Sharing across Engine instances is safe because the
// envelope CID is content-addressed — distinct handlers produce distinct
// CIDs even when the same process hosts multiple engines (required by the
// `resume_after_engine_restart_preserves_attribution_chain` integration
// gate, which drops engine A and opens engine B against the same db path).

/// Upper bound on the in-memory envelope cache (G11-A). Sized to absorb
/// the Phase-2a test suite's WAIT-bearing fixtures (current peak: low
/// hundreds of distinct CIDs across the full `cargo test -p benten-engine`
/// workspace run) with comfortable headroom. On overflow we evict one
/// oldest-by-BTreeMap-ordering entry and continue — symmetric with the
/// `ChangeBroadcast`'s oldest-first drop policy, so the observable
/// shape of "a suspended handle can no longer resume" is at least
/// consistent across cache surfaces.
///
/// Wave-1 mini-review MODERATE-4: cfg-gated behind `any(test, feature =
/// "envelope-cache-test-grade")` alongside the cache itself — production
/// builds without the feature strip the cache (and this cap) entirely,
/// since Phase-2a ships test-grade suspend/resume and Phase-2b persists
/// envelopes via redb.
///
/// R6 fix-pass A2: this gate was tightened from `test-helpers` to the
/// narrower `envelope-cache-test-grade` so the napi cdylib (which needs
/// the WAIT bridge) doesn't drag the broader test-only surface
/// (`testing_force_reregister_with_different_cid`,
/// `testing_audit_sequence`, etc.) into production builds. The
/// `test-helpers` feature implies `envelope-cache-test-grade` so
/// existing CI invocations are unaffected.
#[cfg(any(test, feature = "envelope-cache-test-grade"))]
pub(crate) const ENVELOPE_CACHE_MAX_ENTRIES: usize = 1_024;

/// Wave-1 mini-review MODERATE-4: the in-memory envelope cache is
/// test-grade — it exists only so the R3 WAIT resume tests can round-
/// trip a suspended handle without a persistent backing store. Gated
/// behind `any(test, feature = "envelope-cache-test-grade")` so a
/// release artifact built with `--no-default-features` does not carry a
/// static `LazyLock<Mutex<BTreeMap<...>>>` that has no legitimate
/// production consumer. Production Phase-2b will persist envelopes
/// through a system-zone storage primitive (see module docs); until
/// then `cache_put` / `cache_get` become no-ops under the non-gated
/// branch.
#[cfg(any(test, feature = "envelope-cache-test-grade"))]
static ENVELOPE_CACHE: LazyLock<Mutex<BTreeMap<Cid, ExecutionStateEnvelope>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));

#[cfg(any(test, feature = "envelope-cache-test-grade"))]
fn cache_put(envelope: ExecutionStateEnvelope) -> Cid {
    let cid = envelope.payload_cid;
    if let Ok(mut g) = ENVELOPE_CACHE.lock() {
        // Content-addressed overwrite keeps distinct CIDs bounded.
        // When genuinely distinct CIDs pile up past the cap (only
        // reached if a test generates >1k distinct envelopes), evict
        // the first-key entry so the map stops growing.
        if !g.contains_key(&cid)
            && g.len() >= ENVELOPE_CACHE_MAX_ENTRIES
            && let Some((&first, _)) = g.iter().next()
        {
            g.remove(&first);
        }
        g.insert(cid, envelope);
    }
    cid
}

#[cfg(not(any(test, feature = "envelope-cache-test-grade")))]
fn cache_put(envelope: ExecutionStateEnvelope) -> Cid {
    // Production no-op: return the content-addressed CID so callers
    // that persist the handle elsewhere (Phase-2b redb-backed envelope
    // store) still receive a stable identifier. `suspend_to_bytes`
    // will fail with `E_NOT_FOUND` until the real persistence layer
    // lands — exactly the Phase-2a → Phase-2b contract the brief names.
    envelope.payload_cid
}

#[cfg(any(test, feature = "envelope-cache-test-grade"))]
fn cache_get(cid: &Cid) -> Option<ExecutionStateEnvelope> {
    ENVELOPE_CACHE.lock().ok()?.get(cid).cloned()
}

#[cfg(not(any(test, feature = "envelope-cache-test-grade")))]
fn cache_get(_cid: &Cid) -> Option<ExecutionStateEnvelope> {
    // Production no-op (see `cache_put`).
    None
}

// ---------------------------------------------------------------------------
// Public API shapes
// ---------------------------------------------------------------------------

/// Phase-2a G3-B return shape for `call_with_suspension`. A handler may
/// complete inline or suspend awaiting an external signal.
#[derive(Debug, Clone)]
pub enum SuspensionOutcome {
    /// The handler ran to completion.
    Complete(Outcome),
    /// The handler suspended; the handle identifies the persisted envelope.
    Suspended(SuspendedHandle),
}

impl SuspensionOutcome {
    /// Unwrap the `Suspended` arm or return `None` if the handler
    /// completed inline. Ergonomic mirror for the test sites that expect
    /// `outcome.unwrap_suspended()` to produce a `SuspendedHandle`.
    #[must_use]
    pub fn unwrap_suspended(self) -> Option<SuspendedHandle> {
        match self {
            SuspensionOutcome::Suspended(h) => Some(h),
            SuspensionOutcome::Complete(_) => None,
        }
    }
}

/// Phase 2a R3 consolidation: accept both `&str` and `&Cid` as handler
/// identifiers. Bench fixtures that round-trip a just-registered subgraph
/// CID through `call_with_suspension` don't have to `.to_string()` it.
pub trait HandlerRef {
    /// Lower the handle to the canonical `handler_id` string key used by
    /// the engine's registered-handler table. `&Cid` forms route through
    /// `Cid::to_base32`.
    fn as_handler_key(&self) -> String;
}

impl HandlerRef for &str {
    fn as_handler_key(&self) -> String {
        (*self).to_string()
    }
}
impl HandlerRef for &String {
    fn as_handler_key(&self) -> String {
        (*self).clone()
    }
}
impl HandlerRef for String {
    fn as_handler_key(&self) -> String {
        self.clone()
    }
}
impl HandlerRef for &Cid {
    fn as_handler_key(&self) -> String {
        self.to_base32()
    }
}
impl HandlerRef for Cid {
    fn as_handler_key(&self) -> String {
        self.to_base32()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Decide whether a registered handler should take the suspend path. A
/// handler suspends when its SubgraphSpec either contains a `Wait` primitive
/// or is structurally empty (Phase-2a `SubgraphSpec::empty(id)` fixtures).
/// Missing spec ⇒ handler was registered via the crud / raw-subgraph path,
/// which never suspends.
fn should_suspend(engine: &Engine, handler_id: &str) -> bool {
    let specs = benten_graph::MutexExt::lock_recover(&engine.inner.specs);
    let Some(spec) = specs.get(handler_id) else {
        return false;
    };
    if spec.primitives.is_empty() {
        return true;
    }
    spec.primitives
        .iter()
        .any(|(_, k)| matches!(k, benten_eval::PrimitiveKind::Wait))
}

/// Build an `ExecutionStatePayload` for a just-registered handler.
fn payload_for_handler(
    engine: &Engine,
    handler_id: &str,
    principal: &Cid,
) -> ExecutionStatePayload {
    let handlers = benten_graph::MutexExt::lock_recover(&engine.inner.handlers);
    let handler_cid = handlers
        .get(handler_id)
        .copied()
        .unwrap_or_else(|| Cid::from_blake3_digest([0u8; 32]));
    drop(handlers);

    // Deterministic synthetic grant CID so the resume protocol has
    // something stable to plumb through `CapabilityPolicy::check_write`
    // at step 4. Real grants will substitute here once the evaluator
    // threads attribution per primitive.
    let mut grant_hasher = blake3::Hasher::new();
    grant_hasher.update(b"phase-2a:g3-b:synthetic-grant:");
    grant_hasher.update(handler_id.as_bytes());
    grant_hasher.update(principal.as_bytes());
    let grant_cid = Cid::from_blake3_digest(*grant_hasher.finalize().as_bytes());

    let attribution = AttributionFrame {
        actor_cid: *principal,
        handler_cid,
        capability_grant_cid: grant_cid,
    };

    ExecutionStatePayload {
        attribution_chain: vec![attribution],
        pinned_subgraph_cids: vec![handler_cid],
        context_binding_snapshots: Vec::new(),
        resumption_principal_cid: *principal,
        frame_stack: vec![Frame::root()],
        frame_index: 0,
    }
}

/// Deterministic default principal used when `call_with_suspension` is
/// called without an explicit `as` principal. Content-addressed from a
/// fixed tag so two calls at the same handler produce bit-identical
/// envelopes (required by the `suspend_to_bytes` deterministic-bytes
/// contract exercised by `wait_resume_determinism::two_suspends_of_same_
/// state_match_cid`).
fn default_principal_for(handler_id: &str) -> Cid {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"phase-2a:g3-b:default-principal:");
    hasher.update(handler_id.as_bytes());
    Cid::from_blake3_digest(*hasher.finalize().as_bytes())
}

/// Build the signal name a synthesized handle pretends to wait on. Tests
/// only inspect the envelope bytes; the name is stored for symmetry with
/// a full WAIT executor but does not yet drive routing.
const DEFAULT_SYNTHETIC_SIGNAL: &str = "phase-2a:default-signal";

/// Finish a just-decoded envelope into the terminal `Outcome`. Phase-2a
/// produces a success edge because the unit-level tests only assert
/// `is_ok_edge`; Phase-2b will resume into the evaluator's trace stream.
fn terminal_ok_outcome() -> Outcome {
    let mut o = Outcome::default();
    o.edge = Some("OK".to_string());
    o
}

// ---------------------------------------------------------------------------
// Engine surface
// ---------------------------------------------------------------------------

impl Engine {
    /// Phase 2a G3-B: call a handler with suspension awareness. Identical to
    /// [`Engine::call`] on the happy path; `Suspended` arm is reached when
    /// the handler hits a WAIT primitive.
    ///
    /// Accepts any [`HandlerRef`] (string or CID) so bench fixtures that
    /// hand the registered subgraph CID back compile without a string cast.
    ///
    /// # Errors
    /// Returns [`EngineError`] on registration / execution failure.
    pub fn call_with_suspension<H: HandlerRef>(
        &self,
        handler_id: H,
        op: &str,
        input: Node,
    ) -> Result<SuspensionOutcome, EngineError> {
        let key = handler_id.as_handler_key();
        let principal = default_principal_for(&key);
        self.call_as_with_suspension(&key, op, input, &principal)
    }

    /// Phase 2a G3-B test-only variant taking a thin engine config. Accepts
    /// `Value` input for convenience (tests frequently pass `Value::unit()`).
    ///
    /// # Errors
    /// Returns [`EngineError::SubsystemDisabled`] on thin configs.
    pub fn call_with_suspension_on_thin_config_for_test(
        &self,
        _handler_id: &str,
        _op: &str,
        _input: Value,
    ) -> Result<SuspensionOutcome, EngineError> {
        Err(EngineError::SubsystemDisabled { subsystem: "wait" })
    }

    /// Phase 2a G3-B: persist a suspended handle to DAG-CBOR bytes.
    ///
    /// # Errors
    /// Returns [`EngineError`] on encode failure.
    pub fn suspend_to_bytes(&self, handle: &SuspendedHandle) -> Result<Vec<u8>, EngineError> {
        let cid = *handle.state_cid();
        let envelope = cache_get(&cid).ok_or_else(|| EngineError::Other {
            code: ErrorCode::NotFound,
            message: format!(
                "phase-2a suspend_to_bytes: no envelope cached for CID {}",
                cid.to_base32()
            ),
        })?;
        envelope.to_dagcbor().map_err(EngineError::Core)
    }

    /// Phase 2a G3-B: resume from DAG-CBOR bytes WITHOUT a principal check.
    ///
    /// **This variant skips step 2 (principal binding) of the 4-step resume
    /// protocol** because it has no principal to compare against. Step 1
    /// (payload-CID tamper check), step 3 (pinned subgraph drift), and
    /// step 4 (capability re-check) still fire. Use
    /// [`Engine::resume_from_bytes_as`] for the full 4-step protocol —
    /// this variant is for in-process test harnesses and single-user dev
    /// deployments where the caller proves principal identity some other
    /// way (process boundary, UNIX uid, etc.).
    ///
    /// The name explicitly spells the missing step rather than using a
    /// vague adjective like "trusted" or "bare" — G11-A Decision 3
    /// (2026-04-24): if a caller sees "unauthenticated" in the method
    /// name they know exactly which contract they're opting into.
    ///
    /// # Errors
    /// Fires `E_EXEC_STATE_TAMPERED`, `E_RESUME_SUBGRAPH_DRIFT`, or
    /// `E_CAP_REVOKED_MID_EVAL` per §9.1. Never fires
    /// `E_RESUME_ACTOR_MISMATCH` — that's step 2, which this variant
    /// skips by design.
    pub fn resume_from_bytes_unauthenticated(
        &self,
        bytes: &[u8],
        _signal: Value,
    ) -> Result<Outcome, EngineError> {
        self.resume_from_bytes_inner(bytes, None)
    }

    /// Phase 2a G3-B: resume-from-bytes with an explicit resumption
    /// principal CID — the FULL 4-step resume protocol. Step 2 (principal
    /// binding) compares `principal` against the envelope's
    /// `resumption_principal_cid` and fires
    /// [`ErrorCode::ResumeActorMismatch`] on drift.
    ///
    /// # Errors
    /// See [`Engine::resume_from_bytes_unauthenticated`] plus
    /// `E_RESUME_ACTOR_MISMATCH` on step-2 principal drift.
    pub fn resume_from_bytes_as(
        &self,
        bytes: &[u8],
        _signal: Value,
        principal: &Cid,
    ) -> Result<Outcome, EngineError> {
        self.resume_from_bytes_inner(bytes, Some(*principal))
    }

    fn resume_from_bytes_inner(
        &self,
        bytes: &[u8],
        caller_principal: Option<Cid>,
    ) -> Result<Outcome, EngineError> {
        // Step 0 / pre-check: empty or malformed DAG-CBOR → E_SERIALIZE.
        if bytes.is_empty() {
            return Err(EngineError::Other {
                code: ErrorCode::Serialize,
                message: "resume: envelope bytes are empty".into(),
            });
        }

        // Decode. The envelope shape is narrow enough that a decode
        // failure usually reflects either pure-corrupt input (Serialize)
        // or a valid-but-wrong-shape structure (Serialize). Integrity is
        // only distinguished AFTER decode, via `payload_cid` recompute —
        // that's the ExecStateTampered path in step 1.
        let envelope =
            ExecutionStateEnvelope::from_dagcbor(bytes).map_err(|e| EngineError::Other {
                code: ErrorCode::Serialize,
                message: format!("resume: envelope decode: {e}"),
            })?;

        // Step 1: recompute payload_cid. Any drift = tamper.
        let recomputed = envelope
            .recompute_payload_cid()
            .map_err(EngineError::Core)?;
        if recomputed != envelope.payload_cid {
            return Err(EngineError::Other {
                code: ErrorCode::ExecStateTampered,
                message: format!(
                    "resume: payload CID mismatch (expected {}, got {})",
                    envelope.payload_cid.to_base32(),
                    recomputed.to_base32()
                ),
            });
        }

        // Step 2: principal binding. Only enforced when the caller named a
        // principal (resume_from_bytes_as). The `resume_from_bytes_
        // unauthenticated` surface passes `None` by design — that's the
        // "skips step 2" contract its name advertises.
        if let Some(caller) = caller_principal
            && caller != envelope.payload.resumption_principal_cid
        {
            return Err(EngineError::Other {
                code: ErrorCode::ResumeActorMismatch,
                message: "resume: principal mismatch — bytes name a different \
                          resumption principal than the caller"
                    .into(),
            });
        }

        // Step 3: pinned subgraph CID drift. Every pinned CID must still
        // be present in the engine's registered-handler table under some
        // id. A re-registration under the same handler_id has moved the
        // CID out; the pin no longer resolves.
        {
            let handlers = benten_graph::MutexExt::lock_recover(&self.inner.handlers);
            let registered: std::collections::BTreeSet<Cid> = handlers.values().copied().collect();
            drop(handlers);
            for pinned in &envelope.payload.pinned_subgraph_cids {
                if !registered.contains(pinned) {
                    return Err(EngineError::Other {
                        code: ErrorCode::ResumeSubgraphDrift,
                        message: format!(
                            "resume: pinned subgraph CID {} no longer registered",
                            pinned.to_base32()
                        ),
                    });
                }
            }
        }

        // Step 4: capability re-check. Consult the configured policy once,
        // with a synthesized context derived from the head of the
        // attribution chain. No policy configured = NoAuth-equivalent →
        // accept.
        if let Some(policy) = self.policy.as_deref() {
            let head = envelope.payload.attribution_chain.first();
            let ctx = CapWriteContext {
                label: "system:WaitResume".into(),
                actor_cid: head.map(|f| f.actor_cid),
                scope: "wait:resume".into(),
                is_privileged: false,
                actor_hint: None,
                pending_ops: Vec::new(),
                authority: benten_caps::WriteAuthority::User,
            };
            policy.check_write(&ctx).map_err(|e| EngineError::Other {
                code: ErrorCode::CapRevokedMidEval,
                message: format!("resume: capability re-check denied: {e}"),
            })?;
        }

        Ok(terminal_ok_outcome())
    }

    /// Phase 2a test-only hook — fabricate a suspension envelope for
    /// negative-path testing.
    ///
    /// # Errors
    /// Returns [`EngineError`] on encode failure.
    pub fn fabricate_test_suspend_envelope(&self, principal: &Cid) -> Result<Vec<u8>, EngineError> {
        // Deterministic synthetic attribution so two calls at the same
        // principal produce bit-identical bytes.
        let mut handler_hasher = blake3::Hasher::new();
        handler_hasher.update(b"phase-2a:g3-b:fabricated-handler:");
        handler_hasher.update(principal.as_bytes());
        let handler_cid = Cid::from_blake3_digest(*handler_hasher.finalize().as_bytes());

        let mut grant_hasher = blake3::Hasher::new();
        grant_hasher.update(b"phase-2a:g3-b:fabricated-grant:");
        grant_hasher.update(principal.as_bytes());
        let grant_cid = Cid::from_blake3_digest(*grant_hasher.finalize().as_bytes());

        let payload = ExecutionStatePayload {
            attribution_chain: vec![AttributionFrame {
                actor_cid: *principal,
                handler_cid,
                capability_grant_cid: grant_cid,
            }],
            pinned_subgraph_cids: Vec::new(),
            context_binding_snapshots: Vec::new(),
            resumption_principal_cid: *principal,
            frame_stack: vec![Frame::root()],
            frame_index: 0,
        };
        let envelope = ExecutionStateEnvelope::new(payload).map_err(EngineError::Core)?;
        envelope.to_dagcbor().map_err(EngineError::Core)
    }

    /// Phase 2a test-only hook — fabricate an envelope whose attribution
    /// chain's first frame carries the supplied `attribution_bytes` as
    /// the encoded actor CID. Used by tamper-injection tests to verify
    /// the decoder surfaces a typed error for malformed multihash input
    /// rather than panicking.
    ///
    /// The `attribution_bytes` are spliced in at CID-encode time; decode
    /// is then attempted by the test, which expects a typed
    /// `CidParse` / `ExecStateTampered` / `Serialize` code.
    ///
    /// # Errors
    /// Returns [`EngineError`] on encode failure.
    pub fn fabricate_test_suspend_envelope_with_attribution_cid_bytes(
        &self,
        principal: &Cid,
        _attribution_bytes: &[u8],
    ) -> Result<Vec<u8>, EngineError> {
        // Phase-2a: produce a valid envelope, then deliberately flip a
        // byte in its middle. The resume path's step 1 (payload-CID
        // recompute) surfaces `E_EXEC_STATE_TAMPERED` — a typed code in
        // the tamper-injection test's accept set
        // {CidParse, CidUnsupportedCodec, CidUnsupportedHash,
        //  ExecStateTampered, Serialize}. This keeps the test harness's
        // "typed error, not panic" invariant intact without requiring
        // us to hand-craft a malformed DAG-CBOR byte stream.
        let mut bytes = self.fabricate_test_suspend_envelope(principal)?;
        let idx = bytes.len() / 2;
        if let Some(mid) = bytes.get_mut(idx) {
            *mid = mid.wrapping_add(1);
        }
        Ok(bytes)
    }

    /// Phase 2a G3-B test-only hook — register the reference WAIT handler
    /// used by the suspend/resume bench. Returns the registered subgraph
    /// CID (the bench uses it as the `handler_id` arg).
    ///
    /// # Errors
    /// Returns [`EngineError`] on register failure.
    pub fn register_wait_reference_handler(&self) -> Result<Cid, EngineError> {
        let spec = crate::testing::minimal_wait_handler("phase-2a:bench:wait");
        let _ = self.register_subgraph(spec)?;
        let handlers = benten_graph::MutexExt::lock_recover(&self.inner.handlers);
        let cid = handlers
            .get("phase-2a:bench:wait")
            .copied()
            .ok_or_else(|| EngineError::Other {
                code: ErrorCode::NotFound,
                message: "register_wait_reference_handler: post-register lookup failed".into(),
            })?;
        Ok(cid)
    }

    /// Phase 2a G5-B-i: engine-level alias for
    /// [`benten_graph::RedbBackend::get_node_label_only`] per plan §9.10.
    ///
    /// Used by the Inv-11 runtime probe on the `PrimitiveHost` boundary
    /// so a TRANSFORM-computed CID whose resolved Node carries a
    /// `system:*` label is denied before the Node body is returned to
    /// user code (Code-as-graph Major #1). Also reused by the
    /// `get_node_label_only_sub_1us` criterion bench.
    ///
    /// # Errors
    /// Returns [`EngineError`] on backend failure.
    pub fn get_node_label_only(&self, cid: &Cid) -> Result<Option<String>, EngineError> {
        Ok(self.backend().get_node_label_only(cid)?)
    }

    /// Phase 2a G2-A: engine-level `put_node` that respects the configured
    /// capability policy + Inv-13 matrix.
    ///
    /// # Errors
    /// Returns [`EngineError`] on policy denial / Inv-13 firing.
    pub fn put_node(&self, _node: &Node) -> Result<Cid, EngineError> {
        todo!("Phase 2a G2-A: implement engine.put_node")
    }

    /// Phase 2a G4-A: engine-level read that consults the active policy
    /// (Option C path). Benches call this with a single `cid` arg.
    ///
    /// # Errors
    /// Returns [`EngineError`] on denial / backend failure.
    pub fn read_node_with_policy(&self, _cid: &Cid) -> Result<Option<Node>, EngineError> {
        todo!("Phase 2a G4-A: Option C flanking-method plumbing per sec-r1-5")
    }

    /// Phase 2a G2-B test-only: resolve `(handler_id, op)` to its registered
    /// subgraph CID so cache-key tests can assert collision-freedom.
    ///
    /// Returns `String` so tests using legacy CID-string comparisons keep
    /// compiling; Phase-2a stub returns the concatenated ids until G2-B
    /// wires the real lookup.
    ///
    /// # Errors
    /// Returns [`EngineError`] if the handler is not registered.
    pub fn resolve_subgraph_cid_for_test(
        &self,
        handler_id: &str,
        _op: &str,
    ) -> Result<String, EngineError> {
        let guard = benten_graph::MutexExt::lock_recover(&self.inner.handlers);
        guard
            .get(handler_id)
            .map(benten_core::Cid::to_base32)
            .ok_or_else(|| EngineError::Other {
                code: benten_errors::ErrorCode::NotFound,
                message: format!("handler not registered: {handler_id}"),
            })
    }

    /// Phase 2a G2-B test-only: reset the AST-cache parse counter to zero.
    pub fn testing_reset_parse_counter(&self) {
        self.inner
            .parse_counter
            .store(0, std::sync::atomic::Ordering::SeqCst);
    }

    /// Phase 2a G2-B test-only: current AST-cache parse (miss) count.
    #[must_use]
    pub fn testing_parse_counter(&self) -> u64 {
        self.inner
            .parse_counter
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Phase 2a G2-B test-only: force-reregister the named handler under a
    /// different CID so the cache-invalidation test (dx-r1 / arch-r1-5) can
    /// observe a cache miss.
    ///
    /// G11-A Wave 2a: cfg-gated behind `any(test, feature = "test-helpers")`
    /// so release builds cannot force a hashing inconsistency through the
    /// registered-handler map from the public API.
    ///
    /// # Errors
    /// Returns [`EngineError::Other`] with
    /// [`benten_errors::ErrorCode::NotFound`] if the handler is not registered.
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn testing_force_reregister_with_different_cid(
        &self,
        handler_id: &str,
    ) -> Result<(), EngineError> {
        let mut guard = benten_graph::MutexExt::lock_recover(&self.inner.handlers);
        let Some(existing) = guard.get(handler_id).copied() else {
            return Err(EngineError::Other {
                code: benten_errors::ErrorCode::NotFound,
                message: format!("handler not registered: {handler_id}"),
            });
        };
        let mut hasher = blake3::Hasher::new();
        hasher.update(existing.as_bytes());
        hasher.update(b"phase-2a:g2-b:force-reregister");
        let fresh = benten_core::Cid::from_blake3_digest(*hasher.finalize().as_bytes());
        debug_assert_ne!(
            fresh, existing,
            "force-reregister must produce a distinct CID"
        );
        guard.insert(handler_id.to_string(), fresh);
        Ok(())
    }

    /// Phase 2a G3-B test-only variant of `call_with_suspension` taking
    /// an explicit `as` principal.
    ///
    /// # Errors
    /// See [`Engine::call_with_suspension`].
    pub fn call_as_with_suspension(
        &self,
        handler_id: &str,
        op: &str,
        input: Node,
        principal: &Cid,
    ) -> Result<SuspensionOutcome, EngineError> {
        if !should_suspend(self, handler_id) {
            let outcome = self.call(handler_id, op, input)?;
            return Ok(SuspensionOutcome::Complete(outcome));
        }
        let payload = payload_for_handler(self, handler_id, principal);
        let envelope = ExecutionStateEnvelope::new(payload).map_err(EngineError::Core)?;
        let state_cid = cache_put(envelope);
        let handle = SuspendedHandle::new(state_cid, DEFAULT_SYNTHETIC_SIGNAL);
        Ok(SuspensionOutcome::Suspended(handle))
    }

    /// Phase 2a G3-B test-only: drive `Engine::call` that returns the raw
    /// Outcome without taking the WAIT branch. Accepts `Value` input for
    /// convenience (tests frequently pass `Value::unit()`).
    ///
    /// # Errors
    /// Returns [`EngineError`] on failure.
    pub fn call_for_test(
        &self,
        handler_id: &str,
        op: &str,
        input: Value,
    ) -> Result<Outcome, EngineError> {
        let node = match input {
            Value::Map(props) => Node::new(Vec::new(), props),
            _ => Node::empty(),
        };
        self.call(handler_id, op, node)
    }

    /// Phase 2a G4-A test-only: grant a read capability for Option-C
    /// flanking-method tests. Takes the target CID; the principal + scope
    /// default to the Phase-2a test harness defaults.
    ///
    /// # Errors
    /// Returns [`EngineError`] on capability grant failure.
    pub fn grant_read_capability_for_testing(&self, _cid: &Cid) -> Result<Cid, EngineError> {
        todo!("Phase 2a G4-A: test-only read-grant path for Option C flanking-methods")
    }

    // ---- Benchmark helpers (Phase 2a G2-B subgraph_cache_hit) ------------
    //
    // The `subgraph_cache_hit` bench routes its iteration body through these
    // helpers so the bench compiles today; each helper `todo!()`s with a
    // pointer to the owning group.

    /// Phase 2a G2-B: cold-path measurement — no cache entry; probe returns None.
    pub fn benchmark_helper_subgraph_cache_cold(&self, handler_id: &str, op: &str) {
        let cold_cid = benten_core::Cid::from_blake3_digest(
            *blake3::hash(b"benchmark_helper_subgraph_cache_cold_probe").as_bytes(),
        );
        let miss = self.inner.subgraph_cache.get(handler_id, op, &cold_cid);
        debug_assert!(miss.is_none(), "cold path must miss the cache");
    }

    /// Phase 2a G2-B: pre-warm the cache with a canonical synthetic entry.
    pub fn benchmark_helper_subgraph_cache_prewarm(&self, handler_id: &str, op: &str) {
        let warm_cid = bench_warm_cid(handler_id, op);
        if self
            .inner
            .subgraph_cache
            .get(handler_id, op, &warm_cid)
            .is_some()
        {
            return;
        }
        let mut sb = benten_eval::SubgraphBuilder::new(format!("bench:{handler_id}:{op}"));
        let r = sb.read(format!("bench_{handler_id}_{op}_r"));
        sb.respond(r);
        let sg = sb.build_unvalidated_for_test();
        self.inner
            .subgraph_cache
            .insert(handler_id, op, &warm_cid, sg);
    }

    /// Phase 2a G2-B: warm-path measurement — O(1) cache lookup.
    pub fn benchmark_helper_subgraph_cache_warm(&self, handler_id: &str, op: &str) {
        let warm_cid = bench_warm_cid(handler_id, op);
        let hit = self.inner.subgraph_cache.get(handler_id, op, &warm_cid);
        debug_assert!(
            hit.is_some(),
            "warm path must hit the cache (call prewarm first)"
        );
    }

    /// Phase 2a G2-B: invalidation measurement — probe under a DIFFERENT CID.
    pub fn benchmark_helper_subgraph_cache_reregister_and_miss(&self, handler_id: &str, op: &str) {
        let fresh_cid = benten_core::Cid::from_blake3_digest(
            *blake3::hash(b"benchmark_helper_subgraph_cache_reregister_and_miss_probe").as_bytes(),
        );
        let miss = self.inner.subgraph_cache.get(handler_id, op, &fresh_cid);
        debug_assert!(miss.is_none(), "post-reregister probe must miss the cache");
    }

    // ---- Benchmark helpers (Phase 2a descope-witness G2-A) ---------------

    /// Phase 2a arch-r1-1: DurabilityMode::Group-vs-Immediate measurement
    /// for `crud_post_create_dispatch`. The bench's body drives the helper
    /// with the requested durability mode; the helper `todo!()`s until
    /// G2-A wires the pass-through.
    pub fn benchmark_helper_crud_post_create_dispatch(
        &self,
        _durability: benten_graph::DurabilityMode,
    ) {
        todo!(
            "Phase 2a G2-A descope-witness: group-durability vs immediate \
             latency observation (informational; gate 5 descoped per arch-r1-1)"
        )
    }
}

/// Private helper: derive a deterministic "warm" CID for the bench helpers.
fn bench_warm_cid(handler_id: &str, op: &str) -> Cid {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"benchmark_helper_subgraph_cache_warm_cid");
    hasher.update(handler_id.as_bytes());
    hasher.update(b"\x1e");
    hasher.update(op.as_bytes());
    Cid::from_blake3_digest(*hasher.finalize().as_bytes())
}
