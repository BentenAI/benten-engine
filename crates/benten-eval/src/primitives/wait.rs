//! Phase 2a G3-A: WAIT primitive executor (stub).
//!
//! The WAIT primitive drives the evaluator to a suspension boundary where
//! `ExecutionStateEnvelope` bytes are persisted and handed back via a
//! [`SuspendedHandle`]. Resume protocol lives in `engine_wait.rs`.
//!
//! TODO(phase-2a-G3-A): implement `execute` + `execute_for_test_signal`
//! per plan §9.1 WAIT semantics; populate the `system:WaitPending` zone.

use benten_core::Cid;

use crate::exec_state::ExecutionStateEnvelope;
use crate::{EvalError, TraceStep};

/// Handle to a suspended WAIT. Opaque, carries the CID of the persisted
/// envelope and a private reference for the resume protocol.
#[derive(Debug, Clone)]
pub struct SuspendedHandle {
    /// CID of the persisted `ExecutionStateEnvelope`.
    state_cid: Cid,
    /// Signal name the suspension is waiting for.
    _signal: String,
}

impl SuspendedHandle {
    /// CID of the persisted execution state.
    #[must_use]
    pub fn state_cid(&self) -> &Cid {
        &self.state_cid
    }
}

/// Outcome of a WAIT invocation at execute time.
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

/// Phase-2a test helper: execute WAIT with a synthesised signal name.
///
/// # Errors
/// Returns [`EvalError`] if the WAIT executor rejects.
pub fn execute_for_test_signal(_signal: &str) -> Result<WaitOutcome, EvalError> {
    todo!("Phase 2a G3-A: implement WAIT executor per plan §9.1")
}

/// Phase-2a test helper: execute WAIT and return the emitted trace alongside.
///
/// # Errors
/// Returns [`EvalError`] if the WAIT executor rejects.
pub fn execute_for_test_signal_with_trace(
    _signal: &str,
) -> Result<(WaitOutcome, Vec<TraceStep>), EvalError> {
    todo!("Phase 2a G3-A: implement WAIT trace emission per plan §9.1 + dx-r1")
}

/// Phase-2a test helper: execute WAIT and capture the system:WaitPending
/// writes it made.
///
/// # Errors
/// Returns [`EvalError`] if the WAIT executor rejects.
pub fn execute_and_capture_zone_writes(_signal: &str) -> Result<ZoneWriteCapture, EvalError> {
    todo!(
        "Phase 2a G3-A: register pending-signal entry in system:WaitPending \
         zone per plan §9.1"
    )
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
