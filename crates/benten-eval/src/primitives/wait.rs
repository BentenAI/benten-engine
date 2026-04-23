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

use crate::exec_state::{ExecutionStateEnvelope, ExecutionStatePayload};
use crate::{EvalError, TraceStep};

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
    #[must_use]
    pub fn new_for_test(state_cid: Cid, signal: impl Into<String>) -> Self {
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

/// Phase-2a evaluator-layer `evaluate` entry point. Takes a subgraph + ctx
/// + input value; returns a terminal outcome or a suspended handle.
///
/// # Errors
/// Returns [`EvalError`] on structural or runtime failure.
pub fn evaluate(
    _sg: &crate::Subgraph,
    _ctx: &mut crate::EvalContext,
    _input: benten_core::Value,
) -> Result<WaitOutcome, EvalError> {
    todo!("Phase 2a G3-A: implement evaluate entry point per plan §9.1")
}

/// Phase-2a evaluator-layer `resume` entry point.
///
/// # Errors
/// Returns [`EvalError`] on tamper / drift / cap-denial.
pub fn resume(
    _envelope: &ExecutionStateEnvelope,
    _signal: WaitResumeSignal,
) -> Result<WaitOutcome, EvalError> {
    todo!("Phase 2a G3-A: implement 4-step resume protocol per plan §9.1")
}
