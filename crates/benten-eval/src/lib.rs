//! # benten-eval — Operation primitives + evaluator
//!
//! All 12 operation primitive *types* are registered (so stored subgraphs
//! never require re-registration) and all 12 execute in the iterative
//! evaluator — READ, WRITE, TRANSFORM, BRANCH, ITERATE, WAIT, CALL,
//! RESPOND, EMIT, SANDBOX, SUBSCRIBE, STREAM are production-runtime LIVE
//! (SANDBOX/STREAM/SUBSCRIBE/WAIT executors shipped at Phase 2b; the full
//! surface is carried through Phase 4-Foundation).

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::todo, reason = "R3 red-phase stubs; R5 removes todos")]
#![allow(
    clippy::result_large_err,
    reason = "RegistrationError carries per-invariant diagnostic context (paths, expected/actual CIDs, counts) per R1 triage; Phase-2 will box large diagnostic payloads once the accessor set stabilises"
)]
#![allow(
    clippy::too_many_lines,
    reason = "Invariant-validation pass is intentionally linear so the code reads top-to-bottom as the invariant list"
)]

pub use benten_core::{
    ATTRIBUTION_PROPERTY_KEY, NodeHandle, OperationNode, PrimitiveKind, Subgraph, SubgraphBuilder,
};
use benten_core::{Cid, Value};
pub use benten_errors::ErrorCode;
use std::collections::BTreeMap;

pub mod chunk_sink;
pub mod context;
pub mod diag;
pub mod evaluator;
pub mod exec_state;
pub mod expr;
pub mod host;
pub mod host_error;
pub mod invariants;
pub mod primitives;
// Phase 2b G7-A SANDBOX subsystem. Compile-time wasm32-disabled per
// sec-pre-r1-05 — the wasm32 build cuts SANDBOX entirely.
#[cfg(not(target_arch = "wasm32"))]
pub mod sandbox;
pub mod subgraph_ext;
pub mod suspension_store;
// Phase-3 G21-T1 — typed-CALL engine-side dispatch surface (10 ops:
// Ed25519 sign/verify, BLAKE3 hash, multibase, DID resolve, UCAN
// chain validation, VC verify). Per CLAUDE.md baked-in commitment
// #16 (SANDBOX-vs-CALL framing): crypto ops fit CALL, not SANDBOX
// host-fn surface. The 12-primitive commitment (#1) holds — typed-
// CALL is dispatched THROUGH the existing CALL primitive when its
// `target` starts with `engine:typed:`.
#[cfg(any(test, feature = "testing"))]
pub mod testing;
pub mod typed_call;
// Phase-3 G17-B SANDBOX `.wat`/`.wasm` fixture loader (phase-3-backlog
// §6.2 + r1-wsa-5). Native-only (wasm32 cuts SANDBOX entirely per
// sec-pre-r1-05) + reachable from integration-test binaries
// (`tests/sandbox_*.rs`) + downstream consumers that opt into the
// `testing` feature. See `src/test_fixtures.rs` module-level docstring
// for the loader contract (committed `.wasm`-prefer + `.wat`-fallback).
#[cfg(all(not(target_arch = "wasm32"), any(test, feature = "testing")))]
pub mod test_fixtures;
pub mod time_source;

pub use subgraph_ext::{NodeHandleExt, SubgraphBuilderExt, SubgraphExt};
pub use typed_call::{TYPED_CALL_PREFIX, TypedCallOp};

pub use context::EvalContext;
pub use evaluator::{RunOptions, RunResult};
pub use exec_state::{AttributionFrame, ExecutionStateEnvelope, ExecutionStatePayload, Frame};
pub use host::{NullHost, PrimitiveHost, ViewQuery};
pub use host_error::HostError;
pub use primitives::wait::{
    SignalShape, SuspendedHandle, WaitOutcome, WaitResumeSignal, resume_with_meta,
};
pub use suspension_store::{
    InMemorySuspensionStore, SuspensionKey, SuspensionStore, SuspensionStoreError, WaitMetadata,
    default_process_store,
};
#[cfg(any(test, feature = "testing"))]
pub use time_source::MockMonotonicSource;
pub use time_source::{
    HlcTimeSource, InstantMonotonicSource, MockTimeSource, MonotonicSource, TimeSource,
    default_monotonic_source, default_time_source,
};

/// Phase 2a G4-A test harness: register a callee handler with a declared
/// iteration-budget bound. Consumed by `invariant_8_isolated_call` tests
/// so the Inv-8 multiplicative walker can look up the callee's bound at
/// registration time.
///
/// The registry is a process-global `RwLock<HashMap<String, u64>>`; the
/// table is append-only within a test process (a subsequent register of
/// the same name overwrites the prior entry). Real handler registration
/// is an engine-layer concern; this is purely a test-surface convenience
/// (Phase 2a testing-helpers contract — see plan §3 G4-A).
///
/// # Soundness gate (G4-A mini-review C1 + follow-up tightening)
/// This MUTATION surface is gated behind `cfg(any(test, feature =
/// "testing"))` so ANY non-test build (including dev-profile
/// `cargo build` / `cargo check`, release, bench, and custom deploy
/// profiles) cannot pre-seed the registry that `invariants::budget`
/// consults at registration-time validation. The READ path
/// (`lookup_test_callee`) stays unconditional — in a non-test build
/// the registry is always empty, and the Inv-8 validator rejects
/// unknown callees with `E_INV_REGISTRATION` (see G4-A mini-review M1
/// fix).
///
/// Integration tests that live in `crates/benten-eval/tests/*.rs` are
/// covered by the `cfg(test)` leg because cargo compiles the crate
/// with `--cfg test` when building them. Cross-crate test binaries
/// (e.g. a `benten-engine` integration test that wants to seed the
/// benten-eval registry) must explicitly opt into the `testing`
/// feature via `benten-eval = { path = "...", features = ["testing"]
/// }` under their `[dev-dependencies]`. Earlier iterations included a
/// `debug_assertions` leg for DX convenience; that leg was dropped
/// because `cargo build` (dev-profile) sets `debug_assertions = true`
/// and a compromised dep or accidentally-introduced production-path
/// call to `register_test_callee` would compile in that profile.
#[cfg(any(test, feature = "testing"))]
pub fn register_test_callee(name: &str, bound: u64) {
    let mut guard = TEST_CALLEE_REGISTRY
        .write()
        .expect("test callee registry poisoned");
    guard.insert(name.to_string(), bound);
}

/// Look up a previously-registered callee bound. Returns `None` when the
/// name has not been registered — the multiplicative walker treats a
/// CALL with a `handler` property naming an unregistered callee as an
/// Inv-8-rejectable registration error (Phase 2a G4-A M1 fix), so the
/// fallback is no longer "contribute factor 1."
#[must_use]
pub fn lookup_test_callee(name: &str) -> Option<u64> {
    TEST_CALLEE_REGISTRY
        .read()
        .expect("test callee registry poisoned")
        .get(name)
        .copied()
}

static TEST_CALLEE_REGISTRY: std::sync::LazyLock<
    std::sync::RwLock<std::collections::HashMap<String, u64>>,
> = std::sync::LazyLock::new(|| std::sync::RwLock::new(std::collections::HashMap::new()));

/// Phase 2a G3-B: crate-root alias for
/// [`primitives::wait::evaluate`]. Tests call `benten_eval::evaluate(...)`
/// through this re-export.
///
/// # Errors
/// See [`primitives::wait::evaluate`].
pub fn evaluate(sg: &Subgraph, ctx: &mut EvalContext, input: benten_core::Value) -> Outcome {
    match primitives::wait::evaluate(sg, ctx, input) {
        Ok(WaitOutcome::Complete(v)) => Outcome::Complete(v),
        Ok(WaitOutcome::Suspended(h)) => Outcome::Suspended(h),
        Err(e) => Outcome::Err(e.code()),
    }
}

/// Phase 2a G3-B: crate-root alias for [`primitives::wait::resume`].
///
/// v1-API-stabilization (refinement-audit #878): the `handle` arg now
/// accepts a raw [`SuspendedHandle`] directly (the payload of
/// `Outcome::Suspended` post-collapse), so test harnesses pipe
/// `evaluate(...)` → `Outcome::Suspended(h)` → `resume(.., h, ..)` with no
/// re-wrap. The old `handle: WaitOutcome` shape required a runtime
/// `WaitOutcome::Complete(_)` rejection guard because the type permitted a
/// state the API forbids; the collapsed shape makes that state
/// unrepresentable, so the guard (and its `InvRegistration` error path)
/// is deleted as dead code rather than preserved.
///
/// # Errors
/// See [`primitives::wait::resume`].
pub fn resume(
    _sg: &Subgraph,
    ctx: &mut EvalContext,
    handle: SuspendedHandle,
    signal: WaitResumeSignal,
) -> Outcome {
    let state_cid = *handle.state_cid();
    // Phase-2b G12-E: prefer the EvalContext's configured store
    // (engine-wired durable backend) over the process-default
    // singleton. The fallback path retains the prior behaviour for
    // unit harnesses that build an EvalContext without a store.
    let meta = ctx.suspension_store().get_wait(&state_cid).ok().flatten();
    match primitives::wait::resume_with_meta(meta, signal, ctx.elapsed_ms()) {
        Ok(WaitOutcome::Complete(v)) => Outcome::Complete(v),
        Ok(WaitOutcome::Suspended(h)) => Outcome::Suspended(h),
        Err(e) => Outcome::Err(e.code()),
    }
}

/// Phase 2a G3-A: `Outcome` shape mirrored from `benten-engine` so tests
/// can name `benten_eval::Outcome::{Complete, Suspended, Err}` alongside
/// `SuspendedHandle`. Phase-1 owns the real type in `benten-engine`; the
/// re-export is a narrow proxy whose variants match the expected surface.
///
/// `Suspended` carries a raw [`SuspendedHandle`] (v1-API-stabilization,
/// refinement-audit #878): the prior shape was
/// `Outcome::Suspended(WaitOutcome::Suspended(handle))` — a redundant
/// double-nest where the inner `WaitOutcome` could *only ever* be its
/// `Suspended` arm (the `Complete` arm is already routed to
/// `Outcome::Complete`). Collapsing to `Suspended(SuspendedHandle)` makes
/// the unrepresentable state unrepresentable and removes the dead
/// `WaitOutcome::Complete(_)` rejection guard that the old `resume` shape
/// needed.
///
/// TODO(phase-4-meta — backlog §4.43 v1-API-stabilization, eval/engine
/// Outcome unification): the WAIT surface shipped end-to-end at
/// phase-2b-close (3d0f018), but the eval-side and engine-side `Outcome`s
/// remain distinct shapes. Carried from Phase-2a G3-B for consolidation
/// alongside the broader host-boundary cleanup. (Retargeted from the
/// pre-2026-05-11-phase-rename `phase-3` marker per §4.68 in-source
/// trajectory sweep, refinement-audit #1166.)
#[derive(Debug, Clone)]
pub enum Outcome {
    /// Handler ran to completion.
    Complete(benten_core::Value),
    /// Handler suspended at a WAIT primitive. Carries the
    /// [`SuspendedHandle`] needed to resume.
    Suspended(SuspendedHandle),
    /// Terminal error.
    Err(ErrorCode),
}

/// Configurable invariant limits. Defaults match ENGINE-SPEC §4.
pub mod limits {
    /// Invariant 2: default max operation-subgraph depth.
    pub const DEFAULT_MAX_DEPTH: usize = 64;
    /// Invariant 3: default max fan-out per node.
    pub const DEFAULT_MAX_FANOUT: usize = 16;
    /// Invariant 5: default max total nodes per subgraph.
    pub const DEFAULT_MAX_NODES: usize = 4096;
    /// Invariant 6: default max total edges per subgraph.
    pub const DEFAULT_MAX_EDGES: usize = 8192;
}

/// Evaluator error type.
///
/// `#[non_exhaustive]` (R6b bp-17) — STREAM / WAIT / SUBSCRIBE / SANDBOX
/// runtime errors landed at Phase 2b; downstream matchers must include
/// `_ =>` so adding variants stays a minor version bump.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum EvalError {
    /// Registration- or runtime-time invariant violation. Carries the
    /// per-invariant [`InvariantViolation`] discriminant so callers can
    /// route on the specific invariant that fired.
    #[error("invariant violation: {0:?}")]
    Invariant(InvariantViolation),

    /// Capability-system rejection (denial / revocation / attenuation
    /// failure). Wrapped from `benten_caps::CapError` via `#[from]`.
    #[error("capability: {0}")]
    Capability(#[from] benten_caps::CapError),

    /// Host-boundary failure surfaced through a [`PrimitiveHost`] call.
    /// Replaces the Phase-1 `Graph(GraphError)` variant as part of arch-1
    /// dep-break (phil-r1-2 / plan §9.10 + §9.14) — `benten-eval` no longer
    /// depends on `benten-graph`, so storage-layer rejections route through
    /// the opaque [`HostError`] envelope. The wrapped `HostError` carries a
    /// stable catalog code + optional context on the wire plus an opaque
    /// `Box<dyn StdError>` source that never reaches the wire (sec-r1-6 /
    /// atk-6).
    // HostError's Display already includes the "host error (...)" prefix, so
    // a redundant "host: " in this attribute would render as "host: host
    // error (...)". Delegate the whole Display to HostError (G1-B mini-review
    // nit N1).
    #[error("{0}")]
    Host(HostError),

    /// Pass-through of `benten_core::CoreError` (CID parse / dag-cbor
    /// (de)serialise / canonical-bytes mismatch). Wrapped via `#[from]`.
    #[error("core: {0}")]
    Core(#[from] benten_core::CoreError),

    /// The evaluator was asked to dispatch a primitive whose Phase-1
    /// executor stub raises `todo!()` (e.g. STREAM in Phase 1; SANDBOX
    /// pre-Phase-2b). Carries the offending [`PrimitiveKind`].
    #[error("primitive not implemented for Phase 1: {0:?}")]
    PrimitiveNotImplemented(PrimitiveKind),

    /// Registration rejected because two or more invariants failed
    /// simultaneously (Inv-12 aggregate roll-up). Carries the list of
    /// violated invariant numbers (e.g. `vec![1, 5]`).
    #[error("registration rejected — multiple invariants failed")]
    RegistrationCatchAll {
        /// Sorted list of violated invariant numbers (1-based, matches
        /// `docs/INVARIANT-COVERAGE.md` row numbering).
        violated_invariants: Vec<u8>,
    },

    /// Two writers raced on the same backend slot; the loser sees this
    /// error and may retry. Maps to `E_WRITE_CONFLICT`.
    #[error("write conflict")]
    WriteConflict,

    /// TRANSFORM expression parser rejected the source. Message carries
    /// the parser diagnostic (offset + reason).
    #[error("transform grammar rejected: {0}")]
    TransformSyntax(String),

    /// Iterative evaluator's explicit operand stack exceeded its bound
    /// (Phase-1 stopgap for Inv-2 — depth ceiling). Iterative execution
    /// makes this a typed error rather than a process-level overflow.
    #[error("stack overflow in iterative evaluator")]
    StackOverflow,

    /// Backend / host-side error surfaced through a [`PrimitiveHost`] call.
    /// Used by primitive executors (READ, WRITE, CALL, EMIT, ITERATE) when
    /// the host implementation rejects or fails. The engine's `impl
    /// PrimitiveHost` populates this with a debug rendering of its own
    /// `EngineError`.
    #[error("backend: {0}")]
    Backend(String),

    /// An operation on the `PrimitiveHost` boundary is not yet supported by
    /// any Phase-1 replay path (r6b-ce-2). Distinct from
    /// `PrimitiveNotImplemented` (which names a structural primitive that
    /// the evaluator cannot execute) — `Unsupported` names a host-boundary
    /// method whose backing replay is not yet wired. Maps to
    /// `E_NOT_IMPLEMENTED` at the catalog layer so TS callers get the same
    /// stable code used elsewhere for deferred surfaces.
    #[error("unsupported host operation: {operation}")]
    Unsupported {
        /// Name of the unsupported operation, e.g. `"put_edge"`.
        operation: String,
    },

    /// Typed pass-through of an engine-side "unknown view" rejection
    /// (r6b-err-1). Carried so the origin catalog code
    /// (`ErrorCode::UnknownView`) survives the `PrimitiveHost::read_view`
    /// boundary; previously this collapsed into an opaque
    /// `EvalError::Backend(String)` with a debug-formatted payload.
    #[error("unknown view: {0}")]
    UnknownView(String),

    /// Typed pass-through of an engine-side "IVM view stale" rejection.
    /// Carried so `ErrorCode::IvmViewStale` survives the
    /// `PrimitiveHost::read_view` boundary (r6b-err-1).
    #[error("IVM view stale: {0}")]
    IvmViewStale(String),

    /// Typed pass-through of an engine-side "subsystem disabled"
    /// rejection — the thin-engine honest-no (`without_ivm`,
    /// `without_caps`). Carried so `ErrorCode::SubsystemDisabled` survives
    /// the host-boundary (r6b-err-1).
    #[error("subsystem disabled: {0}")]
    SubsystemDisabled(String),

    /// SANDBOX runtime/registration failure — preserves the typed
    /// [`sandbox::SandboxError`] across the `EvalError` boundary so
    /// the stable `E_SANDBOX_*` catalog code survives the
    /// eval → engine → napi → TS pipeline. Wave-8d-types replaces the
    /// prior wave-8b temporary `EvalError::Backend(format!("..."))`
    /// shape used by `impl PrimitiveHost for Engine::execute_sandbox`.
    ///
    /// `cfg(not(target_arch = "wasm32"))`-gated because
    /// [`sandbox::SandboxError`] is itself wasm32-cut per
    /// sec-pre-r1-05 (wasmtime doesn't compile to wasm32). The wasm32
    /// path uses [`EvalError::SubsystemDisabled`] instead via the
    /// `execute_sandbox_wasm32_unavailable` stub on the engine side.
    #[cfg(not(target_arch = "wasm32"))]
    #[error("sandbox: {0}")]
    Sandbox(#[from] sandbox::SandboxError),

    /// Phase-2b Wave-8i: WAIT primitive in a regular `engine.call()` walk
    /// drove the evaluator to a suspension boundary. Carries the
    /// [`SuspendedHandle`] the dispatcher produced via
    /// [`primitives::wait::evaluate_op`]. This is a control-flow signal,
    /// NOT a runtime failure: the engine catches it in
    /// `dispatch_call_inner` and converts to `EngineError::WaitSuspended`
    /// so callers can either route through `Engine::call_with_suspension`
    /// (which surfaces the same boundary as `SuspensionOutcome::Suspended`)
    /// or persist the handle bytes via `Engine::suspend_to_bytes`.
    ///
    /// Replaces the Phase-2a `Err(PrimitiveNotImplemented(Wait))` shape
    /// at the dispatcher (mod.rs:111), which forced callers to know about
    /// `call_with_suspension` to use WAIT at all.
    #[error("wait suspended (envelope cid={state_cid}, signal={signal:?})", state_cid = handle.state_cid().to_base32(), signal = handle.signal_name())]
    WaitSuspended {
        /// Handle to the persisted suspension envelope; CID + signal name.
        handle: SuspendedHandle,
    },

    /// Phase-3 G21-T1: a typed-CALL dispatch named an op not in the
    /// engine's typed-CALL registry. Maps to
    /// `ErrorCode::TypedCallUnknownOp`.
    #[error("typed-CALL: unknown op '{op_name}'")]
    TypedCallUnknownOp {
        /// The op name that was not recognised.
        op_name: String,
    },

    /// Phase-3 G21-T1: a typed-CALL dispatch supplied an input shape
    /// that does not match the named op's expected schema. Maps to
    /// `ErrorCode::TypedCallInvalidInput`.
    #[error("typed-CALL '{op_name}' input rejected: {reason}")]
    TypedCallInvalidInput {
        /// The op name whose input failed validation.
        op_name: &'static str,
        /// Brief diagnostic reason (which field, what was wrong).
        reason: String,
    },

    /// Phase-3 G21-T1: a typed-CALL dispatch was rejected because
    /// the dispatching grant's capability set does not include the
    /// per-op required capability. Maps to
    /// `ErrorCode::TypedCallCapDenied`.
    #[error("typed-CALL '{op_name}' denied: required capability '{required}' not held")]
    TypedCallCapDenied {
        /// The op name whose cap-check failed.
        op_name: &'static str,
        /// The required capability string.
        required: String,
    },

    /// Phase-3 G21-T1: a typed-CALL op's underlying implementation
    /// returned a typed error that bubbles out of the dispatch
    /// boundary (e.g. `KeypairError` / `UcanError` / `VcError` from
    /// `benten-id`; CID parse failure from `benten-core`). Maps to
    /// `ErrorCode::TypedCallDispatchError`.
    #[error("typed-CALL '{op_name}' dispatch failed: {reason}")]
    TypedCallDispatchError {
        /// The op name whose dispatch failed.
        op_name: &'static str,
        /// Brief diagnostic reason from the underlying op.
        reason: String,
    },

    /// Phase-3 R6-FP Wave-C1 (cap-r6-r1-1 / r4b-cap-6 closure): a
    /// SUBSCRIBE / on_change subscription was terminated mid-stream
    /// because the per-event delivery-time cap-recheck closure
    /// returned `false` (CLR-2 §11 dual-layer recheck). Distinct from
    /// `EvalError::Capability(CapError::Revoked)` (in-flight cap
    /// rejection at primitive evaluation) — this variant names the
    /// stream-termination boundary observable to the consumer side.
    /// Maps to `ErrorCode::SubscribeRevokedMidStream`. Carries the
    /// dispatching actor + the change-event anchor so JS/TS consumers
    /// distinguish 'cap-revoke auto-cancel' from buffer-overflow / GC
    /// / cursor-skip / engine-shutdown drops.
    #[error("subscribe: revoked mid-stream (actor_cid={actor_cid:?} anchor_cid={anchor_cid:?})")]
    SubscribeRevokedMidStream {
        /// Hex-rendered actor CID whose grant was revoked. `None`
        /// when the recheck closure does not surface the actor (the
        /// closure shape is `Fn(&ChangeEvent) -> bool`; closures that
        /// internally consult an actor surface return the actor in
        /// the diagnostic carrier — wired through the
        /// `engine_subscribe.rs` adapter).
        actor_cid: Option<String>,
        /// Hex-rendered change-event anchor CID. Identifies the row
        /// whose delivery triggered the recheck-fail.
        anchor_cid: Option<String>,
    },
}

impl EvalError {
    /// Map this `EvalError` onto the stable [`ErrorCode`] catalog
    /// identifier — preserves the catalog code across the
    /// `eval -> engine -> napi -> TS` boundary so TS callers receive the
    /// same `E_*` discriminant the evaluator raised.
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        match self {
            EvalError::Invariant(v) => v.code(),
            EvalError::Capability(c) => c.code(),
            EvalError::PrimitiveNotImplemented(_) => ErrorCode::PrimitiveNotImplemented,
            EvalError::RegistrationCatchAll { .. } => ErrorCode::InvRegistration,
            EvalError::WriteConflict => ErrorCode::WriteConflict,
            EvalError::TransformSyntax(_) => ErrorCode::TransformSyntax,
            EvalError::StackOverflow => ErrorCode::InvDepthExceeded,
            // Preserve the stable catalog code across the cross-crate
            // boundary. Prior to r6-err-1 these collapsed into `Unknown("")`,
            // which made `EvalError → EngineError → napi → TS` lose the
            // origin error code. Dispatch to inner `.code()` so the catalog
            // identifier survives the round-trip. arch-1 dep-break (G1-B):
            // the former `EvalError::Graph(GraphError)` arm is now
            // `EvalError::Host(HostError)`; HostError's `code` field is the
            // catalog discriminant.
            EvalError::Host(h) => h.code.clone(),
            EvalError::Core(e) => e.code(),
            // r6b-err-3: both `EvalError::Backend` and the engine-side
            // `eval_error_to_engine_error` now spell the stable string the
            // same way — the prior `E_BACKEND` / `E_EVAL_BACKEND` split
            // gave one conceptual state two catalog identifiers.
            EvalError::Backend(_) => ErrorCode::Unknown(String::from("E_EVAL_BACKEND")),
            EvalError::Unsupported { .. } => ErrorCode::NotImplemented,
            EvalError::UnknownView(_) => ErrorCode::UnknownView,
            EvalError::IvmViewStale(_) => ErrorCode::IvmViewStale,
            EvalError::SubsystemDisabled(_) => ErrorCode::SubsystemDisabled,
            #[cfg(not(target_arch = "wasm32"))]
            EvalError::Sandbox(s) => s.code(),
            EvalError::WaitSuspended { .. } => ErrorCode::WaitSuspended,
            EvalError::TypedCallUnknownOp { .. } => ErrorCode::TypedCallUnknownOp,
            EvalError::TypedCallInvalidInput { .. } => ErrorCode::TypedCallInvalidInput,
            EvalError::TypedCallCapDenied { .. } => ErrorCode::TypedCallCapDenied,
            EvalError::TypedCallDispatchError { .. } => ErrorCode::TypedCallDispatchError,
            EvalError::SubscribeRevokedMidStream { .. } => ErrorCode::SubscribeRevokedMidStream,
        }
    }
}

/// Structural-invariant violation variants.
///
/// `#[non_exhaustive]` (R6b bp-17) — invariants 4 (SANDBOX nest depth), 7
/// (SANDBOX output limit), 11 (system-zone reachability), 13 (immutability),
/// 14 (causal attribution) land in Phase 2 and each introduces a variant
/// here; downstream matchers must include `_ =>` so adding variants is a
/// minor version bump.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum InvariantViolation {
    /// Invariant 1: graph contains a cycle.
    Cycle,
    /// Invariant 2: longest-path depth exceeds the configured maximum.
    DepthExceeded,
    /// Invariant 3: per-node fan-out exceeds the configured maximum.
    FanoutExceeded,
    /// Invariant 5: total node count exceeds the configured maximum.
    TooManyNodes,
    /// Invariant 6: total edge count exceeds the configured maximum.
    TooManyEdges,
    /// Invariant 9: handler declared deterministic contains a
    /// non-deterministic primitive (e.g. RANDOM, time read at the
    /// declaration tier).
    Determinism,
    /// Invariant 10: canonical-bytes encoding is not order-independent
    /// (DAG-CBOR canonicalisation drift).
    ContentHash,
    /// Invariant 9 (DSL surface): an ITERATE primitive omitted the
    /// required `max` declaration.
    IterateMaxMissing,
    /// Runtime + registration-time cumulative iteration-budget violation
    /// (invariant 8). Phase-2a folds what was Phase-1's nest-depth stopgap
    /// (`IterateNestDepth`, now stripped) into a single multiplicative-
    /// through-CALL check via `benten-eval::invariants::budget`. Maps to
    /// [`ErrorCode::InvIterateBudget`] / `E_INV_ITERATE_BUDGET`.
    IterateBudget,
    /// Aggregate catch-all for Invariant 12 — fires when two or more
    /// invariants are violated simultaneously. See
    /// `crates/benten-eval/tests/invariants_9_10_12.rs::registration_catch_all_populates_violated_list`.
    Registration,
    /// Invariant 14 (G5-B-ii): a primitive-type in the subgraph did not
    /// declare whether it consumes an `AttributionFrame`. Fires at
    /// registration-time. Maps to `ErrorCode::InvAttribution`.
    Attribution,
    /// Invariant 13 (G5-A): a WRITE primitive declares a literal CID target
    /// that is already registered as an immutable subgraph/Node. Fires at
    /// registration-time (declaration-layer reject). Maps to
    /// [`ErrorCode::InvImmutability`]. Runtime firing lives in
    /// `benten-graph` per plan §9.11.
    Immutability,
    /// Invariant 11 (G5-B-i): a user subgraph declares a READ or WRITE
    /// whose target label falls within a `system:*` system-zone prefix.
    /// Fires at registration-time via the literal-CID walker in
    /// [`crate::invariants::system_zone::validate_registration`]; the
    /// runtime counterpart lives in `benten-engine/src/primitive_host.rs`
    /// and reuses the `ErrorCode::InvSystemZone` code. Maps to
    /// [`ErrorCode::InvSystemZone`].
    SystemZone,
    /// Invariant 4 (G7-B): SANDBOX nest-depth ceiling exceeded. Fires at
    /// registration-time via static SubgraphSpec analysis (counts SANDBOX
    /// nodes along the call-graph) AND at runtime via the
    /// `AttributionFrame.sandbox_depth: u8` counter when a TRANSFORM-
    /// computed SANDBOX target pushes past the configured ceiling
    /// (default 4). D20-RESOLVED — counter INHERITED across CALL boundaries.
    /// Maps to [`ErrorCode::InvSandboxDepth`].
    SandboxDepth,
    /// Invariant 7 (G7-B): SANDBOX cumulative output exceeded
    /// `output_max_bytes`. Fires at the streaming `CountedSink` PRIMARY
    /// path BEFORE bytes are accepted (D17-RESOLVED defense-in-depth) and
    /// also at the primitive return-value backstop. D15 trap-loudly default —
    /// no silent truncation in Phase 2b. Maps to
    /// [`ErrorCode::InvSandboxOutput`].
    SandboxOutput,
}

impl InvariantViolation {
    /// Map this invariant violation onto its stable [`ErrorCode`]
    /// catalog identifier.
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        match self {
            InvariantViolation::Cycle => ErrorCode::InvCycle,
            InvariantViolation::DepthExceeded => ErrorCode::InvDepthExceeded,
            InvariantViolation::FanoutExceeded => ErrorCode::InvFanoutExceeded,
            InvariantViolation::TooManyNodes => ErrorCode::InvTooManyNodes,
            InvariantViolation::TooManyEdges => ErrorCode::InvTooManyEdges,
            InvariantViolation::Determinism => ErrorCode::InvDeterminism,
            InvariantViolation::ContentHash => ErrorCode::InvContentHash,
            InvariantViolation::IterateMaxMissing => ErrorCode::InvIterateMaxMissing,
            InvariantViolation::IterateBudget => ErrorCode::InvIterateBudget,
            InvariantViolation::Registration => ErrorCode::InvRegistration,
            InvariantViolation::Attribution => ErrorCode::InvAttribution,
            InvariantViolation::Immutability => ErrorCode::InvImmutability,
            InvariantViolation::SystemZone => ErrorCode::InvSystemZone,
            InvariantViolation::SandboxDepth => ErrorCode::InvSandboxDepth,
            InvariantViolation::SandboxOutput => ErrorCode::InvSandboxOutput,
        }
    }
}

/// Registration-time error surface. Carries per-invariant context so the
/// DX layer can render "your handler has N nodes, max is M".
#[derive(Debug, Clone)]
pub struct RegistrationError {
    pub(crate) kind: InvariantViolation,
    pub(crate) depth_actual: Option<usize>,
    pub(crate) depth_max: Option<usize>,
    pub(crate) longest_path: Option<Vec<String>>,
    pub(crate) cycle_path: Option<Vec<String>>,
    pub(crate) fanout_actual: Option<usize>,
    pub(crate) fanout_max: Option<usize>,
    pub(crate) fanout_node_id: Option<String>,
    pub(crate) nodes_actual: Option<usize>,
    pub(crate) nodes_max: Option<usize>,
    pub(crate) edges_actual: Option<usize>,
    pub(crate) edges_max: Option<usize>,
    pub(crate) violated_invariants: Option<Vec<u8>>,
    pub(crate) expected_cid: Option<Cid>,
    pub(crate) actual_cid: Option<Cid>,
}

impl RegistrationError {
    /// Construct a new `RegistrationError` from an invariant violation
    /// kind. All per-invariant context fields default to `None`;
    /// builders attach context as needed.
    #[must_use]
    pub fn new(kind: InvariantViolation) -> Self {
        Self {
            kind,
            depth_actual: None,
            depth_max: None,
            longest_path: None,
            cycle_path: None,
            fanout_actual: None,
            fanout_max: None,
            fanout_node_id: None,
            nodes_actual: None,
            nodes_max: None,
            edges_actual: None,
            edges_max: None,
            violated_invariants: None,
            expected_cid: None,
            actual_cid: None,
        }
    }

    /// Stable [`ErrorCode`] catalog identifier for this error
    /// (delegates to the wrapped [`InvariantViolation::code`]).
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        self.kind.code()
    }

    /// Borrowed view of the underlying invariant kind.
    #[must_use]
    pub fn kind(&self) -> &InvariantViolation {
        &self.kind
    }

    /// Observed depth that triggered the violation (`Some` when
    /// `kind == DepthExceeded`).
    #[must_use]
    pub fn depth_actual(&self) -> Option<usize> {
        self.depth_actual
    }

    /// Observed fan-out that triggered the violation (`Some` when
    /// `kind == FanoutExceeded`).
    #[must_use]
    pub fn fanout_actual(&self) -> Option<usize> {
        self.fanout_actual
    }

    /// Sorted list of violated invariant numbers when `kind ==
    /// Registration` (Inv-12 aggregate roll-up).
    #[must_use]
    pub fn violated_invariants(&self) -> Option<&Vec<u8>> {
        self.violated_invariants.as_ref()
    }

    /// Reconstructed cycle path for Invariant-1 failures (node-id sequence).
    #[must_use]
    pub fn cycle_path(&self) -> Option<Vec<String>> {
        self.cycle_path.clone()
    }

    /// Configured max depth when `InvDepthExceeded` fires.
    #[must_use]
    pub fn depth_max(&self) -> Option<usize> {
        self.depth_max
    }

    /// Longest path in the subgraph (diagnostic for `InvDepthExceeded`).
    #[must_use]
    pub fn longest_path(&self) -> Option<Vec<String>> {
        self.longest_path.clone()
    }

    /// Declared-by-caller CID for `InvContentHash` failures.
    #[must_use]
    pub fn expected_cid(&self) -> Option<Cid> {
        self.expected_cid
    }

    /// Computed-from-bytes CID for `InvContentHash` failures.
    #[must_use]
    pub fn actual_cid(&self) -> Option<Cid> {
        self.actual_cid
    }

    /// Configured max nodes (Invariant 5).
    #[must_use]
    pub fn nodes_max(&self) -> Option<usize> {
        self.nodes_max
    }

    /// Actual node count (Invariant 5).
    #[must_use]
    pub fn nodes_actual(&self) -> Option<usize> {
        self.nodes_actual
    }

    /// Configured max edges (Invariant 6).
    #[must_use]
    pub fn edges_max(&self) -> Option<usize> {
        self.edges_max
    }

    /// Actual edge count (Invariant 6).
    #[must_use]
    pub fn edges_actual(&self) -> Option<usize> {
        self.edges_actual
    }

    /// Configured max fan-out (Invariant 3).
    #[must_use]
    pub fn fanout_max(&self) -> Option<usize> {
        self.fanout_max
    }

    /// Node id whose fan-out exceeded the cap (Invariant 3 diagnostic).
    #[must_use]
    pub fn fanout_node_id(&self) -> Option<String> {
        self.fanout_node_id.clone()
    }
}

/// `Display` impl for `RegistrationError` — required so consumers (notably
/// `EngineError::Invariant(#[from] Box<RegistrationError>)`) participate in
/// the standard `std::error::Error::source()` chain via thiserror's
/// `{0}`-format expansion. Phase-2a R6FP catch-up EH4. The rendering is
/// deliberately compact: catalog code as the leading discriminant followed
/// by the first available diagnostic context field. Operators wanting the
/// full diagnostic structure use the typed accessors (`depth_actual()`,
/// `cycle_path()`, etc.) — `Display` is the one-line summary.
impl core::fmt::Display for RegistrationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.kind.code().as_static_str())?;
        if let (Some(actual), Some(max)) = (self.nodes_actual, self.nodes_max) {
            write!(f, " (nodes: {actual}/{max})")?;
        } else if let (Some(actual), Some(max)) = (self.edges_actual, self.edges_max) {
            write!(f, " (edges: {actual}/{max})")?;
        } else if let (Some(actual), Some(max)) = (self.depth_actual, self.depth_max) {
            write!(f, " (depth: {actual}/{max})")?;
        } else if let (Some(actual), Some(max)) = (self.fanout_actual, self.fanout_max) {
            write!(f, " (fanout: {actual}/{max}")?;
            if let Some(ref id) = self.fanout_node_id {
                write!(f, " at node {id}")?;
            }
            write!(f, ")")?;
        } else if let (Some(expected), Some(actual)) = (self.expected_cid, self.actual_cid) {
            write!(f, " (expected CID {expected}, actual {actual})")?;
        } else if let Some(ref violated) = self.violated_invariants {
            // R6 round-2 C2-R2-5: render the invariant numbers as a
            // Display-style comma-separated list rather than the
            // `{:?}` Debug-formatted `Vec<u8>`. Keeps the rest of the
            // impl's compact one-line summary style consistent.
            write!(f, " (violated invariants: ")?;
            for (i, n) in violated.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{n}")?;
            }
            write!(f, ")")?;
        }
        Ok(())
    }
}

/// `Error` impl for `RegistrationError` — paired with `Display` above so the
/// type satisfies `std::error::Error` and can be threaded through `#[from]`
/// / `#[source]` in downstream `thiserror` enums. R6FP catch-up EH4.
impl std::error::Error for RegistrationError {}

/// Borrowed snapshot of a [`SubgraphBuilder`] used by the invariant checker.
/// Kept separate so `invariants` never needs a mutable handle on the
/// builder. After Phase-2b G12-C-cont this snapshot is built by
/// [`crate::subgraph_ext`] from the builder's validator-accessor surface
/// (the builder itself moved to `benten-core`).
pub(crate) struct SubgraphSnapshot<'a> {
    pub(crate) nodes: &'a [OperationNode],
    pub(crate) parallel_fanout: &'a [usize],
    pub(crate) iterate_depth: &'a [usize],
    pub(crate) edges: &'a [(NodeHandle, NodeHandle, String)],
    pub(crate) extra_edges: usize,
    pub(crate) deterministic: bool,
    #[allow(dead_code, reason = "kept for future diagnostic surfaces")]
    pub(crate) handler_id: &'a str,
}

/// Configurable invariant thresholds.
#[derive(Debug, Clone)]
pub struct InvariantConfig {
    /// Invariant 2: maximum operation-subgraph depth.
    pub max_depth: u32,
    /// Invariant 3: maximum per-node fan-out.
    pub max_fanout: u32,
    /// Invariant 5: maximum total node count per subgraph.
    pub max_nodes: u32,
    /// Invariant 6: maximum total edge count per subgraph.
    pub max_edges: u32,
    /// Phase-2b G7-B / Inv-4: maximum SANDBOX nesting depth. The
    /// `AttributionFrame.sandbox_depth: u8` counter is checked against
    /// this ceiling at every SANDBOX entry. Default `4` per D20-RESOLVED
    /// (enough for legitimate composition; 5+ smells like accidental
    /// recursion). The hard type-level ceiling is `u8::MAX` — even with
    /// `max_sandbox_nest_depth = u8::MAX`, the
    /// `checked_add(1).ok_or(SandboxNestedDispatchDepthExceeded)` pattern
    /// in `invariants::sandbox_depth` saturates without wraparound.
    pub max_sandbox_nest_depth: u8,
    /// Phase-2b G7-B / Inv-7: maximum per-call SANDBOX cumulative-output
    /// ceiling in bytes. Registration rejects any SANDBOX node that
    /// declares `output_max_bytes` greater than this value; runtime
    /// `CountedSink` enforces the per-node value (or the engine default
    /// when omitted) against this same hard ceiling. Default is
    /// [`invariants::sandbox_output::DEFAULT_MAX_SANDBOX_OUTPUT_BYTES`]
    /// (16 MiB) per D15 trap-loudly framing.
    pub max_sandbox_output_bytes: u64,
}

impl Default for InvariantConfig {
    fn default() -> Self {
        Self {
            max_depth: u32::try_from(limits::DEFAULT_MAX_DEPTH).unwrap_or(64),
            max_fanout: u32::try_from(limits::DEFAULT_MAX_FANOUT).unwrap_or(16),
            max_nodes: u32::try_from(limits::DEFAULT_MAX_NODES).unwrap_or(4096),
            max_edges: u32::try_from(limits::DEFAULT_MAX_EDGES).unwrap_or(8192),
            max_sandbox_nest_depth: invariants::sandbox_depth::DEFAULT_MAX_SANDBOX_NEST_DEPTH,
            max_sandbox_output_bytes: invariants::sandbox_output::DEFAULT_MAX_SANDBOX_OUTPUT_BYTES,
        }
    }
}

/// A single execution frame on the iterative evaluator's stack.
#[derive(Debug, Clone)]
pub struct ExecutionFrame {
    /// Identifier of the operation Node this frame is executing.
    pub node_id: String,
    /// Position of this frame in the stack (0 = bottom).
    pub frame_index: usize,
}

/// The iterative evaluator (stack-model, no recursion).
///
/// **Phase 1 G6 stub.**
pub struct Evaluator {
    /// Live execution-frame stack. Bounded by [`Self::max_stack_depth`].
    pub stack: Vec<ExecutionFrame>,
    /// Stopgap for Inv-2 — process-level overflow becomes a typed
    /// [`EvalError::StackOverflow`] when the stack would exceed this
    /// depth.
    pub max_stack_depth: u32,
}

impl Evaluator {
    /// Construct a fresh evaluator with default `max_stack_depth = 64`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            max_stack_depth: 64,
        }
    }

    /// Evaluate a primitive operation and return a trace step.
    ///
    /// **G6-A dispatch shim.** This Phase-1 body routes to
    /// [`primitives::dispatch`] so the per-primitive executors (READ, WRITE,
    /// RESPOND, EMIT in G6-A; TRANSFORM, BRANCH, ITERATE, CALL in G6-B) can
    /// be exercised from the test suite without the full stack-model
    /// evaluator. G6-C replaces this body with the real iterative walker
    /// that enforces invariants 2 / 8, owns frame push/pop semantics, and
    /// follows typed error edges across the subgraph.
    ///
    /// # Errors
    ///
    /// Propagates whatever the per-primitive executor returns, plus
    /// [`EvalError::StackOverflow`] when the current stack has reached
    /// [`Evaluator::max_stack_depth`] so G6-C's overflow contract holds
    /// even under the shim.
    pub fn step(
        &mut self,
        op: &OperationNode,
        host: &dyn PrimitiveHost,
    ) -> Result<StepResult, EvalError> {
        if u32::try_from(self.stack.len()).unwrap_or(u32::MAX) >= self.max_stack_depth {
            return Err(EvalError::StackOverflow);
        }
        let result = primitives::dispatch(op, host)?;
        // G6-C owns the full stack discipline; the shim records a frame on
        // successful dispatch and drops one on a terminal RESPOND so the
        // evaluator_stack tests see a non-zero frame delta.
        if result.edge_label == "terminal" {
            self.stack.pop();
        } else {
            self.stack.push(ExecutionFrame {
                node_id: op.id.clone(),
                frame_index: self.stack.len(),
            });
        }
        Ok(result)
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a single primitive execution.
#[derive(Debug, Clone)]
pub struct StepResult {
    /// Identifier of the next operation Node to execute (or `None` if
    /// the step terminated the handler).
    pub next: Option<String>,
    /// Edge label this step traversed (e.g. `"NEXT"`, `"CASE:foo"`,
    /// `"ON_ERROR"`).
    pub edge_label: String,
    /// Output value the executor returned for this step.
    pub output: Value,
}

/// A trace step returned by `engine.trace(handler, input)`.
///
/// Phase 2a dx-r1 / §9.12: the Phase-1 single-variant shape is promoted to
/// an enum so the boundary/budget variants coexist with the per-primitive
/// `Step` rows.
///
/// TODO(phase-4-meta — backlog §4.75; TraceStep boundary-variant + attribution-threading
/// completion): boundary variants exist on the enum and attribution
/// threads onto Step rows; full `SuspendBoundary` / `ResumeBoundary` /
/// `BudgetExhausted` firing + uniform `attribution` threading on
/// EVERY trace row carries from Phase-2a G3-A/G4-A/G5-B (didn't
/// land); pairs with the broader Phase-3 trace-discipline pass.
#[derive(Debug, Clone)]
#[allow(
    clippy::large_enum_variant,
    reason = "Phase-3 G16-B canary: AttributionFrame grew with peer_did_set/device_did/sync_hop_depth fields \
              (Inv-14 device-grain at sync boundary). Boxing `Option<AttributionFrame>` here would force \
              an allocation on every Step row; the size delta is acceptable until profile data shows it."
)]
pub enum TraceStep {
    /// A single primitive execution row (Phase 1 baseline shape preserved
    /// as struct-variant).
    Step {
        /// Operation-node id within the handler.
        node_id: String,
        /// Duration in microseconds.
        duration_us: u64,
        /// Inputs to the primitive.
        inputs: Value,
        /// Outputs produced by the primitive.
        outputs: Value,
        /// Optional error code if the step failed.
        error: Option<ErrorCode>,
        /// Inv-14 attribution (G5-B-ii wires this). Phase-2a default
        /// constructs to `None` until the runtime attribution threader lands.
        attribution: Option<AttributionFrame>,
    },
    /// WAIT primitive drove the evaluator to suspension. Emitted as the
    /// terminal step for the suspended invocation (§9.1 G3-A).
    SuspendBoundary {
        /// CID of the persisted `ExecutionStateEnvelope`.
        state_cid: Cid,
    },
    /// Resume re-entered a suspended execution. Emitted as the first step
    /// after `Engine::resume_from_bytes` (§9.1 G3-A).
    ResumeBoundary {
        /// CID of the `ExecutionStateEnvelope` that was resumed.
        state_cid: Cid,
        /// Value handed to the resumed frame as the signal payload.
        signal_value: Value,
    },
    /// Invariant-8 / Phase-2b SANDBOX-fuel budget exhausted (§9.12).
    BudgetExhausted {
        /// `"inv_8_iteration"` | `"sandbox_fuel"`.
        budget_type: &'static str,
        /// How much budget was consumed before firing.
        consumed: u64,
        /// Configured limit.
        limit: u64,
        /// Path of operation-node ids that produced the exhaustion.
        path: Vec<String>,
    },
}

impl TraceStep {
    /// Convenience: return the primitive's `node_id` for `Step` rows;
    /// `None` for boundary / budget rows.
    #[must_use]
    pub fn node_id(&self) -> Option<&str> {
        match self {
            TraceStep::Step { node_id, .. } => Some(node_id.as_str()),
            _ => None,
        }
    }

    /// Inv-14 attribution accessor. `None` for boundary / budget rows in
    /// Phase 2a; will be `Some` once G5-B-ii wires runtime threading.
    #[must_use]
    pub fn attribution(&self) -> Option<&AttributionFrame> {
        match self {
            TraceStep::Step { attribution, .. } => attribution.as_ref(),
            _ => None,
        }
    }

    /// Phase-1 compat: the `duration_us` field on `Step` rows; `0` for
    /// boundary / budget rows.
    #[must_use]
    pub fn duration_us(&self) -> u64 {
        match self {
            TraceStep::Step { duration_us, .. } => *duration_us,
            _ => 0,
        }
    }

    /// Phase-1 compat: the `error` field on `Step` rows; `None` otherwise.
    #[must_use]
    pub fn error(&self) -> Option<&ErrorCode> {
        match self {
            TraceStep::Step { error, .. } => error.as_ref(),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// TRANSFORM grammar parser. Tests drive the public shape.
// ---------------------------------------------------------------------------

pub mod transform {
    //! TRANSFORM expression grammar + parser surface (G6-B).
    //!
    //! Public entry point for the TRANSFORM expression language. The
    //! grammar is a positive allowlist — any construct outside the BNF in
    //! `docs/TRANSFORM-GRAMMAR.md` is rejected at parse time with
    //! `E_TRANSFORM_SYNTAX`. See the crate-internal `expr` module for the
    //! parser, evaluator, and 50+ built-ins.

    use super::ErrorCode;
    use crate::expr::{Expr, parser};

    /// Typed parse error surface. Carries the byte offset of the first
    /// rejected token so the DSL source-map can highlight the right
    /// character.
    #[derive(Debug, Clone)]
    pub struct TransformParseError {
        /// Byte offset of the first rejected token.
        pub offset: usize,
        /// Human-readable diagnostic reason.
        pub message: String,
        /// Original expression source (echoed for the DX layer).
        pub source: String,
    }

    impl TransformParseError {
        /// Stable [`ErrorCode`] for TRANSFORM parse failures.
        #[must_use]
        pub fn code(&self) -> ErrorCode {
            ErrorCode::TransformSyntax
        }

        /// Byte offset of the first rejected token.
        #[must_use]
        pub fn offset(&self) -> usize {
            self.offset
        }

        /// Offending expression text.
        #[must_use]
        pub fn expression(&self) -> &str {
            &self.source
        }

        /// Human-readable diagnostic reason.
        #[must_use]
        pub fn reason(&self) -> &str {
            &self.message
        }

        /// Pointer to the BNF + denylist documentation file.
        #[must_use]
        pub fn grammar_doc(&self) -> &'static str {
            "docs/TRANSFORM-GRAMMAR.md"
        }
    }

    /// Introspectable AST — wraps an [`Expr`] so tests can assert the
    /// allowlist-only invariant.
    #[derive(Debug, Clone)]
    pub struct AstIntrospect {
        expr: Expr,
    }

    impl AstIntrospect {
        /// The load-bearing fuzz-harness property: every node in the AST
        /// is one of the grammar's admitted variants. This is vacuously
        /// true for any AST the [`parse_transform`] function produces
        /// because the parser's admitted types *are* the allowlist.
        #[must_use]
        pub fn uses_only_allowlisted_nodes(&self) -> bool {
            self.expr.uses_only_allowlisted_nodes()
        }

        /// Borrow the underlying [`Expr`] (crate-internal use).
        #[must_use]
        pub fn expr(&self) -> &Expr {
            &self.expr
        }
    }

    /// Parse a TRANSFORM expression string.
    ///
    /// # Errors
    ///
    /// Returns [`TransformParseError`] (code `E_TRANSFORM_SYNTAX`) for any
    /// construct outside the grammar's positive allowlist.
    pub fn parse_transform(input: &str) -> Result<AstIntrospect, TransformParseError> {
        match parser::parse(input) {
            Ok(expr) => Ok(AstIntrospect { expr }),
            Err(err) => Err(TransformParseError {
                offset: err.offset,
                message: err.message,
                source: input.to_string(),
            }),
        }
    }
}
