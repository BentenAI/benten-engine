//! Executor shims for the twelve operation primitives.
//!
//! Phase-1 executable primitives land as submodules; Phase-2 primitives
//! (`WAIT`, `STREAM`, `SUBSCRIBE`-as-user-op, `SANDBOX`) are recognized by
//! the dispatcher and rejected at call time with
//! [`EvalError::PrimitiveNotImplemented`].
//! This keeps Phase-1 and Phase-2 subgraphs binary-compatible: a subgraph
//! containing a Phase-2 primitive still passes structural validation and
//! round-trips through storage, but is inert at execution time until the
//! executor lands.
//!
//! The dispatcher is thin on purpose. Each per-primitive module exposes an
//! `execute(op)` or `execute(op, host)` function the evaluator calls;
//! primitives themselves are stateless and the Phase-1 test suite drives
//! them with property-carried fixtures (see `tests/primitive_*.rs`). The
//! engine layer (`benten-engine`) supplies a real `PrimitiveHost`
//! implementation at call time; the Phase-1 unit-test suite uses
//! [`NullHost`](crate::NullHost) so per-primitive tests don't need an
//! engine at all.

use crate::{EvalError, OperationNode, PrimitiveHost, PrimitiveKind, StepResult};

pub mod branch;
pub mod call;
pub mod emit;
pub mod iterate;
pub mod read;
pub mod respond;
pub mod transform;
pub mod write;

/// Dispatch a single primitive execution by kind.
///
/// Dispatches to the per-primitive executor in this module; Phase-2
/// primitives (`WAIT`, `STREAM`, `SUBSCRIBE`-as-user-op, `SANDBOX`) return
/// [`EvalError::PrimitiveNotImplemented`] carrying the primitive kind so
/// the test-time message remains precise.
///
/// The dispatcher's contract:
///
/// - Phase-1 executable primitives (`READ`, `WRITE`, `RESPOND`, `EMIT`,
///   `TRANSFORM`, `BRANCH`, `ITERATE`, `CALL`) each resolve to a real
///   executor in the matching submodule.
/// - Phase-2 primitives (`WAIT`, `STREAM`, `SUBSCRIBE`, `SANDBOX`) pass
///   structural validation (so Phase-1 and Phase-2 subgraphs are binary-
///   compatible and round-trip through storage) but return
///   [`EvalError::PrimitiveNotImplemented`] when the evaluator reaches
///   them at call time.
///
/// # Errors
///
/// Returns [`EvalError::PrimitiveNotImplemented`] for any primitive whose
/// executor is not yet implemented. Individual primitives may surface
/// other [`EvalError`] variants per their own contracts.
pub fn dispatch(op: &OperationNode, host: &dyn PrimitiveHost) -> Result<StepResult, EvalError> {
    match op.kind {
        PrimitiveKind::Read => read::execute(op, host),
        PrimitiveKind::Write => write::execute(op, host),
        PrimitiveKind::Respond => respond::execute(op),
        PrimitiveKind::Emit => emit::execute(op, host),
        PrimitiveKind::Transform => transform::execute(op),
        PrimitiveKind::Branch => branch::execute(op),
        PrimitiveKind::Iterate => iterate::execute(op, host),
        PrimitiveKind::Call => call::execute(op, host),
        // Phase-2 primitives — structural validation accepts them, the
        // executor rejects at call time.
        PrimitiveKind::Wait
        | PrimitiveKind::Stream
        | PrimitiveKind::Subscribe
        | PrimitiveKind::Sandbox => Err(EvalError::PrimitiveNotImplemented(op.kind)),
    }
}
