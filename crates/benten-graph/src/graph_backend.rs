//! [`GraphBackend`] — the umbrella storage-layer trait introduced at G13-A
//! (Phase-3 R5 wave-1 canary).
//!
//! ## Why this trait exists
//!
//! Phase-1 + Phase-2 shipped the storage layer with three composable
//! sub-traits — [`KVBackend`], [`NodeStore`], and [`EdgeStore`] — plus a
//! handful of inherent `RedbBackend`-specific methods
//! ([`RedbBackend::snapshot`], [`RedbBackend::register_subscriber`],
//! [`RedbBackend::put_node_with_context`], the closure-based
//! `RedbBackend::transaction`). The engine consumed `RedbBackend` directly,
//! which baked the redb dependency into every layer above — including the
//! `wasm32-unknown-unknown` browser-target build (Phase-2-backlog §1.1
//! PHASE-3-BUNDLE-1).
//!
//! G13-A extracts an *umbrella* trait that composes the three subtraits
//! plus the previously-inherent methods that any production-grade graph
//! backend must expose. G13-B (the next wave) introduces
//! `EngineGeneric<B: GraphBackend>` and threads `B` through every consumer
//! site, freeing the engine from the concrete `RedbBackend` type and
//! letting `BrowserBackend` (G13-C) substitute in for the browser bundle.
//!
//! ## Design constraints (load-bearing — see `arch-r1-2` + `arch-r1-6`
//! + `D-PHASE-3-1` RESOLVED)
//!
//! 1. **Trait is intentionally NOT object-safe.** `type Error` + `type
//!    Snapshot` + `type Transaction` are associated types whose presence
//!    precludes `dyn GraphBackend` materialization at compile time. The
//!    engine consumes `GraphBackend` exclusively via the
//!    *generic-cascade* direction (`Engine<B: GraphBackend>` parameters),
//!    never `Arc<dyn GraphBackend>` / `Box<dyn GraphBackend>` —
//!    this is the load-bearing per-backend zero-cost-dispatch contract
//!    `D-PHASE-3-1` ratified.
//! 2. **`Self::Snapshot: Send + Sync + 'static`.** Snapshots are *owned*
//!    (no borrowing lifetime) so the engine can hold a snapshot across
//!    `.await` points + worker threads (SUBSCRIBE delivery, IVM
//!    materialization, cross-process WAIT-resume). RedbBackend pays a
//!    small clone (the `redb::ReadTransaction` is itself `'static` —
//!    backed by `Arc`-counted internals), per `arch-r1-6` recommendation
//!    (a) "make all snapshot types owned".
//! 3. **`Self::Error: std::error::Error + Send + Sync + 'static`.** Each
//!    backend surfaces its own typed error; the engine erases at the
//!    public boundary via `Box<dyn std::error::Error + Send + Sync>` per
//!    `D-PHASE-3-1a` (`arch-r1-1` BLOCKER closure pinned at G13-B).
//! 4. **`Self::Transaction` is an owned handle type.** No lifetime
//!    parameter (would force GATs + cascade into `Engine<B>`). The
//!    handle is currently a marker (the closure-based
//!    [`RedbBackend::transaction`] inherent method remains the actual
//!    execution surface for Phase-3); future waves may evolve the
//!    handle into a borrowing-runner adapter without re-breaking the
//!    trait surface.
//!
//! ## What G13-A does NOT do
//!
//! - Does NOT add `Engine<B>` cascade (that is G13-B wave-2).
//! - Does NOT add `BrowserBackend` (that is G13-C wave-3).
//! - Does NOT change inherent `RedbBackend` method shapes (callers
//!   continue to enter via `RedbBackend::transaction(|tx| ...)` etc.).
//! - Does NOT change `KVBackend` / `NodeStore` / `EdgeStore` semantics.
//!
//! G13-A is purely an *umbrella-shape extraction* canary. The R5 wave-1
//! canary discipline (single agent, observable surface, mini-review)
//! mirrors Phase-2b G12-A precedent.
//!
//! ## Usage example (post-G13-B, illustrative only — not yet on main)
//!
//! ```ignore
//! use benten_graph::{GraphBackend, RedbBackend};
//!
//! fn open_engine<B: GraphBackend>(backend: B) -> Engine<B> {
//!     // Engine accepts any backend that satisfies the umbrella surface.
//!     Engine::new(backend)
//! }
//!
//! let backend = RedbBackend::open_or_create("./data.redb").unwrap();
//! let engine = open_engine(backend);
//! ```

use std::sync::Arc;

use benten_core::Cid;

use crate::backend::KVBackend;
use crate::store::{ChangeSubscriber, EdgeStore, NodeStore};
use crate::{GraphError, Node, RedbBackend, SnapshotHandle, WriteContext};

/// Marker handle returned by [`GraphBackend::transaction`].
///
/// G13-A introduces this as an owned, lifetime-free handle so the
/// associated `type Transaction` on [`GraphBackend`] satisfies the
/// non-GAT shape required by the generic-cascade contract
/// (`arch-r1-6` recommendation (a) for `Self::Snapshot` applies
/// equally to `Self::Transaction`).
///
/// The closure-based [`RedbBackend::transaction`] inherent method
/// remains the actual transaction-execution surface for Phase 3;
/// `RedbTransactionRunner` is the umbrella-trait shape lock that
/// G13-B + later waves can evolve into a borrowing runner without
/// re-breaking the trait surface.
///
/// G13-A SHIPS THIS AS A UNIT MARKER. Future evolution (Phase-4+) may
/// turn it into an owned `Arc<RedbBackend>`-bearing struct exposing a
/// `run<F, R>(self, f: F) -> Result<R, B::Error>` method that
/// delegates to the inherent closure-based path.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RedbTransactionRunner;

impl RedbTransactionRunner {
    /// Construct a fresh runner handle.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

/// Umbrella graph-backend trait composing [`KVBackend`], [`NodeStore`],
/// [`EdgeStore`], plus the snapshot / subscriber / transaction /
/// put-with-context surface every production-grade backend must expose.
///
/// See the [module docstring](self) for the load-bearing design
/// constraints (non-object-safety, `Self::Snapshot: Send + Sync +
/// 'static`, `Self::Error` shape, `Self::Transaction` shape).
///
/// ## Sub-trait composition
///
/// `GraphBackend: KVBackend + NodeStore + EdgeStore` means every impl
/// site already supplies the byte-level KV API, node CRUD + label-only
/// fast path, and edge CRUD + index-walk APIs. The umbrella adds:
///
/// - `Self::Snapshot` + [`snapshot()`](Self::snapshot) — owned MVCC
///   snapshot handle.
/// - `Self::Transaction` + [`transaction()`](Self::transaction) — owned
///   transaction-runner handle.
/// - [`register_subscriber()`](Self::register_subscriber) — change-event
///   subscriber registration; backends without a fan-out shape (in-RAM
///   browser cache) silently no-op.
/// - [`put_node_with_context()`](Self::put_node_with_context) — privileged
///   put path threading [`WriteContext`] for capability gating + Inv-13
///   matrix dispatch.
///
/// ## Object-safety contract
///
/// The associated types `Snapshot` / `Error` / `Transaction` make this
/// trait by-construction non-object-safe. The engine consumes
/// `GraphBackend` exclusively via the *generic-cascade* direction
/// (`Engine<B: GraphBackend>` parameters). Any future PR that adds
/// `dyn GraphBackend` / `Box<dyn GraphBackend>` / `Arc<dyn GraphBackend>`
/// references at the engine boundary fails the
/// `engine_does_not_reference_dyn_graph_backend_at_engine_boundary` pin
/// landed at G13-B. See [`R3-A umbrella-trait scaffold
/// docs`](crate::graph_backend) for the full design narrative.
///
/// ## Existing implementors
///
/// - [`RedbBackend`] (G13-A) — native redb-backed production storage.
/// - `BrowserBackend` (G13-C wave-3, FORTHCOMING) — in-RAM thin-client
///   cache for `wasm32-unknown-unknown`.
/// - `SnapshotBlobBackend` (G13-D wave-3, FORTHCOMING) — read-only
///   memory-mapped snapshot for the `engine_snapshot` direct-wire path.
///
/// ## Errors
///
/// All methods returning `Result<_, Self::Error>` use the per-backend
/// associated `Error` type. RedbBackend uses [`GraphError`]; future
/// backends declare their own.
pub trait GraphBackend: KVBackend + NodeStore + EdgeStore {
    /// Owned MVCC snapshot handle. Must be `Send + Sync + 'static` so
    /// the engine can hold a snapshot across `.await` points + worker
    /// threads (per `arch-r1-6`).
    type Snapshot: Send + Sync + 'static;

    /// Backend-specific error type. Bounded by the standard
    /// error-object shape so the engine boundary can erase to
    /// `Box<dyn std::error::Error + Send + Sync>` (per `D-PHASE-3-1a` /
    /// `arch-r1-1` BLOCKER closure pinned at G13-B).
    ///
    /// Note: this *intentionally* shadows the inherited
    /// [`KVBackend::Error`] / [`NodeStore::Error`] / [`EdgeStore::Error`]
    /// associated types. Implementors satisfy the umbrella by aligning
    /// all four to the same concrete type (RedbBackend uses
    /// [`GraphError`] uniformly).
    type Error: std::error::Error + Send + Sync + 'static;

    /// Owned transaction-runner handle (no lifetime parameter — see
    /// the [module docstring](self) constraint #4).
    type Transaction;

    /// Open a transaction-runner handle. Returns the per-backend handle
    /// type; the actual closure-based execution surface stays on
    /// per-backend inherent methods (e.g.
    /// [`RedbBackend::transaction`](crate::RedbBackend::transaction)) for
    /// Phase 3.
    fn transaction(&self) -> Self::Transaction;

    /// Register a [`ChangeSubscriber`] for post-commit fan-out.
    ///
    /// The transaction primitive fans change events out synchronously
    /// to every registered subscriber after a successful commit. The
    /// subscriber list is shared across all backend instances on the
    /// same handle.
    ///
    /// Backends that do not maintain a change-event channel
    /// (`BrowserBackend`'s in-RAM thin-client cache) implement this as
    /// a no-op — the trait shape is preserved so the engine can wire
    /// IVM views uniformly without conditional code paths per backend.
    fn register_subscriber(&self, subscriber: Arc<dyn ChangeSubscriber>);

    /// Open an MVCC snapshot. The returned handle observes the
    /// committed state at the call instant; concurrent writes are
    /// invisible until the handle is dropped and a fresh one is
    /// opened.
    ///
    /// `Self::Snapshot: Send + Sync + 'static` so the snapshot can be
    /// held across `.await` boundaries + worker threads.
    ///
    /// # Errors
    ///
    /// G13-A umbrella shape: this method returns the snapshot directly
    /// rather than `Result<Self::Snapshot, Self::Error>` — backend
    /// implementors handle internal failure-to-open by panicking or by
    /// surfacing a sentinel snapshot per their own contract. RedbBackend's
    /// inherent [`RedbBackend::snapshot`] retains the `Result` shape for
    /// callers that need the typed error; the trait method delegates and
    /// unwraps. This narrow tradeoff keeps `Self::Snapshot` lifetime-free
    /// without forcing every consumer site into a `?`. Future waves may
    /// revisit if the failure-rate profile changes.
    fn snapshot(&self) -> Self::Snapshot;

    /// Privileged put-node entry point threading [`WriteContext`].
    ///
    /// This is the canonical privileged write path: `WriteContext`
    /// carries the [`crate::WriteAuthority`] field that drives the Inv-13
    /// 5-row dispatch matrix (User reput → `E_INV_IMMUTABILITY`;
    /// EnginePrivileged dedup → `Ok(cid)`; SyncReplica dedup →
    /// `Ok(cid)`). System-zone label gating + per-call durability tier
    /// selection both consume the context.
    ///
    /// # Errors
    ///
    /// Returns the backend's associated `<Self as GraphBackend>::Error`
    /// on I/O failure, invariant rejection, or system-zone gate
    /// violation. The fully-qualified projection disambiguates against
    /// the inherited `KVBackend::Error` / `NodeStore::Error` /
    /// `EdgeStore::Error` associated types — implementors satisfy the
    /// umbrella by aligning all four to the same concrete type.
    fn put_node_with_context(
        &self,
        node: &Node,
        ctx: &WriteContext,
    ) -> Result<Cid, <Self as GraphBackend>::Error>;
}

// ---------------------------------------------------------------------------
// Implementations
// ---------------------------------------------------------------------------

/// G13-A: [`RedbBackend`] satisfies the [`GraphBackend`] umbrella by
/// pure delegation to its existing inherent methods + the previously-
/// shipped [`KVBackend`] / [`NodeStore`] / [`EdgeStore`] impls.
///
/// Per `arch-r1-6`, [`SnapshotHandle`] is already lifetime-free
/// (it owns an `Option<redb::ReadTransaction>` whose internals are
/// `Arc`-counted), so satisfying `Self::Snapshot: Send + Sync +
/// 'static` requires no shape change here — the existing snapshot
/// type already qualifies.
impl GraphBackend for RedbBackend {
    type Snapshot = SnapshotHandle;
    type Error = GraphError;
    type Transaction = RedbTransactionRunner;

    /// Returns a marker [`RedbTransactionRunner`]. Actual closure-based
    /// transaction execution is on the inherent
    /// [`RedbBackend::transaction`] method; the umbrella shape is the
    /// shape lock that lets future waves evolve the runner without
    /// re-breaking `GraphBackend`.
    fn transaction(&self) -> Self::Transaction {
        RedbTransactionRunner::new()
    }

    /// Pure delegation to the inherent
    /// [`RedbBackend::register_subscriber`]. The inherent method
    /// returns `Result<(), GraphError>` for forward-compat with
    /// Phase-3 WASM peer-fetch backends; the umbrella surface is
    /// `()` because Phase-3 production callers never observe the
    /// failure (RedbBackend's impl is infallible). On the
    /// (unreachable) error path we drop the failure silently — the
    /// inherent method is still available for callers that need
    /// the typed error.
    fn register_subscriber(&self, subscriber: Arc<dyn ChangeSubscriber>) {
        let _ = RedbBackend::register_subscriber(self, subscriber);
    }

    /// Pure delegation. Inherent [`RedbBackend::snapshot`] returns
    /// `Result<SnapshotHandle, GraphError>`; the umbrella shape
    /// returns `Self::Snapshot` directly. Per the trait docstring,
    /// internal failure-to-open is unreachable on a healthy redb
    /// handle (the `db.begin_read()` only fails on severe corruption);
    /// we surface that via `expect` rather than threading `Result`
    /// through every snapshot consumer site. Callers that need the
    /// typed error continue to use the inherent method.
    fn snapshot(&self) -> Self::Snapshot {
        RedbBackend::snapshot(self).expect(
            "RedbBackend::snapshot failed at GraphBackend trait boundary — \
             redb handle is severely corrupt; use the inherent \
             RedbBackend::snapshot for a typed Result",
        )
    }

    /// Pure delegation to the inherent
    /// [`RedbBackend::put_node_with_context`].
    fn put_node_with_context(
        &self,
        node: &Node,
        ctx: &WriteContext,
    ) -> Result<Cid, <Self as GraphBackend>::Error> {
        RedbBackend::put_node_with_context(self, node, ctx)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests and benches may use unwrap/expect per workspace policy"
)]
mod tests {
    use super::*;

    /// Compile-time witness: every required method + associated type
    /// is callable through the trait surface. Mirrors the test pin in
    /// `tests/graph_backend_trait.rs::graph_backend_trait_surface_complete`.
    #[allow(dead_code)]
    fn assert_trait_surface_complete<B: GraphBackend>() {
        let _: fn(&B) -> <B as GraphBackend>::Transaction = B::transaction;
        let _: fn(&B) -> <B as GraphBackend>::Snapshot = B::snapshot;
        let _: fn(&B, &Node, &WriteContext) -> Result<Cid, <B as GraphBackend>::Error> =
            <B as GraphBackend>::put_node_with_context;
    }

    /// Compile-time witness: `Self::Snapshot: Send + Sync + 'static`
    /// (per `arch-r1-6`). Mirrors the test pin in
    /// `tests/graph_backend_trait.rs::graph_backend_snapshot_send_sync_static_for_all_backends`.
    #[allow(dead_code)]
    fn assert_snapshot_send_sync_static<B: GraphBackend>() {
        fn assert_send_sync_static<T: Send + Sync + 'static + ?Sized>() {}
        assert_send_sync_static::<<B as GraphBackend>::Snapshot>();
    }

    /// Smoke: RedbBackend satisfies the umbrella + can be used as a
    /// generic-cascade bound (the load-bearing direction per
    /// `D-PHASE-3-1` RESOLVED).
    #[test]
    fn redb_backend_impls_graph_backend_smoke() {
        fn install_via_generic_cascade<B: GraphBackend>(backend: &B) -> bool {
            let _txn = backend.transaction();
            let _snap = backend.snapshot();
            true
        }

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("smoke.redb");
        let backend = RedbBackend::open_or_create(&path).unwrap();
        assert!(install_via_generic_cascade(&backend));

        // Pin the trait-bound assertions at compile time.
        assert_trait_surface_complete::<RedbBackend>();
        assert_snapshot_send_sync_static::<RedbBackend>();
    }
}
