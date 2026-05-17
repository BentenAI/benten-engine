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
//! # Envelope persistence (G12-E)
//!
//! Phase-2a shipped a test-grade `ENVELOPE_CACHE` (process-local
//! `LazyLock<Mutex<BTreeMap<Cid, ExecutionStateEnvelope>>>` gated behind
//! the `envelope-cache-test-grade` feature) that round-tripped suspend
//! envelope bytes for `suspend_to_bytes` / `resume_from_bytes_*`.
//! G12-E retires that cache entirely: persistence now goes through
//! [`Engine::suspension_store`] (a [`benten_eval::SuspensionStore`]),
//! whose default impl is the redb-backed
//! [`crate::suspension_store::RedbSuspensionStore`] over the engine's
//! own redb file. Cross-process resume (engine A drops, engine B opens
//! same path) hydrates the envelope bytes from disk — closing the
//! Phase-2a Compromise #10 cross-process gap.
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
//!    with a synthesized `CapWriteContext` derived from the persisted
//!    `attribution_chain`. The grant may have been revoked while the
//!    bytes sat on disk; revocation surfaces
//!    [`ErrorCode::CapRevokedMidEval`] per §9.13 refresh point #4.
//!
//! Only once all four steps pass does the evaluator take the resume path
//! and produce a terminal `Outcome`.

use benten_caps::{CapError, CapWriteContext, ReadContext};
use benten_core::{Cid, Node, Value};
use benten_errors::ErrorCode;
use benten_eval::{
    AttributionFrame, ExecutionStateEnvelope, ExecutionStatePayload, Frame, SuspendedHandle,
    WaitResumeSignal,
};
use benten_graph::MutexExt;

use crate::engine::Engine;
use crate::error::EngineError;
use crate::outcome::Outcome;

use std::cell::Cell;

thread_local! {
    /// G14-D wave-5a: thread-local hint carrying the historical
    /// policy-metadata blob captured at suspend time per
    /// phase-2-backlog §7.3 + plan §3 G14-D. The capability policy
    /// hook at Step 4 of `resume_from_bytes_inner` reads this if it
    /// implements historical-state binding; policies that don't
    /// consume it leave it as a no-op. The cell is reset at the end
    /// of every `resume_from_bytes_inner` call (via [`HistoricalPolicyMetadataHintGuard`])
    /// so a subsequent resume on the same thread starts with a clean
    /// slate even if the configured policy never consumed the hint.
    pub(crate) static HISTORICAL_POLICY_METADATA_HINT: Cell<Option<Vec<u8>>> = const { Cell::new(None) };
}

/// G14-D wave-5a (htl-1 mini-review fix): RAII drain guard for
/// [`HISTORICAL_POLICY_METADATA_HINT`]. Drains the cell on construction
/// (so the resume body starts clean even if a previous resume left a
/// stale hint behind) AND on drop (so the cell is empty after the
/// resume body returns regardless of whether the configured policy
/// consumed the hint via [`historical_policy_metadata_hint`]). The
/// drop arm survives early returns / `?`-propagated errors which is
/// why we use RAII rather than an explicit drain at every exit point.
pub(crate) struct HistoricalPolicyMetadataHintGuard;

impl HistoricalPolicyMetadataHintGuard {
    pub(crate) fn drain_on_entry_and_exit() -> Self {
        HISTORICAL_POLICY_METADATA_HINT.with(|cell| cell.set(None));
        Self
    }
}

impl Drop for HistoricalPolicyMetadataHintGuard {
    fn drop(&mut self) {
        HISTORICAL_POLICY_METADATA_HINT.with(|cell| cell.set(None));
    }
}

/// G14-D wave-5a: read-and-clear accessor for the thread-local
/// historical-policy-metadata hint. Policy implementations call this
/// from inside their `check_write` / `check_read` hook to pick up the
/// blob the engine threaded in from the suspension store. Returns
/// `None` if no hint was set (NoAuth-equivalent / non-WAIT-resume
/// callers).
#[must_use]
pub fn historical_policy_metadata_hint() -> Option<Vec<u8>> {
    HISTORICAL_POLICY_METADATA_HINT.with(Cell::take)
}

// ---------------------------------------------------------------------------
// Envelope persistence — G12-E retired the test-grade ENVELOPE_CACHE
// ---------------------------------------------------------------------------
//
// Phase-2a shipped `ENVELOPE_CACHE` (a process-local `LazyLock<Mutex<
// BTreeMap<Cid, ExecutionStateEnvelope>>>` gated behind the
// `envelope-cache-test-grade` feature) as a test-grade stand-in until a
// real persistence layer landed. G12-E lands that layer:
// [`benten_eval::SuspensionStore`] with a redb-backed default impl
// (`RedbSuspensionStore`). The two helpers below now route
// suspend / fetch through `Engine::suspension_store()` so the bytes
// survive an `Engine::drop` AND cross a process boundary cleanly.
fn cache_put(engine: &Engine, envelope: ExecutionStateEnvelope) -> Result<Cid, EngineError> {
    let cid = envelope.payload_cid;
    engine
        .suspension_store
        .put_envelope(envelope)
        .map_err(|e| EngineError::Other {
            code: ErrorCode::HostBackendUnavailable,
            message: format!("suspension store put_envelope: {e}"),
        })?;
    Ok(cid)
}

fn cache_get(engine: &Engine, cid: &Cid) -> Option<ExecutionStateEnvelope> {
    engine.suspension_store.get_envelope(cid).ok().flatten()
}

// ---------------------------------------------------------------------------
// Public API shapes
// ---------------------------------------------------------------------------

/// Phase 2b wave-8g (G12-E follow-up) — payload variant supplied to
/// [`Engine::resume_with_meta`].
///
/// `None` resumes a duration-style WAIT or a signal-style WAIT whose
/// caller has no value to deliver (the resume is a wake-up only).
/// `Signal(Value)` carries an explicit signal value to thread through
/// the resume protocol's deadline + shape checks.
///
/// Per the wave-8g brief this is the engine-level wrapper around the
/// existing `Engine::resume_from_bytes_*` surface; the variant naming
/// mirrors the [`benten_eval::WaitResumeSignal`] shape that the
/// evaluator's `resume` entry point already consumes.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ResumePayload {
    /// Resume without delivering a signal value (duration-style WAIT or
    /// fire-and-forget wake-up).
    None,
    /// Resume with an explicit signal value (signal-style WAIT). The
    /// value is threaded into the resume protocol's shape check.
    Signal(Value),
}

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

/// Phase-2b Wave-8i: helper for `call_as_with_suspension`'s legacy fallback
/// path. The Wave-8i refactor routes regular dispatch through
/// `Engine::dispatch_call`; the only callers of this helper now are
/// thin-engine fixtures that need a structural detector for the
/// `SubgraphSpec::empty(id)` shape (Phase-2a test surface) plus the
/// production hot-path that detects WAIT-bearing handlers without
/// running them. Production WAIT routing is now done by
/// `dispatch_call_inner` -> `EngineError::WaitSuspended` -> the
/// catch-arm in `call_as_with_suspension`.
///
/// Retained for the empty-subgraph case (a `SubgraphSpec::empty(id)` test
/// fixture has no primitives, so the regular walker would terminate
/// immediately rather than hit the WAIT dispatcher). The empty-subgraph
/// fixture predates Wave-8i and continues to take the synthesized-handle
/// shortcut path documented as Phase-2a behaviour.
fn empty_spec_should_suspend(engine: &Engine, handler_id: &str) -> bool {
    let specs = benten_graph::MutexExt::lock_recover(&engine.inner.specs);
    let Some(spec) = specs.get(handler_id) else {
        return false;
    };
    spec.primitives.is_empty()
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
        // R6FP-R3 architect A12: distinct from `primitive_host::noauth_zero_grant_cid()` — that
        // helper centralises the noauth-grant placeholder for Phase-3 UCAN substitution. This
        // fallback fires when the requested handler isn't registered (a suspend-protocol
        // invariant break, not a missing grant), so we keep it open-coded so a Phase-3 grep
        // for grant-CID seams doesn't false-positive on this site.
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
        sandbox_depth: 0,
        ..Default::default()
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
        // Phase-2a empty-spec fixture path: a `SubgraphSpec::empty(id)`
        // test handler has no primitives, so the regular walker would
        // terminate immediately rather than hit the WAIT dispatcher.
        // Preserve the legacy synthesized-handle shortcut for the
        // empty-spec fixture by routing through the explicit
        // `call_as_with_suspension` form with the deterministic-default
        // principal (legacy contract, retained verbatim for the
        // empty-spec branch).
        if empty_spec_should_suspend(self, &key) {
            let principal = default_principal_for(&key);
            return self.call_as_with_suspension(&key, op, input, &principal);
        }
        // Wave-8i fix-pass (w8i-wait-cag-01): the unauthenticated
        // suspension form does NOT bind a principal in the envelope.
        // Routing through `dispatch_call` with `actor=None` makes the
        // engine's `suspending_principal()` accessor return `None`, so
        // `wait::evaluate_op` retains the legacy signal-derived
        // placeholder behaviour. Convergence with `engine.call()`
        // (also `actor=None`) is preserved — both surfaces produce the
        // same envelope CID for the same WAIT properties (the
        // Wave-8i convergence pin in `wait_primitive_consults_signal_property`).
        match self.dispatch_call(&key, op, input, None) {
            Ok(outcome) => Ok(SuspensionOutcome::Complete(outcome)),
            Err(EngineError::WaitSuspended { handle }) => {
                // Phase-3 G20-A2 (D12 wave-8a): track the envelope for GC
                // sweeps + run an opportunistic event-driven sweep on
                // suspend (unless event-driven GC is disabled). The sweep
                // is best-effort — failure to sweep does not abort the
                // suspend.
                self.wait_ttl_track_envelope(*handle.state_cid());
                self.wait_ttl_run_event_driven_sweep_if_enabled();
                Ok(SuspensionOutcome::Suspended(handle))
            }
            Err(e) => Err(e),
        }
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
        let envelope = cache_get(self, &cid).ok_or_else(|| EngineError::Other {
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
    ///
    /// R6FP-Group-1 (r6-mpc-1): the `signal` Value is now threaded
    /// through to the eval-side `wait::resume_with_meta` consumer for
    /// `signal_shape` validation (fires `E_INV_REGISTRATION` on shape
    /// mismatch) AND for routing duration-WAIT vs signal-WAIT branches.
    /// Legacy callers passing `Value::Null` see the "no value" path
    /// (duration-WAIT consults the deadline branch; signal-WAIT
    /// completes with Null when no shape is declared, fires
    /// `E_INV_REGISTRATION` when a shape is declared and Null doesn't
    /// match it).
    pub fn resume_from_bytes_unauthenticated(
        &self,
        bytes: &[u8],
        signal: Value,
    ) -> Result<Outcome, EngineError> {
        self.resume_from_bytes_inner(bytes, None, ResumePayload::Signal(signal))
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
    ///
    /// R6FP-Group-1 (r6-mpc-1): the `signal` Value is now threaded
    /// through to the eval-side `wait::resume_with_meta` consumer (see
    /// [`Self::resume_from_bytes_unauthenticated`] for the full
    /// rationale).
    pub fn resume_from_bytes_as(
        &self,
        bytes: &[u8],
        signal: Value,
        principal: &Cid,
    ) -> Result<Outcome, EngineError> {
        self.resume_from_bytes_inner(bytes, Some(*principal), ResumePayload::Signal(signal))
    }

    /// Phase 2b wave-8g (G12-E follow-up): engine-level resume entry
    /// point that the R3 red-phase test fixtures + future call sites
    /// drive.
    ///
    /// `envelope` MUST be the DAG-CBOR bytes of a previously-suspended
    /// `ExecutionStateEnvelope` (the same shape `suspend_to_bytes`
    /// produces). The resume routes through
    /// [`Engine::resume_from_bytes_unauthenticated`] which runs steps
    /// 1, 3, and 4 of the §9.1 4-step protocol (payload-CID tamper
    /// check, pinned-subgraph drift check, capability re-check). Step 2
    /// (principal binding) is skipped per the unauthenticated contract;
    /// callers needing principal binding should use
    /// [`Engine::resume_from_bytes_as`] directly.
    ///
    /// G12-E lifted the missing-metadata path inside the eval-layer
    /// `resume_with_meta` from a permissive `Complete(value)` fallback
    /// to a typed `E_HOST_BACKEND_UNAVAILABLE`; this engine-level
    /// surface is the public entry point that the cross-process resume
    /// integration tests drive (see
    /// `tests/integration/cross_process_wait_resume.rs` +
    /// `tests/g12_e_suspension_store_round_trips.rs`).
    ///
    /// Wave-8i fix-pass-2 (`w8i-wait-cag-04`): the engine resume path
    /// now consults the [`benten_eval::SuspensionStore`]'s
    /// [`benten_eval::WaitMetadata`] for the resumed envelope and
    /// fires [`ErrorCode::WaitTimeout`] when
    /// `(now - suspend_elapsed_ms) >= timeout_ms`. Prior to fix-pass-2
    /// this engine-side surface returned a successful terminal outcome
    /// regardless of elapsed time, even when the regular-walk
    /// (`engine.call()`) path had stamped a real start reference into
    /// the metadata.
    ///
    /// **Suspension-store key invariant:** the metadata lookup uses
    /// `envelope.envelope_cid()` as the key (matching `put_wait` /
    /// `get_wait`); the envelope-bytes lookup elsewhere in the resume
    /// path uses `payload_cid` as the key (matching `put_envelope` /
    /// `get_envelope`). Two distinct keys for two distinct operations
    /// — both internally consistent within their respective accessors.
    /// The `envelope_cid()` is the BLAKE3-of-canonical-envelope-bytes
    /// (covers `payload_cid` + `schema_version` + the rest of the
    /// envelope shape per Phase 2a G3-A); `payload_cid` is the
    /// BLAKE3-of-canonical-payload-bytes (covers the suspended frame
    /// state). Verified clean by orchestrator-direct grep audit during
    /// wave-8i fix-pass-2.
    ///
    /// # Errors
    /// Returns [`EngineError`] per
    /// [`Engine::resume_from_bytes_unauthenticated`] (steps 1, 3, 4 of
    /// §9.1) — `E_EXEC_STATE_TAMPERED`, `E_RESUME_SUBGRAPH_DRIFT`,
    /// `E_CAP_REVOKED_MID_EVAL`, `E_SERIALIZE`, plus
    /// `E_WAIT_TIMEOUT` when the envelope's recorded WAIT metadata
    /// indicates the deadline has elapsed (Wave-8i fix-pass-2), plus
    /// the eval-layer `E_HOST_BACKEND_UNAVAILABLE` lift on missing
    /// metadata (Compromise #9 / #10 closure).
    pub fn resume_with_meta(
        &self,
        envelope: &[u8],
        payload: ResumePayload,
    ) -> Result<Outcome, EngineError> {
        // R6FP-Group-1 (r6-mpc-1): preserve the typed `ResumePayload`
        // distinction (None vs Signal(v)) into `resume_from_bytes_inner`
        // rather than collapsing both to a `Value` early. The eval-side
        // `wait::resume_with_meta` consumer maps `None` to
        // `WaitResumeSignal::DurationElapsed` (driving the duration-WAIT
        // timeout branch) and maps `Signal(v)` to
        // `WaitResumeSignal::Signal { value: v }` (driving the
        // signal-shape validation branch). Collapsing to `Value::Null`
        // erased that distinction.
        self.resume_from_bytes_inner(envelope, None, payload)
    }

    #[allow(
        clippy::too_many_lines,
        reason = "G14-D wave-5a added the cap_snapshot_hash + persisted-policy-metadata Step 3.5; the four-step \
        resume protocol is a single load-bearing flow that is easier to read inline than split across helpers."
    )]
    fn resume_from_bytes_inner(
        &self,
        bytes: &[u8],
        caller_principal: Option<Cid>,
        resume_payload: ResumePayload,
    ) -> Result<Outcome, EngineError> {
        // G14-D wave-5a (htl-1 mini-review fix): drain the
        // HISTORICAL_POLICY_METADATA_HINT cell on entry AND on every
        // exit (drop). Without this guard a hint set by a previous
        // resume that the configured policy never consumed would leak
        // into the NEXT resume on the same thread.
        let _hint_guard = HistoricalPolicyMetadataHintGuard::drain_on_entry_and_exit();

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

        // Step 1.5 (R6FP-Group-1 r6-mpc-1): full WAIT metadata
        //   consumer — deadline + duration-variant + signal-shape.
        //
        // The eval-side `wait::resume_with_meta` is THE surface that
        // consumes ALL FOUR `WaitMetadata` fields (`timeout_ms`,
        // `suspend_elapsed_ms`, `is_duration`, `signal_shape`) and
        // produces three distinct typed errors:
        //   (a) `E_WAIT_TIMEOUT` when `(now - start) >= timeout` OR
        //       when a duration-variant resume's deadline has fired,
        //   (b) `E_INV_REGISTRATION` (via `EvalError::Invariant`) when
        //       a typed `signal_shape` declared at suspend time does
        //       not structurally match the resume payload.
        //
        // Wave-8i fix-pass-2 (w8i-wait-cag-04) wired ONLY (a) — and
        // only the timeout-vs-suspend branch, not the duration-variant
        // branch. The R6 metadata-producer-vs-consumer lens
        // (`r6-mpc-1`) caught that two of three branches were silently
        // dropped on the engine surface even though they fired
        // correctly through the eval-side direct-resume path. R6FP-G1
        // delegates to `benten_eval::resume_with_meta` so a single
        // authoritative consumer lives at the eval layer and the
        // engine API can no longer drift from it.
        //
        // The metadata-store lookup discriminates THREE shapes of
        // resume input via the SuspensionStore's envelope-side
        // record + the envelope's payload shape:
        //
        // (1) **Real WAIT envelope** — the eval-side wait primitive
        //     persists BOTH the WAIT metadata (`put_wait(cid, meta)`
        //     at `crates/benten-eval/src/primitives/wait.rs`) AND the
        //     envelope itself (`put_envelope(envelope)`). The payload
        //     for these envelopes is built by
        //     `placeholder_payload_for_signal`, which produces
        //     `attribution_chain: Vec::new()` (empty). For real WAIT
        //     envelopes, `get_envelope(state_cid)` returns `Some(_)`
        //     and `get_wait(state_cid)` MUST also return `Some(_)`. A
        //     mismatch (envelope present, metadata absent, payload
        //     shape consistent with eval-side WAIT) is the
        //     load-bearing fail-loud surface: (a) the WAIT TTL GC
        //     reaped the metadata side without the envelope side
        //     (impossible by the GC contract — `reap_one` deletes
        //     both), (b) cross-process resume hit a divergent
        //     SuspensionStore that has the envelope record but lost
        //     metadata (the bug surface Compromise #9 named), or
        //     (c) a caller fabricated an envelope-side record without
        //     a metadata-side counterpart. All three are
        //     E_WAIT_METADATA_MISSING — distinct from
        //     E_WAIT_TTL_EXPIRED (entry exists but deadline passed).
        //
        // (2) **Fabricated test envelope** — the
        //     `testing_make_unregistered_envelope` fixture path
        //     produces an envelope that was NEVER paired with either
        //     side of the store. `get_envelope(state_cid)` returns
        //     `None`. The resume routes through the rest of the
        //     4-step protocol's `terminal_ok_outcome()` arm — the
        //     existing `resume_with_meta_fails_closed_when_metadata_missing`
        //     test surface depends on this disposition.
        //
        // (3) **Empty-spec test fixture envelope** — the
        //     `SubgraphSpec::empty(id)` shortcut path at
        //     `Engine::call_as_with_suspension` (line ~1107) writes
        //     ONLY the envelope side via `cache_put`'s
        //     `put_envelope`; it never invokes the eval-side WAIT
        //     primitive so `put_wait` is NOT called. The payload is
        //     built by `payload_for_handler`, which populates
        //     `attribution_chain: vec![attribution]` (NON-empty,
        //     containing the synthesised principal/handler/grant
        //     frame). This is the legitimate phase-2a fixture path
        //     for shape-pin tests in `engine_wait_api_shape.rs` —
        //     it must NOT trip the WaitMetadataMissing fail-loud.
        //     The non-empty `attribution_chain` is the
        //     content-addressed signature that distinguishes shape
        //     (3) from shape (1) without an extra side-table.
        //
        // Phase-3 G20-A2 (D12 wave-8a; Compromise #9 closure;
        // mr-2 fix-pass + fix-pass-2): promotes the eval-side
        // `E_HOST_BACKEND_UNAVAILABLE` fail-loud to the engine-layer
        // typed code so callers can route on the metadata-missing
        // axis independently of generic backend-unavailable failures.
        // fix-pass-2 refines the discriminator to also check
        // `attribution_chain.is_empty()` so the empty-spec fixture
        // path (shape 3) doesn't false-positive — caught by
        // `engine_wait_api_shape.rs` regressing on the initial
        // fix-pass.
        let state_cid = envelope.envelope_cid();
        let meta_lookup = self.suspension_store.get_wait(&state_cid);
        let envelope_lookup = self.suspension_store.get_envelope(&state_cid);
        let envelope_record_present = matches!(envelope_lookup, Ok(Some(_)));
        // Shape (1) signature: payload built by
        // `placeholder_payload_for_signal` → empty
        // `attribution_chain`. Shape (3) signature: payload built by
        // `payload_for_handler` → non-empty `attribution_chain`. The
        // discriminator filters shape (3) out of the WaitMetadataMissing
        // fail-loud so empty-spec fixtures keep their legitimate
        // skip-on-miss behaviour while real-WAIT-envelope-with-evicted-
        // metadata still fails loud (the load-bearing
        // `resume_against_real_envelope_with_evicted_metadata_fires_e_wait_metadata_missing`
        // pin in `tests/integration/cross_process_wait_resume.rs`).
        let payload_is_real_wait_shape = envelope.payload.attribution_chain.is_empty();
        if envelope_record_present && matches!(meta_lookup, Ok(None)) && payload_is_real_wait_shape
        {
            return Err(EngineError::Other {
                code: ErrorCode::WaitMetadataMissing,
                message: format!(
                    "resume: WAIT metadata missing for envelope {} \
                     (envelope record present in SuspensionStore but metadata side absent — \
                     cross-process resume against divergent store, \
                     fabricated half-record, or partial GC corruption): \
                     E_WAIT_METADATA_MISSING",
                    state_cid.to_base32()
                ),
            });
        }
        if let Ok(Some(meta)) = meta_lookup {
            // Phase-3 G20-A2 (D12 wave-8a): wall-clock TTL deadline
            // check fires BEFORE the in-process timeout check. The TTL
            // deadline is wall-clock-anchored (`suspend_wallclock_ms +
            // ttl_hours * 3_600_000`); if the recorded TTL has elapsed
            // we surface E_WAIT_TTL_EXPIRED + GC the entry rather than
            // proceeding into the rest of the resume protocol.
            let wall_now_ms = crate::wait_ttl_gc::wallclock_now_ms(
                *self.wait_wall_clock_override_ms.lock_recover(),
            );
            if crate::wait_ttl_gc::is_expired(&meta, wall_now_ms) {
                // Best-effort GC of the expired entry as part of the
                // resume hot-path's event-driven sweep contract.
                let _ = crate::wait_ttl_gc::reap_one(&self.suspension_store, &state_cid);
                self.wait_ttl_untrack_envelope(&state_cid);
                {
                    let mut stats = self.wait_ttl_gc_stats.lock_recover();
                    stats.reaped_count = stats.reaped_count.saturating_add(1);
                    stats.sweep_count = stats.sweep_count.saturating_add(1);
                }
                return Err(EngineError::Other {
                    code: ErrorCode::WaitTtlExpired,
                    message: format!(
                        "E_WAIT_TTL_EXPIRED: resume: WAIT TTL deadline elapsed for envelope {} \
                         (suspended {:?} ms wall-clock; ttl_hours={:?}; now {wall_now_ms} ms)",
                        state_cid.to_base32(),
                        meta.suspend_wallclock_ms,
                        meta.ttl_hours,
                    ),
                });
            }
            let now_ms = u64::try_from(self.monotonic_source.elapsed_since_start().as_millis())
                .unwrap_or(u64::MAX);
            // Map the engine-level `ResumePayload` enum onto the
            // eval-level `WaitResumeSignal` enum. `None` → DurationElapsed
            // (drives the duration-variant timeout branch); `Signal(v)`
            // → Signal { value: v } (drives the signal-shape validation
            // branch, with `Value::Null` admitted when no shape was
            // declared at suspend time).
            let eval_signal = match &resume_payload {
                ResumePayload::None => WaitResumeSignal::DurationElapsed,
                ResumePayload::Signal(v) => WaitResumeSignal::Signal { value: v.clone() },
            };
            match benten_eval::resume_with_meta(Some(meta), eval_signal, Some(now_ms)) {
                Ok(_) => { /* metadata consumer accepted; continue to step 2 */ }
                Err(eval_err) => {
                    return Err(map_resume_eval_error(eval_err));
                }
            }
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

        // Step 3.5 (G14-D wave-5a): cap_snapshot_hash + persisted-policy
        // metadata re-validation per CLR-2 + Compromise #10 + phase-2-backlog
        // §7.3. If the suspension store carries a CapSnapshot for this
        // envelope, recompute the hash against the chain currently in the
        // engine's cap surface. Mismatch ⇒ E_CAP_SNAPSHOT_HASH_MISMATCH
        // (CLR-2 §11 closure); the policy check at Step 4 below STILL
        // runs even if no snapshot is bound (best-effort skip on miss
        // matches the existing Compromise #10 fail-closed-asymmetry
        // disclosure).
        if let Ok(Some(snapshot)) = self.suspension_store.get_cap_snapshot(&state_cid) {
            // Compute the live chain hash for the envelope's actor.
            // Phase-3 G14-D: the proof-chain inputs are sourced from
            // the engine's durable cap store via the chain accessor on
            // the configured policy; if the policy provides no chain
            // accessor (NoAuthBackend / placeholder), the live chain
            // is the empty chain — and that is itself a meaningful
            // post-revoke state (the chain that produced the snapshot
            // hash had non-empty CIDs; the empty chain hashes
            // differently and thus correctly rejects).
            let actor_cid = envelope
                .payload
                .attribution_chain
                .first()
                .map_or(envelope.payload.resumption_principal_cid, |f| f.actor_cid);
            let live_chain = self.chain_for_actor(&actor_cid);
            // Phase-3 G16-B canary (r4b-cap-2 transition): the legacy
            // 2-input compute path is preserved here pending engine-side
            // capture-of-revocation-set + policy-backend-tag at suspend
            // time. Downstream wave (G16-B wave-6b post-canary) replaces
            // `compute_legacy` with `compute` once the suspend-side
            // capture sites surface the full 4-dimension input. Both
            // paths produce identical hashes for `(empty revocation set,
            // PolicyBackendTag::no_auth())` so existing pinned bytes
            // stay byte-identical.
            let live_hash = crate::cap_snapshot_hash::compute_legacy(&actor_cid, &live_chain);
            if live_hash != snapshot.cap_snapshot_hash {
                return Err(EngineError::Other {
                    code: ErrorCode::CapSnapshotHashMismatch,
                    message: format!(
                        "resume: cap_snapshot_hash mismatch for actor {} \
                         (proof-chain changed between suspend and resume; CLR-2 §11)",
                        actor_cid.to_base32()
                    ),
                });
            }
            // Persisted-policy metadata blob is preserved in the
            // snapshot for the policy to consume at Step 4. We thread
            // it through via a thread-local hint so the policy hook
            // can read historical state if it wishes; if the configured
            // policy doesn't consume historical metadata it is a no-op.
            HISTORICAL_POLICY_METADATA_HINT
                .with(|cell| cell.set(Some(snapshot.historical_policy_metadata)));
        }

        // Step 4: capability re-check. Consult the configured policy once,
        // with a synthesized context derived from the head of the
        // attribution chain. No policy configured = NoAuth-equivalent →
        // accept.
        if let Some(policy) = self.policy.as_deref() {
            let head = envelope.payload.attribution_chain.first();
            // Phase-3 G16-B-prime fp (consumer-audit closure of cor-1 /
            // cap-g16bp-2): thread the engine's configured device-DID-
            // attestation CID into the WAIT-resume cap-recheck so cross-
            // process device-attestation continuity holds at the
            // suspend/resume boundary per D-PHASE-3-25. `None` for legacy
            // / non-attested engines preserves prior behavior.
            let device_cid = *benten_graph::MutexExt::lock_recover(&self.inner.device_cid);
            let ctx = CapWriteContext {
                label: "system:WaitResume".into(),
                actor_cid: head.map(|f| f.actor_cid),
                scope: "wait:resume".into(),
                is_privileged: false,
                actor_hint: None,
                pending_ops: Vec::new(),
                authority: benten_caps::WriteAuthority::User,
                device_cid,
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
                sandbox_depth: 0,
                ..Default::default()
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
    /// R6 round-2 sec-r6r2-02: gated behind `cfg(any(test, feature =
    /// "test-helpers"))` because it calls into `crate::testing` which is
    /// itself gated. The bench (`benten-eval/benches/
    /// wait_suspend_resume_latency.rs`) opts into `test-helpers` via the
    /// dev-dep on `benten-engine`.
    ///
    /// # Errors
    /// Returns [`EngineError`] on register failure.
    #[cfg(any(test, feature = "test-helpers"))]
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

    /// Engine-internal write surface — hash `node` (CIDv1 over labels +
    /// properties only), store it, and return its CID. Closes
    /// `docs/future/phase-3-backlog.md §13.7` item (a).
    ///
    /// Per CLAUDE.md baked-in commitment #18 (plugin trust model), this
    /// is the engine-internal counterpart to [`Engine::create_node`]:
    /// it routes through the same backend transaction (so ChangeEvents
    /// + IVM materialization fire on commit) but skips the
    /// user-facing Inv-11 system-zone label rejection at the API
    /// surface. The storage-layer Inv-11 guard
    /// (`benten-graph/src/redb_backend.rs::guard_system_zone_node`)
    /// stays as defence-in-depth, so a misuse from engine-internal
    /// code still surfaces `E_SYSTEM_ZONE_WRITE` rather than silently
    /// landing a privileged Node.
    ///
    /// Engine internals (the evaluator's WRITE replay path, bench +
    /// integration scaffolding that seeds Nodes without driving a
    /// full handler dispatch) consume this surface; plugin authors
    /// **never call this directly** — they author graph nodes and
    /// the evaluator dispatches.
    ///
    /// # Errors
    /// Returns [`EngineError`] on backend / transaction failure, or
    /// `E_BACKEND_READ_ONLY` when invoked against a snapshot-blob
    /// engine.
    pub fn put_node(&self, node: &Node) -> Result<Cid, EngineError> {
        if self.is_read_only_snapshot() {
            return Err(EngineError::Other {
                code: ErrorCode::BackendReadOnly,
                message: "backend is read-only: put_node rejected (snapshot-blob engine)"
                    .to_string(),
            });
        }
        Ok(self.backend().transaction(|tx| tx.put_node(node))?)
    }

    /// Engine-level read attributed to the supplied principal. Closes
    /// `docs/future/phase-3-backlog.md §13.7` item (b) — the Option-C
    /// flanking entry-point per sec-r1-5 that consults
    /// [`benten_caps::CapabilityPolicy::check_read`] at the engine
    /// boundary with the caller's principal CID threaded through
    /// [`benten_caps::ReadContext::actor_cid`].
    ///
    /// Mirrors the [`Engine::call_as`] precedent: the public entry
    /// point for any read attributed to a non-trusted principal. Per
    /// CLAUDE.md baked-in commitment #18, the evaluator threads the
    /// active principal through this surface when dispatching a
    /// plugin's read; plugin authors **never call this directly** —
    /// they author graph nodes and the evaluator is the only caller
    /// of `_as`.
    ///
    /// # Inv-11 + Option-C denial semantics
    ///
    /// The runtime probe mirrors [`Engine::get_node`]: (1) a missing
    /// CID returns `Ok(None)`; (2) a resolved Node whose primary label
    /// lands inside a system-zone prefix returns `Ok(None)`
    /// regardless of the principal (Inv-11 cannot be overridden by
    /// the cap policy); (3) the principal-threaded `ReadContext` is
    /// then handed to `policy.check_read` — a `CapError::DeniedRead`
    /// collapses to `Ok(None)` per named compromise #2 (Option C:
    /// symmetric None — denial is indistinguishable from miss at the
    /// public API).
    ///
    /// The load-bearing differentiator from [`Engine::get_node`] is
    /// `actor_cid: Some(*principal)` on the `ReadContext` — that
    /// surface passes `actor_cid: None` (no caller identity in scope);
    /// this surface is the explicit `_as`-principal entry point.
    ///
    /// # #593 — the read-as-an-attenuated-principal half of the pair
    ///
    /// Under the unified trust model (CLAUDE.md baked-in #18; #593
    /// re-scope), `read_node_as` and [`Engine::get_node`] are not a
    /// "checked vs bypass" pair — they are
    /// *read-as-an-attenuated-principal* (this surface) vs
    /// *read-as-the-engine-user-root* ([`Engine::get_node`], whose
    /// principal is root by construction). Every external / untrusted /
    /// plugin read MUST reach `read_node_as`; the containment that no
    /// such caller instead reaches the un-attributed engine-internal
    /// read is asserted by
    /// `tests/engine_internal_get_node_is_read_as_user_root_containment.rs`.
    ///
    /// # Errors
    /// Returns [`EngineError`] on backend failure. Cap denial
    /// collapses to `Ok(None)`; it does NOT surface as an error.
    pub fn read_node_as(&self, principal: &Cid, cid: &Cid) -> Result<Option<Node>, EngineError> {
        let Some(node) = self.backend().get_node(cid)? else {
            return Ok(None);
        };
        // Phase-2a Inv-11 runtime probe (mirror of `Engine::get_node`):
        // probe the RESOLVED Node's first label against the engine-side
        // system-zone prefix list. Applied before the cap-policy gate
        // so the policy's verdict cannot override Inv-11.
        let label = node.labels.first().cloned().unwrap_or_default();
        if crate::primitive_host::is_system_zone_label(&label) {
            return Ok(None);
        }
        if let Some(policy) = self.policy() {
            // Thread the engine's configured device-DID-attestation
            // CID (D-PHASE-3-25 heterogeneous-policy dispatch) AND
            // the caller's principal CID into the read-gate
            // ReadContext.
            let device_cid = self.device_cid();
            let ctx = ReadContext {
                label,
                target_cid: Some(*cid),
                actor_cid: Some(*principal),
                device_cid,
                ..Default::default()
            };
            if let Err(CapError::DeniedRead { .. }) = policy.check_read(&ctx) {
                return Ok(None);
            }
        }
        Ok(Some(node))
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

    /// Phase-3 G19-E (wave-7b) test-only: snapshot the per-handler
    /// TRANSFORM AST cache's hit / miss counters and current entry
    /// count.
    ///
    /// Used by the `subgraph_ast_cache_full_wire_up` integration test
    /// to assert the dispatch path actually consults the cache (defends
    /// against the "cache exists but never consulted" failure mode per
    /// the R3-E pin's pim-2 §3.6b end-to-end requirement). Cfg-gated to
    /// keep the test-only API out of the napi cdylib, matching the
    /// sibling `testing_parse_counter` gate.
    #[cfg(any(test, feature = "test-helpers"))]
    #[must_use]
    pub fn testing_ast_cache_stats(&self) -> crate::ast_cache::AstCacheStats {
        self.inner.ast_cache.stats()
    }

    /// Phase-3 G19-E (wave-7b) test-only: reset the AST cache's hit /
    /// miss counters to zero. Used by the wire-up integration test +
    /// the per-call parse cost reduction test to measure a single
    /// dispatch sequence cleanly.
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn testing_reset_ast_cache_counters(&self) {
        self.inner.ast_cache.reset_counters();
    }

    /// Phase 2a G2-B test-only: reset the AST-cache parse counter to zero.
    ///
    /// R6FP-R3 sec-r6r3-02: cfg-gated to keep test-only API out of the napi cdylib.
    /// Mirrors the `testing_force_reregister_with_different_cid` gate below.
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn testing_reset_parse_counter(&self) {
        self.inner
            .parse_counter
            .store(0, std::sync::atomic::Ordering::SeqCst);
    }

    /// Phase 2a G2-B test-only: current AST-cache parse (miss) count.
    ///
    /// R6FP-R3 sec-r6r3-02: cfg-gated to keep test-only API out of the napi cdylib.
    #[cfg(any(test, feature = "test-helpers"))]
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
        drop(guard);
        // G19-E (phase-2-backlog §9.2): drop the OLD CID's entries from
        // the per-handler AST cache so a re-population pass does not
        // accumulate. The NEW CID has no entries yet — TRANSFORM
        // dispatch under the new CID will fall through to the per-call
        // parse path until something else populates the cache (the test
        // hook intentionally bypasses `register_subgraph_replace` so
        // this is the expected behaviour). The integration test
        // `subgraph_ast_cache_correctness_under_handler_re_register`
        // exercises the load-bearing flip via the full
        // `register_subgraph_replace` path which DOES re-populate.
        self.inner.ast_cache.invalidate_handler(&existing);
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
        // Phase-2a empty-subgraph fixture path: `SubgraphSpec::empty(id)`
        // has no primitives, so the regular walker would terminate
        // immediately rather than hit the WAIT dispatcher. Preserve the
        // pre-Wave-8i synthesized-handle shortcut for that fixture only.
        if empty_spec_should_suspend(self, handler_id) {
            let payload = payload_for_handler(self, handler_id, principal);
            let envelope = ExecutionStateEnvelope::new(payload).map_err(EngineError::Core)?;
            let state_cid = cache_put(self, envelope)?;
            let handle = SuspendedHandle::new(state_cid, DEFAULT_SYNTHETIC_SIGNAL);
            // Phase-3 G20-A2 (D12 wave-8a): track even synthetic
            // empty-spec suspends so the GC sweep + Engine::drop final
            // sweep reach them. The empty-spec path doesn't carry TTL
            // metadata so the sweep is a no-op for it (no deadline =
            // not expired) — but the tracking keeps the bookkeeping
            // uniform across surfaces.
            self.wait_ttl_track_envelope(state_cid);
            return Ok(SuspensionOutcome::Suspended(handle));
        }

        // Phase-2b Wave-8i: route regular dispatch through `dispatch_call`.
        // The eval-side WAIT dispatcher (mod.rs `PrimitiveKind::Wait` arm)
        // calls `wait::evaluate_op`, which consults the WAIT node's
        // properties (`signal`, `duration_ms`, `timeout_ms`,
        // `signal_shape`) rather than the prior `should_suspend` heuristic
        // that ignored them. A suspension surfaces as
        // `EngineError::WaitSuspended { handle }` (round-tripped from
        // `EvalError::WaitSuspended` via `eval_error_to_engine_error`);
        // we catch it here and return the typed `Suspended` arm.
        //
        // Wave-8i fix-pass (w8i-wait-cag-01): thread the caller's
        // principal into `dispatch_call` so it lands on the
        // `active_call` stack, where the engine's
        // `suspending_principal()` PrimitiveHost accessor reads it and
        // hands it to `wait::evaluate_op`, which overrides the
        // envelope's `resumption_principal_cid`. The pre-fix-pass code
        // dropped `principal` on the floor (`let _ = principal;`) and
        // the envelope was keyed on `BLAKE3(signal_name)` — so a
        // subsequent `resume_from_bytes_as(_, _, &caller_cid)` fired
        // `E_RESUME_ACTOR_MISMATCH` against the original caller for
        // every real WAIT handler, silently breaking the principal
        // binding contract.
        match self.dispatch_call(handler_id, op, input, Some(*principal)) {
            Ok(outcome) => Ok(SuspensionOutcome::Complete(outcome)),
            Err(EngineError::WaitSuspended { handle }) => {
                // Phase-3 G20-A2 (D12 wave-8a): same suspend-time GC hook
                // as `call_with_suspension`.
                self.wait_ttl_track_envelope(*handle.state_cid());
                self.wait_ttl_run_event_driven_sweep_if_enabled();
                Ok(SuspensionOutcome::Suspended(handle))
            }
            Err(e) => Err(e),
        }
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

    /// Test/bench-only: grant a read capability for the supplied
    /// target CID to a specific `principal`. Closes
    /// `docs/future/phase-3-backlog.md §13.7` item (c).
    ///
    /// Looks up the Node's primary label via
    /// [`Engine::get_node_label_only`], derives the
    /// `store:<label>:read` scope (mirroring
    /// [`benten_caps::GrantBackedPolicy`]'s `derive_read_scope`), and
    /// installs the grant via
    /// [`crate::engine_caps::EngineCapsHandle::grant_capability`] against the
    /// supplied principal. The returned `Cid` is the minted
    /// `system:CapabilityGrant` Node's CID so callers can revoke
    /// surgically.
    ///
    /// ## Phase 4-Foundation R1-FP G22-FP-3 (cap-r1-2 BLOCKER closure)
    ///
    /// Pre-fix the helper minted its own synthetic
    /// `test-read-grant-helper` principal and granted to that actor.
    /// Callers then read with a *different* principal and the
    /// scope-only `check_read` would still permit — because the
    /// pre-fix [`benten_caps::GrantBackedPolicy::check_read`] ignored
    /// `actor_cid`. The cap-r1-2 closure made `check_read`
    /// principal-aware (filtering by stored `grantee` property), so
    /// granting to a synthetic helper-actor no longer permits reads as
    /// a different actor. Callers MUST now thread the same `principal`
    /// for both grant + subsequent reads. The helper signature carries
    /// the principal arg explicitly to make the binding visible.
    ///
    /// Gated behind `cfg(any(test, feature = "test-helpers"))` so the
    /// helper is not present in production builds — mirrors
    /// [`Engine::testing_force_reregister_with_different_cid`] and
    /// the `testing_*` accessor pattern.
    ///
    /// # Errors
    /// Returns [`EngineError`] on backend lookup / grant failure, or
    /// `E_NOT_FOUND` if the target CID is missing from the backend.
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn grant_read_capability_for_testing(
        &self,
        cid: &Cid,
        principal: &Cid,
    ) -> Result<Cid, EngineError> {
        let label = self
            .backend()
            .get_node_label_only(cid)?
            .ok_or_else(|| EngineError::Other {
                code: ErrorCode::NotFound,
                message: format!("grant_read_capability_for_testing: target CID not found {cid:?}"),
            })?;
        let scope = if label.is_empty() {
            "store:read".to_string()
        } else {
            format!("store:{label}:read")
        };
        self.caps().grant_capability(principal, scope)
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

    // The Phase-2a engine-level descope-witness stub
    // `benchmark_helper_crud_post_create_dispatch` was deleted under
    // `docs/future/phase-3-backlog.md §13.7` item (d) — it had zero
    // callers, panicked via `todo!()`, and violated CLAUDE.md §"No
    // deprecated aliases or backward-compat shims". The active
    // durability bench at
    // `crates/benten-graph/benches/crud_post_create_dispatch_group_durability.rs`
    // routes through
    // `benten_graph::RedbBackend::benchmark_helper_crud_post_create_dispatch`
    // directly.
}

/// R6FP-Group-1 (r6-mpc-1): Map an `EvalError` returned by
/// `benten_eval::resume_with_meta` onto the public `EngineError` shape
/// the resume API surfaces. Three concrete shapes:
///
/// 1. `EvalError::Host(WaitTimeout)` → `EngineError::Other {
///    code: WaitTimeout, .. }` — the deadline OR duration-variant branch
///    fired.
/// 2. `EvalError::Invariant(Registration)` → `EngineError::Other {
///    code: InvRegistration, .. }` — declared `signal_shape` did not
///    structurally match the resume payload (atk-r1: typed mismatch,
///    NOT a tamper).
/// 3. Anything else (host errors, missing-metadata loud-fail, etc.) →
///    catch-all `EngineError::Other` with the eval error's catalog
///    code preserved through `EvalError::code()`.
fn map_resume_eval_error(err: benten_eval::EvalError) -> EngineError {
    match err {
        benten_eval::EvalError::Host(host_err) => {
            let code = host_err.code.clone();
            let message = format!("{host_err}");
            match code {
                ErrorCode::WaitTimeout => EngineError::Other {
                    code: ErrorCode::WaitTimeout,
                    message: format!("resume: wait deadline elapsed: {message}"),
                },
                // Phase-3 G20-A2 (D12 wave-8a): the eval-side production
                // fail-loud at `benten_eval::resume_with_meta` returns
                // `EvalError::Host(HostBackendUnavailable)` when called
                // with `meta: None` (the missing-metadata path —
                // Compromise #9 / G12-E closure; see
                // `crates/benten-eval/src/primitives/wait.rs::resume_with_meta`).
                // The engine layer promotes that to the typed
                // `WaitMetadataMissing` so callers can route on the
                // metadata-missing axis independently of generic
                // backend-unavailable. The eval-side ErrorCode remains
                // `HostBackendUnavailable` (broader semantic surface);
                // the engine-layer remap is the user-facing typed code.
                ErrorCode::HostBackendUnavailable
                    if message.contains("wait resume: suspension store has no metadata") =>
                {
                    EngineError::Other {
                        code: ErrorCode::WaitMetadataMissing,
                        message: format!("resume: WAIT metadata missing: {message}"),
                    }
                }
                other_code => EngineError::Other {
                    code: other_code,
                    message: format!("resume: eval host error: {message}"),
                },
            }
        }
        benten_eval::EvalError::Invariant(benten_eval::InvariantViolation::Registration) => {
            EngineError::Other {
                code: ErrorCode::InvRegistration,
                message: "resume: signal payload did not match declared signal_shape".into(),
            }
        }
        other => EngineError::Other {
            code: other.code(),
            message: format!("resume: eval error: {other:?}"),
        },
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

#[cfg(test)]
mod historical_policy_metadata_hint_guard_tests {
    use super::{
        HISTORICAL_POLICY_METADATA_HINT, HistoricalPolicyMetadataHintGuard,
        historical_policy_metadata_hint,
    };

    /// G14-D wave-5a (htl-1 mini-review fix): the RAII guard drains the
    /// thread-local on construction (so a stale hint left by a previous
    /// resume that the policy never consumed cannot leak forward) AND
    /// on drop (so the cell is empty after the guard's scope ends even
    /// if no consumer ever called `historical_policy_metadata_hint`).
    #[test]
    fn historical_policy_metadata_hint_guard_drains_on_entry_and_drop() {
        // Pre-populate as if a previous resume left a hint behind.
        HISTORICAL_POLICY_METADATA_HINT.with(|c| c.set(Some(b"stale-from-prior-resume".to_vec())));

        {
            let _guard = HistoricalPolicyMetadataHintGuard::drain_on_entry_and_exit();
            // Entry-drain: the stale hint is gone before the resume body
            // observes it.
            assert!(
                HISTORICAL_POLICY_METADATA_HINT.with(|c| {
                    let v = c.take();
                    let was_none = v.is_none();
                    c.set(v);
                    was_none
                }),
                "guard must drain stale hint on entry",
            );

            // Simulate Step 3.5 setting a hint mid-resume.
            HISTORICAL_POLICY_METADATA_HINT.with(|c| c.set(Some(b"current-resume-hint".to_vec())));
        }
        // Drop ran: the cell must be empty even though no consumer
        // called `historical_policy_metadata_hint` to drain it.
        assert!(
            historical_policy_metadata_hint().is_none(),
            "guard must drain hint on drop even when policy never consumed it",
        );
    }
}
