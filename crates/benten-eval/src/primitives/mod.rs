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
//! `execute(op) -> Result<StepResult, EvalError>` function the evaluator
//! calls; the primitives themselves are stateless and the Phase-1 test
//! suite drives them with property-carried fixtures (see
//! `tests/primitive_*.rs`). G7's engine layer will extend this with real
//! backend access once the engine handle is stabilised; the Phase-1
//! executors are deliberately pure over the `OperationNode` so the
//! evaluator-scoped tests don't need an engine at all.

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
/// The dispatcher's contract:
///
/// - Phase-1 executable primitives owned by G6-A (`READ`, `WRITE`,
///   `RESPOND`, `EMIT`) delegate to their per-primitive module.
/// - Phase-1 executable primitives owned by G6-B (`TRANSFORM`, `BRANCH`,
///   `ITERATE`, `CALL`) are dispatched by G6-B; today this module calls the
///   evaluator back via the unimplemented arms so `cargo check` passes in
///   parallel commits. G6-B replaces the unimplemented arms with real
///   executors.
/// - Phase-2 primitives (`WAIT`, `STREAM`, `SUBSCRIBE`, `SANDBOX`) return
///   [`EvalError::PrimitiveNotImplemented`] carrying the primitive kind so
///   the test-time message remains precise.
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
        // G6-B scope — TRANSFORM, BRANCH, ITERATE, CALL.
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
