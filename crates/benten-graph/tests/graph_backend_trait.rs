//! R3-A RED-PHASE pins for the `GraphBackend` umbrella trait
//! (G13-A wave 1).
//!
//! Pin sources (per r2-test-landscape §2.1 G13-A + plan §3 G13-A
//! must-pass column):
//!
//! - `tests/graph_backend_trait_surface_complete` — plan §3 G13-A
//! - `tests/redb_backend_impls_graph_backend` — plan §3 G13-A
//! - `tests/graph_backend_not_object_safe_per_d_phase_3_1_resolution` — `arch-r1-2` BLOCKER
//! - `tests/graph_backend_snapshot_send_sync_static_for_all_backends` — `arch-r1-6`
//!
//! ## Trait surface (per plan §3 G13-A row)
//!
//! ```text
//! pub trait GraphBackend: KVBackend + NodeStore + EdgeStore {
//!     type Snapshot: Send + Sync + 'static;
//!     type Error: std::error::Error + Send + Sync + 'static;
//!     type Transaction;
//!     fn transaction(&self) -> Self::Transaction;
//!     fn register_subscriber(&self, sub: Arc<dyn ChangeSubscriber>);
//!     fn snapshot(&self) -> Self::Snapshot;
//!     fn put_node_with_context(&self, ...) -> Result<Cid, Self::Error>;
//! }
//! ```
//!
//! Per D-PHASE-3-1 RESOLVED: NOT object-safe (associated `type Error` +
//! `type Snapshot` + `type Transaction` preclude dyn-erasure). Engine
//! consumes via `Engine<B: GraphBackend>` generic-cascade, never
//! `Arc<dyn GraphBackend>`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G13-A wave 1 introduces benten_graph::GraphBackend"]
fn graph_backend_trait_surface_complete() {
    // G13-A implementer wires this:
    //   use benten_graph::GraphBackend;
    //   fn assert_trait_complete<B: GraphBackend>() {
    //       // Compile-time check: every required method + associated type
    //       // is present on the trait. The function body simply names them
    //       // — if any one is missing, the function fails to compile.
    //       let _: fn(&B) -> <B as GraphBackend>::Transaction = B::transaction;
    //       let _: fn(&B) -> <B as GraphBackend>::Snapshot = B::snapshot;
    //   }
    //   assert_trait_complete::<benten_graph::RedbBackend>();
    //
    // OBSERVABLE consequence: a refactor that drops `transaction()` or
    // `snapshot()` from the trait fails this test's compile. The
    // `register_subscriber` + `put_node_with_context` methods compose
    // at G13-A landing time via the inherited subtraits (NodeStore +
    // ChangeSubscriber registration).
    unimplemented!("G13-A wires trait-surface compile-time completeness assertion");
}

#[test]
#[ignore = "RED-PHASE: G13-A — plan §3 G13-A — RedbBackend impls GraphBackend"]
fn redb_backend_impls_graph_backend() {
    // G13-A implementer wires this:
    //   fn assert_impl<B: benten_graph::GraphBackend>(_: &B) {}
    //   let backend = benten_graph::RedbBackend::create(tempfile::tempdir().unwrap().path()).unwrap();
    //   assert_impl(&backend);
    //
    // OBSERVABLE consequence: RedbBackend (existing Phase-1 impl) also
    // impls the new GraphBackend trait. Defends against G13-A landing
    // the trait but forgetting to add an `impl GraphBackend for
    // RedbBackend` adapter.
    unimplemented!("G13-A wires RedbBackend trait-impl assertion");
}

#[test]
#[ignore = "RED-PHASE: G13-A — arch-r1-2 BLOCKER — generic-cascade not dyn-erased"]
fn graph_backend_not_object_safe_per_d_phase_3_1_resolution() {
    // arch-r1-2 BLOCKER pin per D-PHASE-3-1 RESOLVED. Mirrors the
    // existing `blob_backend_trait_object_safety_per_d_resolution.rs`
    // pattern (Phase-3 wave-1pre, G13-pre-B): the trait is NOT object-
    // safe by construction (associated types preclude dyn-erasure),
    // and the test pins the POSITIVE direction — the trait IS usable
    // as a generic bound.
    //
    // G13-A implementer wires this:
    //   fn install_via_generic_cascade<B: benten_graph::GraphBackend>(backend: &B) -> bool {
    //       let _ = backend.snapshot();
    //       true  // signal that the generic-cascade path compiled
    //   }
    //   let backend = benten_graph::RedbBackend::create(...).unwrap();
    //   assert!(install_via_generic_cascade(&backend));
    //
    // The non-object-safety is guaranteed BY CONSTRUCTION: the type
    // system rejects `dyn GraphBackend` because of the associated
    // types. No separate runtime assertion is required (and any
    // attempt to assert it would itself fail to compile).
    //
    // OBSERVABLE consequence: `Engine<B: GraphBackend>` cascade compiles;
    // a refactor that drops the associated types (re-enabling dyn-
    // erasure) would still pass this positive smoke, but the
    // engine-side cascade test (`engine_generic_compiles_with_redb_default`)
    // would catch it. Together both pins lock the design.
    unimplemented!(
        "G13-A wires generic-cascade positive smoke + relies on type system for negative"
    );
}

#[test]
#[ignore = "RED-PHASE: G13-A — arch-r1-6 — Snapshot: Send + Sync + 'static"]
fn graph_backend_snapshot_send_sync_static_for_all_backends() {
    // arch-r1-6 pin. The associated `type Snapshot` MUST satisfy
    // `Send + Sync + 'static` so the engine can hold a snapshot
    // across `.await` points + worker threads (e.g. for SUBSCRIBE
    // delivery, IVM materialization, cross-process WAIT-resume).
    //
    // G13-A implementer wires this:
    //   fn assert_snapshot_bounds<B: benten_graph::GraphBackend>() {
    //       fn assert_send_sync_static<T: Send + Sync + 'static + ?Sized>() {}
    //       assert_send_sync_static::<<B as benten_graph::GraphBackend>::Snapshot>();
    //   }
    //   assert_snapshot_bounds::<benten_graph::RedbBackend>();
    //   // G13-C lands BrowserBackend — that test extends this assertion
    //   // for the browser-target Snapshot type.
    //
    // OBSERVABLE consequence: any Snapshot impl that ships a !Send /
    // !Sync / borrowed value fails this assertion at compile time.
    unimplemented!("G13-A wires Snapshot: Send + Sync + 'static compile-time bounds assertion");
}
