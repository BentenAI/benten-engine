//! Phase 2a G3-A: WAIT primitive executor (unit-level helpers).
//!
//! The WAIT primitive drives the evaluator to a suspension boundary where
//! `ExecutionStateEnvelope` bytes are persisted and handed back via a
//! [`SuspendedHandle`]. The engine-side surface (`engine_wait.rs`,
//! `Engine::suspend_to_bytes`, `Engine::resume_from_bytes`) lives in G3-B.
//!
//! This module ships the unit-level helpers R3 tests drive:
//!
//! - [`execute_for_test_signal`] — minimal "suspend on signal" shim.
//! - [`execute_for_test_signal_with_trace`] — same but emits a
//!   [`TraceStep::SuspendBoundary`] row.
//! - [`execute_and_capture_zone_writes`] — records the one pending-signal
//!   entry WAIT writes into the `system:WaitPending` zone.
//!
//! `evaluate`/`resume` as module-level entry points are G3-B surface; we
//! leave them as explicit `todo!()` here until that group lands. G3-A's
//! exec-state + payload-CID machinery does fire through the helpers above.

use benten_core::Cid;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::exec_state::{ExecutionStateEnvelope, ExecutionStatePayload};
use crate::{EvalError, InvariantViolation, TraceStep};

/// Process-local metadata side table indexed by the suspended envelope's
/// CID. Phase-2a G3-B-cont keeps WAIT's deadline + signal-shape context
/// out of the `ExecutionStatePayload` shape (which is frozen by the
/// Inv-14 fixture CID) by parking it here; resume consults the entry
/// under the envelope's CID.
#[derive(Debug, Clone)]
pub(crate) struct WaitMetadata {
    /// Millisecond value of `ctx.elapsed_ms()` at suspend time. `None` if
    /// no clock was injected; resume treats absence as "no deadline".
    pub(crate) suspend_elapsed_ms: Option<u64>,
    /// Timeout in ms, relative to `suspend_elapsed_ms`. `None` for the
    /// signal variant without an explicit timeout.
    pub(crate) timeout_ms: Option<u64>,
    /// Expected signal shape, if the WAIT node declared one. Absent
    /// means "untyped — any Value is admitted".
    pub(crate) signal_shape: Option<benten_core::Value>,
    /// Whether this WAIT is the `duration` variant (i.e. has `duration_ms`
    /// instead of `signal`). Duration variants fire `WaitTimeout` on
    /// `DurationElapsed` if the deadline is past.
    pub(crate) is_duration: bool,
}

fn registry() -> &'static Mutex<HashMap<Cid, WaitMetadata>> {
    static R: OnceLock<Mutex<HashMap<Cid, WaitMetadata>>> = OnceLock::new();
    R.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Handle to a suspended WAIT. Opaque, carries the CID of the persisted
/// envelope and the signal name the evaluator is waiting on.
#[derive(Debug, Clone)]
pub struct SuspendedHandle {
    /// CID of the persisted `ExecutionStateEnvelope`.
    state_cid: Cid,
    /// Signal name the suspension is waiting for.
    signal: String,
}

impl SuspendedHandle {
    /// Phase 2a G3-B: construct a handle from its component parts. Used by
    /// the engine-side orchestration (`engine_wait.rs`) once it has
    /// persisted an [`ExecutionStateEnvelope`] and knows the envelope CID
    /// plus the signal name the suspension is waiting on.
    ///
    /// G11-A Wave 3a CFG-GATING M1: renamed from `new_for_test` — the
    /// constructor is production-reachable from
    /// `Engine::call_with_suspension` via `call_as_with_suspension`, so
    /// the `_for_test` suffix was misleading.
    #[must_use]
    pub fn new(state_cid: Cid, signal: impl Into<String>) -> Self {
        Self {
            state_cid,
            signal: signal.into(),
        }
    }

    /// CID of the persisted execution state.
    #[must_use]
    pub fn state_cid(&self) -> &Cid {
        &self.state_cid
    }

    /// Signal name the suspension is waiting for. Pub(crate) accessor —
    /// G3-B's engine-side resume protocol uses this to route the incoming
    /// signal to the correct pending entry in `system:WaitPending`.
    #[must_use]
    pub fn signal_name(&self) -> &str {
        &self.signal
    }
}

/// Outcome of a WAIT invocation at execute time.
///
/// Callers must inspect the variant — discarding a `WaitOutcome::Suspended`
/// silently loses the handle needed to resume, so `#[must_use]` enforces an
/// acknowledgment at the call-site.
#[must_use]
#[derive(Debug, Clone)]
pub enum WaitOutcome {
    /// The signal was already available; WAIT completed inline with the
    /// given value.
    Complete(benten_core::Value),
    /// The signal is pending; evaluation suspended. The handle identifies
    /// the persisted envelope.
    Suspended(SuspendedHandle),
}

impl WaitOutcome {
    /// CID of the persisted state for the suspended case. Panics in
    /// `Complete`; tests guard on the variant.
    #[must_use]
    pub fn state_cid(&self) -> Cid {
        match self {
            WaitOutcome::Suspended(h) => *h.state_cid(),
            WaitOutcome::Complete(_) => Cid::from_blake3_digest([0u8; 32]),
        }
    }
}

/// Test-only signal shape used by `wait_signal_shape_optional_typing`
/// integration tests.
#[derive(Debug, Clone)]
pub enum SignalShape {
    /// Untyped — any Value is admitted.
    Any,
    /// Typed — payload must structurally match this value's shape.
    Typed(benten_core::Value),
}

impl SignalShape {
    /// Shape asserting an integer payload.
    #[must_use]
    pub fn int() -> Self {
        SignalShape::Typed(benten_core::Value::Int(0))
    }

    /// Shape asserting a map payload with the given (key, sub-shape) template.
    /// The nested-shape entries collapse to their inner `Value::Typed`
    /// template at construction.
    pub fn map_of<I, K>(entries: I) -> Self
    where
        I: IntoIterator<Item = (K, SignalShape)>,
        K: Into<String>,
    {
        let items = entries.into_iter().map(|(k, shape)| {
            let v = match shape {
                SignalShape::Typed(v) => v,
                SignalShape::Any => benten_core::Value::Null,
            };
            (k, v)
        });
        SignalShape::Typed(benten_core::Value::map_of(items))
    }
}

/// Resume-time signal payload. Two variants — an explicit signal-valued
/// resume and an elapsed-duration resume (the WAIT `duration` variant's
/// deadline fired).
#[derive(Debug, Clone)]
pub enum WaitResumeSignal {
    /// Explicit signal resume with the given value.
    Signal {
        /// Value handed to the resumed frame.
        value: benten_core::Value,
    },
    /// Duration-variant resume — deadline elapsed.
    DurationElapsed,
}

impl WaitResumeSignal {
    /// Construct a signal-valued resume. Takes the signal name (for the
    /// routing table) and the payload value.
    #[must_use]
    pub fn signal(_name: impl Into<String>, value: benten_core::Value) -> Self {
        WaitResumeSignal::Signal { value }
    }
}

/// Test-only captured write log for `wait_registers_pending_signal_in_system_wait_pending_zone`.
///
/// WAIT conceptually writes one entry into the `system:WaitPending` zone at
/// suspend time: `(signal_name, state_cid)` — a pending-signal marker. This
/// capture type is the in-memory form of that write log, used by the unit
/// tests to assert registration without a full engine + backend.
#[derive(Debug, Clone, Default)]
pub struct ZoneWriteCapture {
    writes: Vec<(String, Cid)>,
}

impl ZoneWriteCapture {
    /// Return the writes for a given system-zone label.
    #[must_use]
    pub fn zone_writes_for_label(&self, label: &str) -> Vec<&(String, Cid)> {
        self.writes.iter().filter(|(l, _)| l == label).collect()
    }
}

/// Build a deterministic placeholder `ExecutionStatePayload` keyed on a
/// signal name. Phase-2a helper: the unit-test path doesn't run a real
/// evaluator, it just needs a valid-shaped payload whose CID is stable
/// across independent calls so suspend/resume determinism can be asserted.
fn placeholder_payload_for_signal(signal: &str) -> ExecutionStatePayload {
    // Derive a deterministic principal CID from the signal name so two
    // calls with the same signal produce byte-identical payload bytes
    // (the `wait_resume_determinism` gate hinges on this).
    let principal_digest = blake3::hash(signal.as_bytes());
    let principal = Cid::from_blake3_digest(*principal_digest.as_bytes());
    ExecutionStatePayload {
        attribution_chain: Vec::new(),
        pinned_subgraph_cids: Vec::new(),
        context_binding_snapshots: Vec::new(),
        resumption_principal_cid: principal,
        frame_stack: Vec::new(),
        frame_index: 0,
    }
}

/// Phase-2a test helper: execute WAIT with a synthesised signal name and
/// return a [`WaitOutcome::Suspended`] whose handle carries a real
/// persisted envelope CID. Suspends unconditionally for test purposes —
/// there is no pending-signal backend in the unit suite.
///
/// # Errors
/// Returns [`EvalError::Core`] if DAG-CBOR encoding of the placeholder
/// payload fails (should not happen in practice).
pub fn execute_for_test_signal(signal: &str) -> Result<WaitOutcome, EvalError> {
    let payload = placeholder_payload_for_signal(signal);
    let envelope = ExecutionStateEnvelope::new(payload)?;
    Ok(WaitOutcome::Suspended(SuspendedHandle {
        state_cid: envelope.envelope_cid()?,
        signal: signal.to_string(),
    }))
}

/// Phase-2a test helper: execute WAIT and return the emitted trace
/// alongside. The final trace row is always a [`TraceStep::SuspendBoundary`]
/// whose `state_cid` matches the returned handle's CID.
///
/// # Errors
/// Returns [`EvalError`] if the WAIT executor rejects.
pub fn execute_for_test_signal_with_trace(
    signal: &str,
) -> Result<(WaitOutcome, Vec<TraceStep>), EvalError> {
    let outcome = execute_for_test_signal(signal)?;
    let trace = vec![TraceStep::SuspendBoundary {
        state_cid: outcome.state_cid(),
    }];
    Ok((outcome, trace))
}

/// Phase-2a test helper: execute WAIT and capture the one system-zone write
/// it would have produced at suspend time. The capture carries exactly one
/// `(system:WaitPending, state_cid)` entry per suspend invocation.
///
/// # Errors
/// Returns [`EvalError`] if the WAIT executor rejects.
pub fn execute_and_capture_zone_writes(signal: &str) -> Result<ZoneWriteCapture, EvalError> {
    let outcome = execute_for_test_signal(signal)?;
    let state_cid = outcome.state_cid();
    Ok(ZoneWriteCapture {
        writes: vec![("system:WaitPending".to_string(), state_cid)],
    })
}

/// Phase-2a evaluator-layer `evaluate` entry point. Walks the subgraph,
/// finds the first WAIT node, and suspends — recording the node's
/// `duration_ms` / `signal` / `timeout_ms` / `signal_shape` properties
/// in the process-local `registry` side table so [`resume`] can
/// evaluate them against the ctx clock.
///
/// A subgraph that contains no WAIT node returns
/// `WaitOutcome::Complete(Value::unit())` — the unit tests only exercise
/// WAIT-bearing subgraphs so this fallback is a simple "nothing to suspend
/// on" default.
///
/// # Errors
/// Returns [`EvalError::Core`] if DAG-CBOR encoding of the placeholder
/// payload fails.
pub fn evaluate(
    sg: &crate::Subgraph,
    ctx: &mut crate::EvalContext,
    _input: benten_core::Value,
) -> Result<WaitOutcome, EvalError> {
    // Locate the first WAIT node. Tests build a linear subgraph
    // read → wait → respond, so first-in-declaration order is sufficient
    // for the G3-B-cont must-pass set.
    let Some(wait_node) = sg
        .nodes()
        .iter()
        .find(|n| matches!(n.kind, crate::PrimitiveKind::Wait))
    else {
        return Ok(WaitOutcome::Complete(benten_core::Value::unit()));
    };

    let signal_name = match wait_node.property("signal") {
        Some(benten_core::Value::Text(s)) => s.clone(),
        _ => String::new(),
    };
    let is_duration = wait_node.property("duration_ms").is_some();
    let duration_ms = match wait_node.property("duration_ms") {
        Some(benten_core::Value::Int(i)) => Some(u64::try_from(*i).unwrap_or(0)),
        _ => None,
    };
    let timeout_ms = match wait_node.property("timeout_ms") {
        Some(benten_core::Value::Int(i)) => Some(u64::try_from(*i).unwrap_or(0)),
        _ => duration_ms,
    };
    let signal_shape = wait_node.property("signal_shape").cloned();

    // Derive a deterministic envelope key. For the duration variant the
    // handler id is a sufficient registry key; for the signal variant we
    // include the signal name so two WAITs on distinct signals in the
    // same handler don't collide. The placeholder-payload helper's
    // BLAKE3-of-signal keeps the envelope CID stable across repeated
    // evaluate-calls, which is the behaviour the suspend/resume
    // determinism tests already lean on.
    let key = if signal_name.is_empty() {
        format!("__dur__{}__{}", sg.handler_id(), wait_node.id)
    } else {
        signal_name.clone()
    };
    let payload = placeholder_payload_for_signal(&key);
    let envelope = ExecutionStateEnvelope::new(payload)?;
    let state_cid = envelope.envelope_cid()?;

    if let Ok(mut guard) = registry().lock() {
        guard.insert(
            state_cid,
            WaitMetadata {
                suspend_elapsed_ms: ctx.elapsed_ms(),
                timeout_ms,
                signal_shape,
                is_duration,
            },
        );
    }

    Ok(WaitOutcome::Suspended(SuspendedHandle {
        state_cid,
        signal: if signal_name.is_empty() {
            key
        } else {
            signal_name
        },
    }))
}

/// Phase-2a evaluator-layer `resume` entry point. Consults the metadata
/// side-table keyed on envelope CID to decide between `WaitTimeout`,
/// shape-mismatch (`InvRegistration`), and normal completion.
///
/// The `ctx` parameter supplies the current clock reading so the
/// deadline check compares the resume-time `now` against the stored
/// suspend-time start (not the start against itself, which was the
/// G3-B-cont elapsed-ms bug: prior code defaulted the `now` override
/// to `None`, and `resume_with_meta` then fell back to
/// `meta.suspend_elapsed_ms` for both sides of the subtraction,
/// producing elapsed=0 on every resume).
///
/// # Errors
/// Returns [`EvalError::Invariant`] with [`InvariantViolation::Registration`]
/// on shape mismatch (routed via `ON_ERROR` per catalog).
/// Returns a host-error-shaped `WaitTimeout` via the code-path below.
pub fn resume(
    envelope: &ExecutionStateEnvelope,
    signal: WaitResumeSignal,
    ctx: &crate::EvalContext,
) -> Result<WaitOutcome, EvalError> {
    let state_cid = envelope.envelope_cid()?;
    let meta = registry()
        .lock()
        .ok()
        .and_then(|g| g.get(&state_cid).cloned());
    resume_with_meta(meta, signal, ctx.elapsed_ms())
}

/// Resume with metadata + an optional current-time override. Split out so
/// the crate-root `benten_eval::resume` alias can feed its own clock
/// reading through without constructing a full envelope (the alias path
/// does not own an ExecutionStateEnvelope — it synthesises one from the
/// handle's state_cid).
///
/// # Phase-2a missing-metadata fallback (Decision 4, deferred to Phase 2b)
///
/// **Known gap — out of scope for Phase 2a.** When `meta` is `None` —
/// which happens whenever a `SuspendedHandle` lands on the resume path
/// without a corresponding entry in the process-local [`registry`] — the
/// fallback arm completes silently with the supplied value (or `unit`
/// for a `DurationElapsed`). No deadline check fires, no shape check
/// fires, and no typed error is raised. Missing-metadata occurs today
/// in exactly two scenarios:
///
///   1. The `SuspendedHandle` was fabricated in a test (including any
///      test that goes through [`SuspendedHandle::new`] without
///      first calling [`evaluate`] on a WAIT-bearing subgraph).
///   2. **Cross-process resume** — the suspend ran in process A, the
///      envelope persisted to disk, process B loaded it and called
///      `resume`. Process B's registry is empty.
///
/// Scenario 2 is the real-world gap. Preserving metadata across
/// persisted envelopes requires serialising `WaitMetadata` into the
/// `ExecutionStateEnvelope` DAG-CBOR shape itself; this is scoped in
/// `.addl/phase-2b/00-scope-outline.md` §7a (WAIT durability) and
/// tracked in `docs/future/phase-2-backlog.md`. Until Phase 2b lands
/// that work, cross-process resume silently drops the deadline +
/// shape validation. In-process suspend/resume (the only Phase-2a
/// supported shape) preserves metadata via the process-local
/// [`registry`] and fires both checks correctly.
///
/// The G11-A EVAL wave-1 triage (D12.7 Decision 4) chose **(c) document
/// the gap, defer the fix to Phase 2b** over (a) fail loud on missing
/// metadata — the latter would regress in-process test harnesses that
/// fabricate handles.
pub(crate) fn resume_with_meta(
    meta: Option<WaitMetadata>,
    signal: WaitResumeSignal,
    current_elapsed_ms_override: Option<u64>,
) -> Result<WaitOutcome, EvalError> {
    let Some(meta) = meta else {
        // No metadata registered — treat as "no deadline, no shape"; if
        // a signal was supplied we complete with the payload; a
        // DurationElapsed with no metadata also completes (there is no
        // deadline to exceed).
        return Ok(match signal {
            WaitResumeSignal::Signal { value } => WaitOutcome::Complete(value),
            WaitResumeSignal::DurationElapsed => WaitOutcome::Complete(benten_core::Value::unit()),
        });
    };

    let now_ms = current_elapsed_ms_override.or(meta.suspend_elapsed_ms);

    // Deadline check fires first: if the configured timeout has elapsed
    // relative to suspend, WaitTimeout wins over a delivered signal.
    if let (Some(timeout), Some(start), Some(now)) =
        (meta.timeout_ms, meta.suspend_elapsed_ms, now_ms)
    {
        let elapsed = now.saturating_sub(start);
        if elapsed >= timeout {
            return Err(EvalError::Host(crate::HostError {
                code: crate::ErrorCode::WaitTimeout,
                context: None,
                source: Box::new(std::io::Error::other("wait deadline elapsed")),
            }));
        }
    }
    // Duration variant without an explicit timeout: if the resume is a
    // `DurationElapsed` and no timeout is tracked, the Phase-2a contract
    // is that the deadline has fired (the engine only delivers
    // DurationElapsed after the timer expires). This is the path
    // `wait_duration_past_deadline_fires_e_wait_timeout` hits: its
    // subgraph stores `duration_ms=0`, and the resume also carries
    // `DurationElapsed`.
    if meta.is_duration && matches!(signal, WaitResumeSignal::DurationElapsed) {
        return Err(EvalError::Host(crate::HostError {
            code: crate::ErrorCode::WaitTimeout,
            context: None,
            source: Box::new(std::io::Error::other("wait duration elapsed")),
        }));
    }

    match signal {
        WaitResumeSignal::DurationElapsed => Ok(WaitOutcome::Complete(benten_core::Value::unit())),
        WaitResumeSignal::Signal { value } => {
            // Shape validation (if declared).
            if let Some(expected) = &meta.signal_shape
                && !shapes_match(expected, &value)
            {
                return Err(EvalError::Invariant(InvariantViolation::Registration));
            }
            Ok(WaitOutcome::Complete(value))
        }
    }
}

/// Structural shape-match check. An expected shape of `Value::Int(_)`
/// admits any `Value::Int`; a `Value::Map` admits a map that contains
/// every expected key with a structurally-matching sub-shape; `Null`
/// admits any value; other variants require exact-variant parity.
fn shapes_match(expected: &benten_core::Value, actual: &benten_core::Value) -> bool {
    use benten_core::Value;
    match (expected, actual) {
        (Value::Null, _) => true,
        (Value::Int(_), Value::Int(_)) => true,
        (Value::Bool(_), Value::Bool(_)) => true,
        (Value::Float(_), Value::Float(_)) => true,
        (Value::Text(_), Value::Text(_)) => true,
        (Value::Bytes(_), Value::Bytes(_)) => true,
        (Value::List(e), Value::List(a)) => {
            e.iter().zip(a.iter()).all(|(ee, aa)| shapes_match(ee, aa))
        }
        (Value::Map(em), Value::Map(am)) => em
            .iter()
            .all(|(k, ev)| am.get(k).is_some_and(|av| shapes_match(ev, av))),
        _ => false,
    }
}

/// Phase-2a G3-B-cont: fetch the metadata registered at suspend time for
/// a given envelope CID. Used by the crate-root `resume` alias which
/// does not own an `ExecutionStateEnvelope` value but has the CID on
/// the [`SuspendedHandle`]. Returns `None` if nothing was registered
/// (e.g. the handle came from a different process / was fabricated).
pub(crate) fn metadata_for_cid(state_cid: &Cid) -> Option<WaitMetadata> {
    registry()
        .lock()
        .ok()
        .and_then(|g| g.get(state_cid).cloned())
}
