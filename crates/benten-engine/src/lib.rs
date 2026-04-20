//! # benten-engine
//!
//! Orchestrator crate composing the Benten graph engine public API.
//!
//! [`EngineBuilder`] selects capability policy, IVM subscriber, production
//! mode guard, and durability. [`Engine`] exposes CRUD (Node + Edge),
//! `register_subgraph` (runs G6 invariants), `transaction` (closure over
//! [`benten_graph::Transaction`]), `snapshot` (MVCC handle), and the three
//! privileged system-zone entry points
//! `grant_capability` / `create_view` / `revoke_capability`.
//! [`change::ChangeBroadcast`] fans committed events to every registered
//! subscriber.
//!
//! Call-time primitive dispatch (register_crud → evaluator → primitive
//! execution) is threaded through `impl PrimitiveHost for Engine`
//! (see [`primitive_host`]) — host-side writes are buffered inside the
//! active-call frame and replayed atomically at commit.
//!
//! ## Module layout (post R6 Wave 2 split)
//!
//! - [`error`] — [`EngineError`] + conversions.
//! - [`builder`] — [`EngineBuilder`] fluent surface + `BackendGrantReader`.
//! - [`engine`] — [`Engine`] struct + CRUD / register / dispatch /
//!   view-read / transaction / snapshot / diagnostics.
//! - [`primitive_host`] — `impl PrimitiveHost for Engine` + the buffered
//!   replay state (`ActiveCall` / `PendingHostOp`).
//! - [`subgraph_spec`] — DSL builders (`SubgraphSpec`, `WriteSpec`, …).
//! - [`outcome`] — response shapes (`Outcome`, `Trace`, …).
//! - [`engine_transaction`] — `EngineTransaction` passed into the
//!   `.transaction(|tx| …)` closure.
//! - [`change`] — [`change::ChangeBroadcast`] subscriber fan-out.
//! - [`change_probe`] — [`ChangeProbe`] observation handle.
//! - [`testing`] — integration-test helpers consumed by sibling crates.
//!
//! `lib.rs` is deliberately thin after the R6 Wave 2 split (R-major-01): it
//! declares the modules and re-exports the full public surface so call-site
//! paths like `benten_engine::Engine` stay stable.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(
    missing_docs,
    reason = "TODO(phase-2-docs): benten-engine orchestrator exposes ~60 pub items across EngineBuilder, Engine, SubgraphSpec builder, Outcome/Trace response shapes, and EngineError variants. Crate-root + module-root docs land in R6 (see lib.rs / engine.rs / outcome.rs module headers); per-item sweep deferred to Phase-2 when the public surface is re-audited post-evaluator-completion and the SubgraphSpec DSL stabilises."
)]
#![allow(
    clippy::todo,
    reason = "Phase-1 scope: primitive-dispatch deliverables remain as typed todos until benten-eval's evaluator gains a PrimitiveHost trait (Phase 2)."
)]

pub mod builder;
pub mod change;
pub mod change_probe;
pub mod engine;
pub mod engine_transaction;
pub mod error;
pub mod outcome;
pub mod primitive_host;
pub mod subgraph_spec;
pub mod testing;

// ---------------------------------------------------------------------------
// Stub-crate markers
// ---------------------------------------------------------------------------
//
// Touch the stub crates so the dependency graph is real, not just declared.
// TODO(phase-1-cleanup, G8): retire these three `const _:` assertions together
// with the `STUB_MARKER` constants in benten-caps / benten-eval / benten-ivm
// once those crates are no longer stub-phase (R-minor-08). Kept for now so
// any stealth dependency removal surfaces as a compile error rather than a
// silent regression.
#[allow(
    dead_code,
    reason = "stub-marker assertions — see R-minor-08 for Phase-1 retirement"
)]
const _BENTEN_CAPS_MARKER: &str = benten_caps::STUB_MARKER;
#[allow(
    dead_code,
    reason = "stub-marker assertions — see R-minor-08 for Phase-1 retirement"
)]
const _BENTEN_EVAL_MARKER: &str = benten_eval::STUB_MARKER;
#[allow(
    dead_code,
    reason = "stub-marker assertions — see R-minor-08 for Phase-1 retirement"
)]
const _BENTEN_IVM_MARKER: &str = benten_ivm::STUB_MARKER;

// ---------------------------------------------------------------------------
// Public re-exports — preserve every call-site path that existed before the
// R6 Wave 2 split.
// ---------------------------------------------------------------------------

pub use benten_core::ErrorCode;
pub use benten_eval::PrimitiveKind;

pub use builder::EngineBuilder;
pub use change_probe::ChangeProbe;
pub use engine::{CHANGE_STREAM_MAX_BUFFERED, Engine};
pub use engine_transaction::EngineTransaction;
pub use error::EngineError;
pub use outcome::{
    AnchorHandle, HandlerPredecessors, NestedTx, Outcome, OutcomeExt, ReadViewOptions,
    TerminalError, Trace, TraceStep, ViewCreateOptions,
};
pub use subgraph_spec::{
    GrantSubject, IntoCallInput, IntoSubgraphSpec, IterateBody, RevokeScope, RevokeSubject,
    SubgraphSpec, SubgraphSpecBuilder, WriteSpec,
};

// ---------------------------------------------------------------------------
// Tests — lightweight smoke coverage; the heavy lifting lives in
// `tests/` + `tests/integration/`.
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests and benches may use unwrap/expect per workspace policy"
)]
mod tests {
    use super::*;
    use benten_core::testing::canonical_test_node;

    #[test]
    fn create_then_get_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
        let node = canonical_test_node();
        let cid = engine.create_node(&node).unwrap();
        let fetched = engine.get_node(&cid).unwrap().expect("node exists");
        assert_eq!(fetched, node);
        assert_eq!(fetched.cid().unwrap(), cid);
    }

    #[test]
    fn missing_cid_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
        let cid = canonical_test_node().cid().unwrap();
        assert!(engine.get_node(&cid).unwrap().is_none());
    }
}
