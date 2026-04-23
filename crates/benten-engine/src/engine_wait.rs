//! Phase 2a G3-B / N3: WAIT public API — `call_with_suspension`,
//! `suspend_to_bytes`, `resume_from_bytes`, `resume_from_bytes_as`.
//!
//! Sibling module to `engine.rs` following the Phase-1 5d-K pattern. Stubs
//! return `todo!()`-bodies so tests fail at runtime with clear pointers to
//! the owning group.

use benten_core::{Cid, Node, Value};
use benten_eval::SuspendedHandle;

use crate::engine::Engine;
use crate::error::EngineError;
use crate::outcome::Outcome;

/// Phase-2a G3-B return shape for `call_with_suspension`. A handler may
/// complete inline or suspend awaiting an external signal.
#[derive(Debug, Clone)]
pub enum SuspensionOutcome {
    /// The handler ran to completion.
    Complete(Outcome),
    /// The handler suspended; the handle identifies the persisted envelope.
    Suspended(SuspendedHandle),
}

/// Alias kept for back-compat with earlier test spec drafts
/// (`SuspendedOrComplete` ↔ `SuspensionOutcome`).
pub type SuspendedOrComplete = SuspensionOutcome;

/// Phase 2a R3 consolidation: accept both `&str` and `&Cid` as handler
/// identifiers. Bench fixtures that round-trip a just-registered subgraph
/// CID through `call_with_suspension` don't have to `.to_string()` it.
pub trait HandlerRef {}
impl HandlerRef for &str {}
impl HandlerRef for &String {}
impl HandlerRef for String {}
impl HandlerRef for &Cid {}
impl HandlerRef for Cid {}

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
        _handler_id: H,
        _op: &str,
        _input: Node,
    ) -> Result<SuspensionOutcome, EngineError> {
        todo!(
            "Phase 2a G3-B: implement call_with_suspension per plan §9.1 + \
             engine_wait_api_shape tests"
        )
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
        todo!(
            "Phase 2a G3-B: thin-config suspension path surfaces \
             E_SUBSYSTEM_DISABLED"
        )
    }

    /// Phase 2a G3-B: persist a suspended handle to DAG-CBOR bytes.
    ///
    /// # Errors
    /// Returns [`EngineError`] on encode failure.
    pub fn suspend_to_bytes(&self, _handle: &SuspendedHandle) -> Result<Vec<u8>, EngineError> {
        todo!("Phase 2a G3-B: implement suspend_to_bytes per plan §9.1")
    }

    /// Phase 2a G3-B: resume from DAG-CBOR bytes. Runs the 4-step resume
    /// protocol (§9.1). Supply the signal value that fulfils the WAIT.
    ///
    /// # Errors
    /// Fires `E_EXEC_STATE_TAMPERED`, `E_RESUME_ACTOR_MISMATCH`,
    /// `E_RESUME_SUBGRAPH_DRIFT`, or `E_CAP_REVOKED_MID_EVAL` per §9.1.
    pub fn resume_from_bytes(&self, _bytes: &[u8], _signal: Value) -> Result<Outcome, EngineError> {
        todo!("Phase 2a G3-B: implement 4-step resume per plan §9.1")
    }

    /// Phase 2a G3-B: resume-from-bytes with an explicit resumption
    /// principal CID (used by `resume_decode_failure_not_panic.rs`).
    ///
    /// # Errors
    /// See [`Engine::resume_from_bytes`].
    pub fn resume_from_bytes_as(
        &self,
        _bytes: &[u8],
        _signal: Value,
        _principal: &Cid,
    ) -> Result<Outcome, EngineError> {
        todo!(
            "Phase 2a G3-B: implement resume_from_bytes_as per plan §9.1 + \
             sec-r1-1 atk-1 mitigation"
        )
    }

    /// Phase 2a test-only hook — fabricate a suspension envelope for
    /// negative-path testing.
    ///
    /// # Errors
    /// Returns [`EngineError`] on encode failure.
    pub fn fabricate_test_suspend_envelope(
        &self,
        _principal: &Cid,
    ) -> Result<Vec<u8>, EngineError> {
        todo!("Phase 2a G3-B test-only: fabricate synthetic envelope for resume tests")
    }

    /// Phase 2a test-only hook — fabricate an envelope whose attribution CID
    /// bytes are supplied raw (for tamper-injection tests).
    ///
    /// # Errors
    /// Returns [`EngineError`] on encode failure.
    pub fn fabricate_test_suspend_envelope_with_attribution_cid_bytes(
        &self,
        _principal: &Cid,
        _attribution_bytes: &[u8],
    ) -> Result<Vec<u8>, EngineError> {
        todo!("Phase 2a G3-B test-only: tampered-attribution envelope fabrication")
    }

    /// Phase 2a G3-B test-only hook — register the reference WAIT handler
    /// used by the suspend/resume bench. Returns the registered subgraph
    /// CID (the bench uses it as the `handler_id` arg).
    ///
    /// # Errors
    /// Returns [`EngineError`] on register failure.
    pub fn register_wait_reference_handler(&self) -> Result<Cid, EngineError> {
        todo!("Phase 2a G3-B: register canonical WAIT bench handler")
    }

    /// Phase 2a G5-B-i: engine-level alias for
    /// [`benten_graph::RedbBackend::get_node_label_only`] per plan §9.10.
    ///
    /// # Errors
    /// Returns [`EngineError`] on backend failure.
    pub fn get_node_label_only(&self, _cid: &Cid) -> Result<Option<String>, EngineError> {
        todo!("Phase 2a G5-B-i: engine alias for get_node_label_only fast path")
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

    /// Back-compat alias for callers that used `test_hook_parse_counter`.
    #[must_use]
    pub fn test_hook_parse_counter(&self) -> u64 {
        self.testing_parse_counter()
    }

    /// Phase 2a G2-B test-only: force-reregister the named handler under a
    /// different CID so the cache-invalidation test (dx-r1 / arch-r1-5) can
    /// observe a cache miss.
    ///
    /// # Errors
    /// Returns [`EngineError::Other`] with
    /// [`benten_errors::ErrorCode::NotFound`] if the handler is not registered.
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
        _handler_id: &str,
        _op: &str,
        _input: Node,
        _principal: &Cid,
    ) -> Result<SuspensionOutcome, EngineError> {
        todo!(
            "Phase 2a G3-B: principal-scoped call_with_suspension variant for \
             resume_with_substituted_principal_rejects"
        )
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
    pub fn benchmark_helper_subgraph_cache_reregister_and_miss(
        &self,
        handler_id: &str,
        op: &str,
    ) {
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
