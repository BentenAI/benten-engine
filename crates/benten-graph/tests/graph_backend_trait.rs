//! G13-A wave-1 (canary): GREEN-PHASE pins for the `GraphBackend`
//! umbrella trait.
//!
//! Pin sources (per r2-test-landscape §2.1 G13-A + plan §3 G13-A
//! must-pass column):
//!
//! - `tests/graph_backend_trait_surface_complete` — plan §3 G13-A
//! - `tests/redb_backend_impls_graph_backend` — plan §3 G13-A
//! - `tests/graph_backend_not_object_safe_per_d_phase_3_1_resolution` — `arch-r1-2` BLOCKER
//! - `tests/graph_backend_snapshot_send_sync_static_for_all_backends` — `arch-r1-6`
//!
//! ## Trait surface (per plan §3 G13-A row + landed at G13-A wave-1)
//!
//! ```text
//! pub trait GraphBackend: KVBackend + NodeStore + EdgeStore {
//!     type Snapshot: Send + Sync + 'static;
//!     type Error: std::error::Error + Send + Sync + 'static;
//!     type Transaction;
//!     fn transaction(&self) -> Self::Transaction;
//!     fn register_subscriber(&self, sub: Arc<dyn ChangeSubscriber>);
//!     fn snapshot(&self) -> Self::Snapshot;
//!     fn put_node_with_context(&self, ...) -> Result<Cid, <Self as GraphBackend>::Error>;
//! }
//! ```
//!
//! Per D-PHASE-3-1 RESOLVED: NOT object-safe (associated `type Error` +
//! `type Snapshot` + `type Transaction` preclude dyn-erasure). Engine
//! consumes via `Engine<B: GraphBackend>` generic-cascade, never
//! `Arc<dyn GraphBackend>`.

#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use benten_core::{Cid, Node};
use benten_graph::{
    ChangeSubscriber, EdgeStore, GraphBackend, KVBackend, NodeStore, RedbBackend, WriteContext,
};

/// Compile-time witness: every required method + associated type is
/// callable through the trait surface. A refactor that drops
/// `transaction()` / `snapshot()` / `register_subscriber()` /
/// `put_node_with_context()` from the trait fails this test's compile.
#[test]
fn graph_backend_trait_surface_complete() {
    fn assert_trait_complete<B: GraphBackend>() {
        // Method-as-fn-pointer pins — confirm the trait method
        // signatures haven't drifted.
        let _: fn(&B) -> <B as GraphBackend>::Transaction = <B as GraphBackend>::transaction;
        let _: fn(&B) -> <B as GraphBackend>::Snapshot = <B as GraphBackend>::snapshot;
        let _: fn(&B, Arc<dyn ChangeSubscriber>) = <B as GraphBackend>::register_subscriber;
        let _: fn(&B, &Node, &WriteContext) -> Result<Cid, <B as GraphBackend>::Error> =
            <B as GraphBackend>::put_node_with_context;
    }

    // Sub-trait composition pins — confirm the umbrella inherits
    // KVBackend + NodeStore + EdgeStore.
    fn assert_subtrait_composition<B: GraphBackend>() {
        fn require_kv<T: KVBackend + ?Sized>() {}
        fn require_node<T: NodeStore + ?Sized>() {}
        fn require_edge<T: EdgeStore + ?Sized>() {}
        require_kv::<B>();
        require_node::<B>();
        require_edge::<B>();
    }

    assert_trait_complete::<RedbBackend>();
    assert_subtrait_composition::<RedbBackend>();
}

/// Plan §3 G13-A: RedbBackend impls GraphBackend.
///
/// Defends against G13-A landing the trait but forgetting to add an
/// `impl GraphBackend for RedbBackend` adapter.
#[test]
fn redb_backend_impls_graph_backend() {
    fn assert_impl<B: GraphBackend>(_: &B) {}

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("g13a-impl-pin.redb");
    let backend = RedbBackend::create(&path).unwrap();
    assert_impl(&backend);
}

/// arch-r1-2 BLOCKER pin per D-PHASE-3-1 RESOLVED: trait is NOT
/// object-safe by construction (associated types preclude dyn-erasure),
/// and the trait IS usable as a generic bound.
///
/// The non-object-safety is guaranteed BY CONSTRUCTION: the type
/// system rejects `dyn GraphBackend` because of the associated types.
/// We pin the POSITIVE direction here (generic-cascade compiles); the
/// engine-side syntactic-grep pin
/// (`crates/benten-engine/tests/engine_no_dyn_graph_backend.rs`) catches
/// any future regression that re-enables dyn-erasure at engine boundary.
#[test]
fn graph_backend_not_object_safe_per_d_phase_3_1_resolution() {
    fn install_via_generic_cascade<B: GraphBackend>(backend: &B) -> bool {
        let _ = backend.snapshot();
        let _ = backend.transaction();
        true
    }

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("g13a-not-object-safe-pin.redb");
    let backend = RedbBackend::create(&path).unwrap();
    assert!(install_via_generic_cascade(&backend));
}

/// arch-r1-6 pin: associated `type Snapshot` MUST satisfy
/// `Send + Sync + 'static` so the engine can hold a snapshot across
/// `.await` points + worker threads (SUBSCRIBE delivery, IVM
/// materialization, cross-process WAIT-resume).
///
/// G13-C lands BrowserBackend — that wave extends this assertion via
/// its own test pin file for the browser-target Snapshot type.
#[test]
fn graph_backend_snapshot_send_sync_static_for_all_backends() {
    fn assert_snapshot_bounds<B: GraphBackend>() {
        fn assert_send_sync_static<T: Send + Sync + 'static + ?Sized>() {}
        assert_send_sync_static::<<B as GraphBackend>::Snapshot>();
    }
    assert_snapshot_bounds::<RedbBackend>();
}
