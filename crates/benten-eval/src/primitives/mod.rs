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
// Phase 2b G7-A SANDBOX executor. Compile-time wasm32-disabled per
// sec-pre-r1-05 (the executor depends on `wasmtime` which doesn't
// build for wasm32).
//
// **cr-g7a-mr-7 fix-pass:** the `pub mod sandbox` declaration is
// `cfg(not(target_arch = "wasm32"))`-gated HERE in addition to the
// inner-attribute `#![cfg(not(target_arch = "wasm32"))]` on the file
// body. The two-layer gating is intentional defence-in-depth: any
// future `pub use` at lib.rs that re-exports symbols from this module
// MUST also carry the cfg-gate or the wasm32 build will fail to find
// the symbol. The mod-level cfg here makes the requirement obvious to
// future maintainers.
#[cfg(not(target_arch = "wasm32"))]
pub mod sandbox;
pub mod stream;
pub mod subscribe;
pub mod transform;
pub mod wait;
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
        // Phase-2b G6-A: STREAM + SUBSCRIBE land their executors here.
        // STREAM allocates a sink + source pair; SUBSCRIBE registers a
        // change-event subscription against the engine's ChangeStream
        // port. Both return `Ok` immediately; runtime delivery is driven
        // by the engine's IVM subscriber.
        PrimitiveKind::Stream => stream::execute(op, host),
        PrimitiveKind::Subscribe => subscribe::execute(op, host),
        // Remaining Phase-2 primitives — structural validation accepts
        // them; the executor rejects at call time until their wave lands.
        PrimitiveKind::Wait | PrimitiveKind::Sandbox => {
            Err(EvalError::PrimitiveNotImplemented(op.kind))
        }
        // PrimitiveKind is `#[non_exhaustive]` (G12-C-cont relocation kept the
        // attribute for downstream-crate version-evolution discipline). Any
        // future variant added at the source-of-truth `benten-core` site
        // surfaces here as `PrimitiveNotImplemented` until a dedicated
        // executor lands.
        _ => Err(EvalError::PrimitiveNotImplemented(op.kind)),
    }
}
