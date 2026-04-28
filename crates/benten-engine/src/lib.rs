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
//! - [`subgraph_spec`] — DSL builders (`SubgraphSpec`, `PrimitiveSpec`,
//!   `WriteSpec`, …).
//! - [`outcome`] — response shapes (`Outcome`, `Trace`, …).
//! - [`engine_transaction`] — `EngineTransaction` passed into the
//!   `.transaction(|tx| …)` closure.
//! - [`change`] — [`change::ChangeBroadcast`] subscriber fan-out.
//! - [`change_probe`] — [`ChangeProbe`] observation handle.
//! - `testing` — integration-test helpers consumed by sibling crates
//!   (cfg-gated behind `any(test, feature = "test-helpers")`; not part
//!   of the production cdylib surface).
//!
//! `lib.rs` is deliberately thin after the R6 Wave 2 split (R-major-01): it
//! declares the modules and re-exports the full public surface so call-site
//! paths like `benten_engine::Engine` stay stable.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(
    missing_docs,
    reason = "TODO(phase-2b-docs): benten-engine orchestrator exposes ~60 pub items across EngineBuilder, Engine, SubgraphSpec builder, Outcome/Trace response shapes, and EngineError variants. Crate-root + module-root docs land in R6 (see lib.rs / engine.rs / outcome.rs module headers); per-item sweep deferred to Phase-2b when the public surface is re-audited post-evaluator-completion and the SubgraphSpec DSL stabilises."
)]
#![allow(
    clippy::todo,
    reason = "Phase-1 scope: primitive-dispatch deliverables remain as typed todos until benten-eval's evaluator gains a PrimitiveHost trait (Phase 2)."
)]

pub mod builder;
pub mod change;
pub mod change_probe;
pub mod engine;
pub(crate) mod engine_caps;
// Phase 2b G7-A — workspace-level `engine.toml` loader (Ben's brief
// addition). Compile-time wasm32-disabled per the same sec-pre-r1-05
// cut as SANDBOX itself; on wasm32 the engine config is always
// built-in defaults.
#[cfg(not(target_arch = "wasm32"))]
pub mod engine_config;
pub(crate) mod engine_crud;
pub(crate) mod engine_diagnostics;
pub mod engine_transaction;
pub(crate) mod engine_views;
pub mod error;
pub mod outcome;
pub mod primitive_host;
pub mod subgraph_spec;
// R6 round-2 sec-r6r2-02: gate the test-helper surface (`principal_cid`,
// `minimal_wait_handler`, `policy_with_grants`, `counting_capability_policy`,
// `subgraph_bytes_for_handler`, etc.) behind `cfg(any(test, feature =
// "test-helpers"))` so the napi cdylib (which opts into the narrower
// `envelope-cache-test-grade` feature only) does NOT compile this surface
// into production. Sibling crates' integration tests reach in via dev-deps
// that already declare `benten-engine = { features = ["test-helpers"] }`
// (see `benten-eval/Cargo.toml:66`, `benten-graph/Cargo.toml:86`,
// `benten-caps/Cargo.toml:40`), so the test path is unaffected.
#[cfg(any(test, feature = "test-helpers"))]
pub mod testing;

// ---------------------------------------------------------------------------
// Public re-exports — preserve every call-site path that existed before the
// R6 Wave 2 split.
// ---------------------------------------------------------------------------

pub use benten_errors::ErrorCode;
pub use benten_eval::PrimitiveKind;

#[cfg(not(target_arch = "wasm32"))]
pub use engine_config::{EngineConfig, EngineConfigError, SandboxSection};

pub use builder::{EngineBuilder, NOAUTH_STARTUP_LOG};
pub use change_probe::ChangeProbe;
pub use engine::{CHANGE_STREAM_MAX_BUFFERED, Engine};
pub use engine_transaction::EngineTransaction;
pub use error::EngineError;
pub use outcome::{
    AnchorHandle, BudgetExhaustedView, DiagnosticInfo, HandlerPredecessors, NestedTx, Outcome,
    OutcomeExt, ReadViewOptions, TerminalError, Trace, TraceStep, UserViewInputPattern,
    UserViewSpec, UserViewSpecBuilder, ViewCreateOptions,
};
pub use subgraph_spec::{
    GrantSubject, IntoCallInput, IntoSubgraphSpec, IterateBody, PrimitiveSpec, RevokeScope,
    RevokeSubject, SubgraphSpec, SubgraphSpecBuilder, WriteSpec,
};

pub mod engine_sandbox;
pub mod engine_stream;
pub mod engine_subscribe;
pub mod engine_wait;
pub mod system_zones;

pub use benten_eval::chunk_sink::{Chunk, ChunkSink};
pub use engine_sandbox::{SANDBOX_UNAVAILABLE_ON_WASM_TEXT, SandboxNodeDescription};
pub use engine_stream::{StreamCursor, StreamHandle};
pub use engine_subscribe::{OnChangeCallback, SubscribeCursor, Subscription};
pub use engine_wait::SuspensionOutcome;
pub use system_zones::SYSTEM_ZONE_PREFIXES;

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
