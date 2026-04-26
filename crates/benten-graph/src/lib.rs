//! # benten-graph
//!
//! Storage layer for the Benten graph engine. Defines the [`KVBackend`] trait
//! and a [`RedbBackend`] implementation over [`redb`] v4.
//!
//! The trait boundary is deliberate: a future WASM target will fetch content-
//! addressed bytes from peers (via `iroh` or HTTP) with an in-memory cache,
//! and the evaluator should not notice the difference. Defining `KVBackend`
//! in Phase 1 preserves that option.
//!
//! ## Module layout
//!
//! - [`backend`] — the [`KVBackend`] trait, [`ScanResult`], [`BatchOp`],
//!   [`DurabilityMode`].
//! - [`store`] — [`NodeStore`] / [`EdgeStore`] traits plus the
//!   [`ChangeSubscriber`] trait and [`ChangeEvent`] schema. Each backend
//!   implements `NodeStore` / `EdgeStore` directly (no blanket impl — the
//!   index-maintenance contract is per-backend).
//! - [`redb_backend`] — the concrete [`RedbBackend`], its `KVBackend` /
//!   `NodeStore` / `EdgeStore` impls, and the index maintenance.
//! - this module — [`GraphError`] and the Phase-1 stubs (`Transaction`,
//!   `WriteContext`, `SnapshotHandle`) owned by G3 / G6.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(clippy::todo, reason = "Phase 1 stubs cleared as G2-B/G3/G5/G6 land")]

use benten_core::{Cid, CoreError, Node};
pub use benten_errors::ErrorCode;

use crate::store::subgraph_key;

pub mod backend;
pub mod immutability;
pub(crate) mod indexes;
pub mod mutex_ext;
pub mod redb_backend;
pub mod store;
pub mod transaction;

pub use backend::{BatchOp, DurabilityMode, KVBackend, ScanIter, ScanResult};
pub use mutex_ext::{MutexExt, RwLockExt};
pub use redb_backend::RedbBackend;
pub use store::{ChangeEvent, ChangeKind, ChangeSubscriber, EdgeStore, NodeStore};
pub use transaction::{PendingOp, Transaction};

/// Phase 2a G2-A / C5 stub: additional test-only and Phase-2a methods on
/// [`RedbBackend`]. These are split into a trait impl here so the main
/// `redb_backend.rs` stays focused on Phase-1 semantics while Phase-2a
/// stubs surface the new API shape in one place.
///
/// TODO(phase-2b-redb-stubs): implement real bodies; the stubs below
/// `todo!()` so tests fail at runtime with a clear pointer to the owning
/// group. Retag from `phase-2a-G2-A / G5-A` after R6FP-R3 architect A11
/// — both Phase-2a groups closed without picking up these stubs; ownership
/// migrates to Phase-2b alongside the broader benten-graph storage rewrite.
impl RedbBackend {
    /// Phase 2a G2-A: `create`-alias for the `open_or_create` constructor —
    /// new R3 tests prefer this name.
    ///
    /// # Errors
    /// Returns [`GraphError`] on redb open failure.
    pub fn create(path: impl AsRef<std::path::Path>) -> Result<Self, GraphError> {
        Self::open_or_create(path)
    }

    /// Phase 2a G5-B-i: label-only read for the Inv-11 runtime probe
    /// (Code-as-graph Major #1).
    ///
    /// Used by the Inv-11 runtime hook in
    /// `benten-engine/src/primitive_host.rs` so a TRANSFORM-computed CID
    /// whose resolved Node carries a `system:*` label cannot flank the
    /// registration-time walker (plan §9.10 + Code-as-graph Major #1).
    ///
    /// # Phase-2a implementation shape
    ///
    /// The impl is **full-decode-then-drop**: read the `n:CID` redb key,
    /// DAG-CBOR-decode the bytes into a `benten_core::Node`, then keep
    /// only the first label. The name "label-only" describes the return
    /// shape, not a byte-bounded header read. The `<1µs` gate
    /// (`get_node_label_only_sub_1us` criterion bench) enforces the
    /// Phase-2a target against this full-decode impl; a truly partial
    /// decoder (stop at the `labels` field in the CBOR stream) is a
    /// Phase-2b perf refinement — the public signature is stable either
    /// way.
    ///
    /// # Errors
    /// Returns [`GraphError::Core`] (carrying a `CoreError::Serialize`)
    /// on decode failure, or [`GraphError::Redb`] on redb I/O failure.
    pub fn get_node_label_only(&self, cid: &Cid) -> Result<Option<String>, GraphError> {
        use crate::store::{decode_err, node_key};
        let Some(bytes) = self.get(&node_key(cid))? else {
            return Ok(None);
        };
        let node: benten_core::Node = serde_ipld_dagcbor::from_slice(&bytes)
            .map_err(decode_err)
            .map_err(GraphError::from)?;
        Ok(node.labels.into_iter().next())
    }

    /// Phase 2a G5-A test-only hook: inject a [`Node`] under a caller-
    /// specified `cid`, bypassing the content-addressing invariant so the
    /// Inv-13 5-row matrix can synthesise the otherwise-vacuous
    /// `User x content-differs` row (plan §9.11 row 2).
    ///
    /// Under real content-addressed storage the CID *is* the content, so a
    /// mismatched pair is unreachable from user code — this hook exists
    /// solely to exercise the row-2 error path so Inv-13's matrix stays
    /// testable end-to-end.
    ///
    /// Semantics:
    ///
    /// - [`WriteAuthority::User`] + `cid` already persisted -> fires
    ///   [`GraphError::InvImmutability`] (row 2). The mismatched bytes are
    ///   not written.
    /// - [`WriteAuthority::User`] + `cid` absent -> injects the bytes at
    ///   the requested key and warms the bloom cache for `cid`.
    /// - Any non-`User` authority -> rejected with [`GraphError::Redb`]
    ///   guarding the hook against accidental misuse from privileged paths.
    ///
    /// G11-A Wave 2a: cfg-gated behind `any(test, feature = "testing")`.
    /// This hook bypasses the content-addressing invariant by design (to
    /// synthesise the otherwise-vacuous Inv-13 row-2 `User x content-
    /// differs` path); leaving it exposed in release builds would let any
    /// caller flank the Inv-13 5-row dispatch matrix.
    ///
    /// # Errors
    /// Returns [`GraphError`] on write failure or non-User authority misuse.
    #[cfg(any(test, feature = "testing"))]
    pub fn put_node_at_cid_for_test(
        &self,
        cid: &Cid,
        node: &benten_core::Node,
        ctx: &WriteContext,
    ) -> Result<Cid, GraphError> {
        self.put_node_at_cid_for_test_impl(cid, node, ctx)
    }

    /// Phase 2a test-only hook: drain the ChangeEvent buffer for
    /// `inv_13_dedup_does_not_emit_changeevent` assertions. G5-A populates
    /// the write side; G2-A leaves an empty-drain shim so the method
    /// surface exists for the Inv-13 matrix tests to compile against.
    ///
    /// Wave-1 mini-review MODERATE-3: cfg-gated behind `any(test,
    /// feature = "testing")`. Production builds strip the buffer and
    /// the public accessor together.
    #[cfg(any(test, feature = "testing"))]
    pub fn drain_change_events_for_test(&self) -> Vec<ChangeEvent> {
        self.drain_change_events_for_test_impl()
    }

    /// Phase 2a test-only hook: whether the bloom-filter cache has warmed
    /// for this CID. Backed by [`immutability::CidExistenceCache::warmed_for`]
    /// — authoritative (records real inserts), not subject to bloom false
    /// positives.
    ///
    /// Wave-1 mini-review MODERATE-3: cfg-gated behind `any(test,
    /// feature = "testing")`.
    #[cfg(any(test, feature = "testing"))]
    pub fn cache_contains_cid(&self, cid: &Cid) -> bool {
        self.cache_contains_cid_impl(cid)
    }

    /// Phase 2a test-only hook: force the next `put_node`'s bloom probe to
    /// report `true` unconditionally (one-shot), so tests can exercise the
    /// false-positive fallback path reliably.
    ///
    /// G11-A Wave 2a: cfg-gated behind `any(test, feature = "testing")`.
    #[cfg(any(test, feature = "testing"))]
    pub fn force_bloom_collision_for_next_put(&self) {
        self.force_bloom_collision_for_next_put_impl();
    }

    /// Phase 2a test-only hook: non-mutating probe of the bloom filter.
    ///
    /// G11-A Wave 2a: cfg-gated behind `any(test, feature = "testing")`.
    #[cfg(any(test, feature = "testing"))]
    pub fn bloom_may_contain_for_test(&self, cid: &Cid) -> bool {
        self.bloom_may_contain_for_test_impl(cid)
    }

    /// Phase 2a test-only hook: force the bloom filter to report positive
    /// for `cid` until the backend is dropped. Persistent (not one-shot —
    /// contrast with [`Self::force_bloom_collision_for_next_put`]).
    ///
    /// G11-A Wave 2a: cfg-gated behind `any(test, feature = "testing")`.
    #[cfg(any(test, feature = "testing"))]
    pub fn force_bloom_positive_for_test(&self, cid: &Cid) {
        self.force_bloom_positive_for_test_impl(cid);
    }

    /// Phase 2a arch-r1-1 descope-witness bench helper. The accompanying
    /// bench (`crud_post_create_dispatch_group_durability.rs`) routes its
    /// iteration body through this helper so the bench compiles today.
    ///
    /// TODO(phase-2b-benchmark-durability-wiring): wire durability-mode
    /// pass-through through `put_node` so the Group vs Immediate delta is
    /// observable. Retag from `phase-2a-G2-A` after R6FP-R3 architect A11.
    pub fn benchmark_helper_crud_post_create_dispatch(&self, _durability: DurabilityMode) {
        todo!(
            "Phase 2a G2-A descope-witness: implement durability-mode pass-through \
             per arch-r1-1 + named Compromise #N+3"
        )
    }

    /// Phase 2a test-only hook: return the `DurabilityMode` of the last
    /// `put_node` that targeted a Node carrying `label`. Populated by
    /// [`RedbBackend::put_node_with_context`] after every successful commit
    /// so the `capability_grant_writes_immediate` test can assert that the
    /// privileged path overrode the configured durability.
    ///
    /// Wave-1 mini-review MODERATE-3: cfg-gated behind `any(test,
    /// feature = "testing")`.
    #[cfg(any(test, feature = "testing"))]
    pub fn last_put_node_durability_for_label(&self, label: &str) -> Option<DurabilityMode> {
        self.last_put_node_durability_for_label_impl(label)
    }

    /// Phase 2a test-only hook: reset the bytes-read counter for the
    /// `get_node_label_only_fast_path_reads_prefix_only` assertion.
    ///
    /// G11-A Wave 3a CFG-GATING M2: stub is gated behind `any(test,
    /// feature = "testing")` so the no-op cannot leak into release
    /// builds. Body remains a no-op — the byte-counter instrumentation
    /// is a Phase-2b perf-instrumentation refinement.
    #[cfg(any(test, feature = "testing"))]
    pub fn reset_read_byte_counter(&self) {}

    /// Phase 2a test-only hook: read bytes consumed since the last reset.
    ///
    /// G11-A Wave 3a CFG-GATING M2: stub is gated behind `any(test,
    /// feature = "testing")`.
    #[cfg(any(test, feature = "testing"))]
    pub fn read_bytes_since_reset(&self) -> usize {
        0
    }

    /// G12-C: store a `Subgraph` under its DAG-CBOR canonical
    /// encoding, returning its CID.
    ///
    /// The subgraph is keyed by the `s:` prefix plus the CID bytes, parallel
    /// to the `n:CID` / `e:CID` schema for Nodes / Edges. The CID itself is
    /// BLAKE3 over the canonical DAG-CBOR bytes so a round-trip through
    /// [`RedbBackend::load_subgraph_verified`] recomputes the identical CID.
    ///
    /// # Errors
    /// Returns [`GraphError`] on encode / write failure.
    pub fn store_subgraph(&self, sg: &benten_core::Subgraph) -> Result<Cid, GraphError> {
        let bytes = sg.to_dag_cbor().map_err(GraphError::from)?;
        let digest = blake3::hash(&bytes);
        let cid = Cid::from_blake3_digest(*digest.as_bytes());
        self.put(&subgraph_key(&cid), &bytes)?;
        Ok(cid)
    }

    /// G12-C: load a subgraph by CID, verifying integrity.
    ///
    /// Hash-first: the stored bytes are BLAKE3-hashed and compared against
    /// the caller-supplied `cid` before any decode is attempted. Mismatch
    /// fires [`CoreError::ContentHashMismatch`] (mapped to
    /// `E_INV_CONTENT_HASH` via [`GraphError::code`]); a matching hash that
    /// still fails to decode surfaces as [`CoreError::Serialize`] (mapped
    /// to `E_SERIALIZE`). A missing CID returns `Ok(None)`.
    ///
    /// # Errors
    /// - [`GraphError::Core`] carrying `CoreError::ContentHashMismatch` on
    ///   tamper / corruption.
    /// - [`GraphError::Core`] carrying `CoreError::Serialize` on decode
    ///   failure of (hash-matching) bytes.
    /// - [`GraphError::Redb`] on redb I/O failure.
    pub fn load_subgraph_verified(
        &self,
        cid: &Cid,
    ) -> Result<Option<benten_core::Subgraph>, GraphError> {
        let Some(bytes) = self.get(&subgraph_key(cid))? else {
            return Ok(None);
        };
        let sg =
            benten_core::Subgraph::load_verified_with_cid(cid, &bytes).map_err(GraphError::from)?;
        Ok(Some(sg))
    }

    /// Phase 2a test-only hook: corrupt on-disk subgraph bytes via a
    /// mutator closure.
    ///
    /// Reads the bytes at `s:CID`, hands them to `mutate` for in-place
    /// modification, writes them back under the ORIGINAL key. The key is
    /// preserved (not re-computed from the mutated bytes) so the next
    /// `load_subgraph_verified(cid)` observes the CID-vs-content drift as
    /// `E_INV_CONTENT_HASH` rather than a clean miss.
    ///
    /// G11-A Wave 2a: cfg-gated behind `any(test, feature = "testing")`
    /// so a release build cannot reach the corruption primitive.
    ///
    /// # Errors
    /// Returns [`GraphError::Redb`] with a string payload if the CID is
    /// missing (the hook has no bytes to corrupt), or propagates I/O
    /// failures from the underlying put/get.
    #[cfg(any(test, feature = "testing"))]
    pub fn corrupt_subgraph_bytes_for_test<F>(&self, cid: &Cid, mutate: F) -> Result<(), GraphError>
    where
        F: FnOnce(&mut [u8]),
    {
        let key = subgraph_key(cid);
        let Some(mut bytes) = self.get(&key)? else {
            return Err(GraphError::Redb(format!(
                "corrupt_subgraph_bytes_for_test: CID {cid:?} not present"
            )));
        };
        mutate(&mut bytes);
        self.put(&key, &bytes)?;
        Ok(())
    }

    /// Phase 2a test-only hook: inject raw bytes under their computed CID.
    ///
    /// Hashes `bytes` to produce the CID, writes under `s:CID`, returns
    /// the CID. Unlike [`Self::corrupt_subgraph_bytes_for_test`] this path
    /// preserves the content-addressing invariant (key = hash(value)) —
    /// its purpose is to inject bytes that hash-match but fail to DECODE
    /// as a `Subgraph`, exercising the `E_SERIALIZE` arm of
    /// `load_subgraph_verified`.
    ///
    /// G11-A Wave 2a: cfg-gated behind `any(test, feature = "testing")`.
    ///
    /// # Errors
    /// Returns [`GraphError`] on write failure.
    #[cfg(any(test, feature = "testing"))]
    pub fn inject_raw_subgraph_bytes_for_test(&self, bytes: &[u8]) -> Result<Cid, GraphError> {
        let digest = blake3::hash(bytes);
        let cid = Cid::from_blake3_digest(*digest.as_bytes());
        self.put(&subgraph_key(&cid), bytes)?;
        Ok(cid)
    }
}

/// Re-export of [`benten_core::WriteAuthority`]. Phase 2a ucca-9 / arch-r1-2
/// frozen shape lives in benten-core so every mid-stack crate uses the same
/// type. See `benten-core/src/lib.rs` for docs.
pub use benten_core::WriteAuthority;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors from the storage layer.
///
/// R1 triage `P1.graph.error-polymorphism` (G2-A) moved backend errors behind
/// an associated [`KVBackend::Error`] type. `GraphError` remains the concrete
/// error for `RedbBackend` and the type into which `CoreError` (serialization,
/// CID parsing) flows; other backends are free to pick their own.
///
/// r6-err-3 added `RedbSource(#[from] redb::Error)` so the six redb sub-type
/// `From` impls funnel the original error through `redb::Error` with
/// `std::error::Error::source()` preserved. The string-payload `Redb(String)`
/// variant is retained for test-fixture injection (see
/// `tests/failure_injection_rollback.rs`) and for internal
/// "missing transaction handle" bookkeeping.
/// `#[non_exhaustive]` (R6b bp-17) — Phase 2 introduces per-backend error
/// kinds (e.g. WASM `IndexedDBBackend`, peer-fetch errors); downstream
/// matchers must include `_ =>` so adding variants is a minor version bump.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum GraphError {
    /// Propagated from `benten-core` (CID construction, canonical
    /// serialization, DAG-CBOR decode via `CoreError::Serialize`).
    #[error("core: {0}")]
    Core(#[from] CoreError),

    /// redb I/O or transactional failure with the original `redb::Error`
    /// preserved behind `#[source]` so `std::error::Error::source()` walks
    /// the chain. The six redb sub-error types (`DatabaseError`,
    /// `TransactionError`, `TableError`, `StorageError`, `CommitError`)
    /// each have a native `From<X> for redb::Error`; our `From` impls
    /// funnel through that so the origin kind is preserved.
    #[error("redb: {0}")]
    RedbSource(#[from] redb::Error),

    /// redb I/O or transactional failure, string-payload form. Retained
    /// for test-fixture injection (e.g.
    /// `GraphError::Redb("injected failure".into())`) and for internal
    /// "post-commit handle missing" bookkeeping inside the transaction
    /// primitive. Production conversion sites should use [`GraphError::RedbSource`]
    /// instead so the `std::error::Error::source` chain is preserved.
    #[error("redb: {0}")]
    Redb(String),

    /// DAG-CBOR decode of a stored Node failed. Indicates on-disk corruption
    /// or a format drift. The [`NodeStore`] / [`EdgeStore`] blanket impls
    /// route decode errors through [`CoreError::Serialize`] → `Core` instead;
    /// this variant is retained for any direct-decode call path (notably the
    /// retained inherent `RedbBackend::get_node` helper).
    #[error("decode: {0}")]
    Decode(String),

    /// `open_existing` was called on a path where no database file exists.
    ///
    /// The Display form shows only the basename (e.g. `benten.redb`) so the
    /// rendered message — which flows through napi into JS `Error.message`
    /// — does not leak the absolute filesystem path (r6-err-7: avoids
    /// leaking the caller's home-directory / username). The full `PathBuf`
    /// remains on the struct field for programmatic introspection and
    /// Debug rendering.
    #[error("backend not found: {}", redact_path_for_display(path))]
    BackendNotFound {
        /// Path supplied to the failed `open_existing` call.
        path: std::path::PathBuf,
    },

    /// A write was attempted on a system-zone label (label starting with
    /// `"system:"`) without the privileged flag set. Phase 1 SC1 stopgap.
    #[error("system-zone write not permitted from user path: {label}")]
    SystemZoneWrite {
        /// The `system:` label the user-zone path tried to write.
        label: String,
    },

    /// A nested transaction was rejected. Phase 1 G3 stub.
    #[error("nested transactions are not supported")]
    NestedTransactionNotSupported {},

    /// Invariant 13 (immutability) rejection — an unprivileged
    /// [`WriteAuthority::User`] re-put of an already-stored CID, per plan
    /// §9.11 rows 1-2. Fires `E_INV_IMMUTABILITY`. G2-A lands the User-path
    /// firing at the storage layer; G5-A extends to cover the full 5-row
    /// matrix (EnginePrivileged dedup + SyncReplica dedup rows do NOT return
    /// this error — they dedup to `Ok(cid)`).
    ///
    /// Phase-2a R6 EH1: carries the [`WriteAuthority`] under which the
    /// re-put attempt was made so the rendered Display + downstream
    /// diagnostics can name which row of the 5-row matrix actually
    /// fired. The catalog (`docs/ERROR-CATALOG.md`) promises this
    /// field; the firing sites at `redb_backend.rs:1038, 1186` already
    /// know the authority and now thread it through.
    // R6 round-2 EH1-R2-OBS: render the CID via its `Display` impl
    // (`{cid}`) rather than `{cid:?}`. `Cid::Display` produces the
    // base32 multibase form catalogued in `docs/ERROR-CATALOG.md`,
    // whereas `{cid:?}` printed the wrapped `[u8; 32]` tuple — useful
    // for engine internals but not for operator-facing diagnostics.
    // `attempted_authority` retains `{:?}` because `WriteAuthority`
    // has no `Display` impl (it's an internal enum, not an
    // operator-renderable identifier).
    #[error(
        "immutability violation: CID {cid} already persisted (attempted_authority: {attempted_authority:?})"
    )]
    InvImmutability {
        /// The CID the re-put targeted.
        cid: Cid,
        /// Authority under which the re-put was attempted. Always the
        /// load-bearing diagnostic — only [`WriteAuthority::User`] reaches
        /// this variant in production today; G5-A test hooks
        /// (`put_node_at_cid_for_test`) constrain to `User` as well, so
        /// observing any other variant in the field signals a regression
        /// in the dispatch matrix.
        attempted_authority: WriteAuthority,
    },

    /// The transaction's closure returned `Err`, so the write batch was
    /// rolled back.
    #[error("transaction aborted: {reason}")]
    TxAborted {
        /// Human-readable reason the closure returned `Err`.
        reason: String,
    },
}

impl GraphError {
    /// Map a `GraphError` to its stable ERROR-CATALOG code.
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        match self {
            GraphError::Core(e) => e.code(),
            // `RedbSource` preserves the full `std::error::Error::source`
            // chain (r6-err-3); the catalog code is still `E_GRAPH_INTERNAL`
            // because the underlying redb error kind is opaque to
            // cross-language consumers. The string-payload `Redb` and
            // `Decode` variants carry the same catalog code for parity.
            GraphError::RedbSource(_) | GraphError::Redb(_) | GraphError::Decode(_) => {
                ErrorCode::GraphInternal
            }
            GraphError::BackendNotFound { .. } => ErrorCode::BackendNotFound,
            GraphError::SystemZoneWrite { .. } => ErrorCode::SystemZoneWrite,
            GraphError::NestedTransactionNotSupported {} => {
                ErrorCode::NestedTransactionNotSupported
            }
            GraphError::InvImmutability { .. } => ErrorCode::InvImmutability,
            GraphError::TxAborted { .. } => ErrorCode::TxAborted,
        }
    }
}

/// Render a `Path` for the Display of [`GraphError::BackendNotFound`] with
/// only its basename + a placeholder prefix so the rendered message does
/// not leak the absolute filesystem path through to user-facing error
/// strings. The full path is still available on the struct variant for
/// programmatic use and for `Debug` rendering.
fn redact_path_for_display(path: &std::path::Path) -> String {
    match path.file_name() {
        Some(name) => format!("<redacted>/{}", name.to_string_lossy()),
        None => "<redacted>".to_string(),
    }
}

// r6-err-3: preserve `std::error::Error::source()` on redb failures.
// Each redb sub-error type has a native `From<X> for redb::Error` in the
// redb crate, so we funnel through `redb::Error` and store it under
// `RedbSource` with `#[source]` preservation via `thiserror`'s `#[from]`.
impl From<redb::DatabaseError> for GraphError {
    fn from(e: redb::DatabaseError) -> Self {
        GraphError::RedbSource(e.into())
    }
}
impl From<redb::TransactionError> for GraphError {
    fn from(e: redb::TransactionError) -> Self {
        GraphError::RedbSource(e.into())
    }
}
impl From<redb::TableError> for GraphError {
    fn from(e: redb::TableError) -> Self {
        GraphError::RedbSource(e.into())
    }
}
impl From<redb::StorageError> for GraphError {
    fn from(e: redb::StorageError) -> Self {
        GraphError::RedbSource(e.into())
    }
}
impl From<redb::CommitError> for GraphError {
    fn from(e: redb::CommitError) -> Self {
        GraphError::RedbSource(e.into())
    }
}

// ---------------------------------------------------------------------------
// RedbBackend
// ---------------------------------------------------------------------------
//
// The concrete `RedbBackend` struct, its `KVBackend` impl, the three
// construction entry points (`open` / `open_existing` / `open_or_create`),
// and the label + property-value index plumbing all live in
// [`redb_backend`]. `pub use redb_backend::RedbBackend` re-exports it at
// crate root so existing call sites (and the integration tests) don't need
// to know about the module split.

// ---------------------------------------------------------------------------
// Phase 1 stubs — expanded in G3 / G6
// ---------------------------------------------------------------------------

/// A MVCC snapshot handle returned by [`RedbBackend::snapshot`]. Reads
/// through this handle observe the database state at the instant the
/// snapshot was opened; concurrent writes to the backend are invisible until
/// the handle is dropped.
///
/// G3-A lands a partial shape: [`SnapshotHandle::get_node`] is implemented
/// (thin wrapper over a `redb::ReadTransaction` held across the handle's
/// lifetime). [`SnapshotHandle::scan_label`] stays a G6 stub — it depends
/// on the label-index scan plumbing that G6 owns.
///
/// Implements `Drop` so explicit `drop(handle)` in tests is the idiomatic
/// way to release the snapshot's read-transaction lifetime.
pub struct SnapshotHandle {
    /// redb ReadTransaction captured at snapshot-open time. redb's read
    /// transactions are lightweight (no writer lock held) and observe the
    /// committed state at the instant `begin_read()` returned.
    pub(crate) read_txn: Option<redb::ReadTransaction>,
}

impl Drop for SnapshotHandle {
    fn drop(&mut self) {
        // Dropping the `ReadTransaction` releases the snapshot naturally.
        self.read_txn.take();
    }
}

impl SnapshotHandle {
    /// Retrieve a Node by CID from the snapshot view. Reads through the
    /// handle observe the point-in-time state captured when
    /// [`RedbBackend::snapshot`] was called; concurrent writes are
    /// invisible until the handle is dropped and a fresh snapshot is
    /// opened.
    ///
    /// # Errors
    /// - [`GraphError::Redb`] on any redb I/O failure.
    /// - [`GraphError::Decode`] if a stored Node fails to decode.
    pub fn get_node(&self, cid: &Cid) -> Result<Option<Node>, GraphError> {
        use redb::ReadableTable;
        let Some(read_txn) = self.read_txn.as_ref() else {
            return Ok(None);
        };
        let table = read_txn.open_table(redb_backend::NODES_TABLE)?;
        let key = store::node_key(cid);
        let Some(v) = table.get(key.as_slice())? else {
            return Ok(None);
        };
        let node: Node = serde_ipld_dagcbor::from_slice(&v.value())
            .map_err(|e| GraphError::Decode(format!("snapshot get_node decode: {e}")))?;
        Ok(Some(node))
    }

    /// Scan all nodes with a given label from the snapshot view.
    ///
    /// Uses the label-index multimap table opened against the snapshot's
    /// read-transaction so results reflect the point-in-time state captured
    /// when [`RedbBackend::snapshot`] was called; concurrent writes are
    /// invisible to this scan.
    ///
    /// # Errors
    ///
    /// - [`GraphError::Redb`] on any redb I/O failure.
    /// - [`GraphError::Core`] if an index entry fails to decode.
    pub fn scan_label(&self, label: &str) -> Result<Vec<Cid>, GraphError> {
        if label.is_empty() {
            return Ok(Vec::new());
        }
        let Some(read_txn) = self.read_txn.as_ref() else {
            return Ok(Vec::new());
        };
        let table = read_txn.open_multimap_table(crate::indexes::LABEL_INDEX_TABLE)?;
        let values = table.get(label.as_bytes())?;
        let mut out = Vec::new();
        for v in values {
            let v = v?;
            let cid = crate::indexes::cid_from_index_bytes(v.value())?;
            out.push(cid);
        }
        Ok(out)
    }
}

// ChangeReceiver intentionally does NOT live in benten-graph.
//
// Per the implementation plan (R1 architect addendum, line ~605), the
// channel concretion — tokio-broadcast on native, synchronous
// `Vec<Box<dyn ChangeSubscriber>>` fan-out on WASM — lives in
// `benten-engine::change`. The graph crate exposes only the
// [`ChangeSubscriber`] callback trait ([`store::ChangeSubscriber`]) so it
// carries no async-runtime dependency. Backends register subscribers via
// `RedbBackend::register_subscriber(Arc<dyn ChangeSubscriber>)`; the
// transaction primitive (G3) fans change events out to registered
// subscribers synchronously after a successful commit.

/// Metadata passed to the capability pre-write hook.
///
/// `is_privileged = true` marks an engine-API-only path (grant_capability,
/// create_view, revoke_capability), bypassing the system-zone label ban.
///
/// **Phase 1 G3-A / SC1 stub.**
#[derive(Debug, Clone)]
pub struct WriteContext {
    /// The Node's primary label — used for the system-zone prefix check.
    pub label: String,
    /// Marks an engine-API-only path. User code cannot reach this without
    /// going through one of the engine's privileged methods.
    pub is_privileged: bool,
    /// Phase 2a G2-B / ucca-9 / arch-r1-2: authority under which the write
    /// runs. Defaults to [`WriteAuthority::User`]. `EnginePrivileged` aligns
    /// with `is_privileged = true`; `SyncReplica` is Phase-3 reserved.
    pub authority: WriteAuthority,
}

impl Default for WriteContext {
    fn default() -> Self {
        Self {
            label: String::new(),
            is_privileged: false,
            authority: WriteAuthority::User,
        }
    }
}

impl WriteContext {
    /// Construct a non-privileged write context for a given label. This is
    /// the constructor user-authored code paths use.
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            is_privileged: false,
            authority: WriteAuthority::User,
        }
    }

    /// Construct a WriteContext flagged as privileged (engine-API-only
    /// path). This is the only constructor that bypasses the SC1
    /// system-zone ban. User code cannot call this without going through
    /// `Engine::grant_capability`, `Engine::create_view`, or
    /// `Engine::revoke_capability`.
    #[must_use]
    pub fn privileged_for_engine_api() -> Self {
        Self {
            label: String::new(),
            is_privileged: true,
            authority: WriteAuthority::EnginePrivileged,
        }
    }

    /// Set the [`WriteAuthority`] for this context (builder-style).
    ///
    /// TODO(phase-2b-write-authority-coherence): wire `EnginePrivileged`
    /// to also flip `is_privileged = true` at call sites, so both axes
    /// stay coherent. Retag from `phase-2a-G2-B` after R6FP-R3 architect A11.
    #[must_use]
    pub fn with_authority(mut self, authority: WriteAuthority) -> Self {
        if matches!(authority, WriteAuthority::EnginePrivileged) {
            self.is_privileged = true;
        }
        self.authority = authority;
        self
    }

    /// Called by the transaction primitive to enforce the SC1 stopgap.
    /// Rejects writes to any label starting with `"system:"` unless
    /// `is_privileged == true`. Returns the `label` string in the error so
    /// diagnostics can point at the exact reserved label the write
    /// attempted.
    ///
    /// # Errors
    /// [`GraphError::SystemZoneWrite`] on an unprivileged system-zone
    /// label.
    pub fn enforce_system_zone(&self) -> Result<(), GraphError> {
        if !self.is_privileged && self.label.starts_with("system:") {
            return Err(GraphError::SystemZoneWrite {
                label: self.label.clone(),
            });
        }
        Ok(())
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
    use benten_core::testing::canonical_test_node;

    fn temp_backend() -> (RedbBackend, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("benten.redb");
        let backend = RedbBackend::open(&path).unwrap();
        (backend, dir)
    }

    #[test]
    fn put_then_get_roundtrip() {
        let (backend, _dir) = temp_backend();
        let node = canonical_test_node();
        let cid = backend.put_node(&node).unwrap();

        let fetched = backend.get_node(&cid).unwrap().expect("node must exist");
        assert_eq!(fetched, node);

        // Re-hashing the fetched node reproduces the CID — proves end-to-end
        // content-addressing through the storage layer.
        assert_eq!(fetched.cid().unwrap(), cid);
    }

    #[test]
    fn get_missing_returns_none() {
        let (backend, _dir) = temp_backend();
        let cid = canonical_test_node().cid().unwrap();
        assert!(backend.get_node(&cid).unwrap().is_none());
    }

    #[test]
    fn delete_is_idempotent() {
        let (backend, _dir) = temp_backend();
        let node = canonical_test_node();
        let cid = backend.put_node(&node).unwrap();
        // Delete via the Node-level API (uses the `n:` key schema).
        backend.delete_node(&cid).unwrap();
        backend.delete_node(&cid).unwrap(); // second delete must not panic
        assert!(backend.get_node(&cid).unwrap().is_none());
    }

    #[test]
    fn batch_put_is_atomic() {
        let (backend, _dir) = temp_backend();
        let pairs = vec![
            (b"k1".to_vec(), b"v1".to_vec()),
            (b"k2".to_vec(), b"v2".to_vec()),
        ];
        backend.put_batch(&pairs).unwrap();
        assert_eq!(backend.get(b"k1").unwrap().as_deref(), Some(b"v1".as_ref()));
        assert_eq!(backend.get(b"k2").unwrap().as_deref(), Some(b"v2".as_ref()));
    }

    #[test]
    fn scan_empty_prefix_returns_everything() {
        let (backend, _dir) = temp_backend();
        let pairs = vec![
            (b"alpha".to_vec(), b"1".to_vec()),
            (b"beta".to_vec(), b"2".to_vec()),
            (b"gamma".to_vec(), b"3".to_vec()),
        ];
        backend.put_batch(&pairs).unwrap();

        let hits = backend.scan(&[]).unwrap();
        assert_eq!(hits.len(), 3, "empty prefix must match every key");

        // Confirm redb returns results in sorted key order so callers can
        // rely on it for deterministic downstream processing (content
        // listings, IVM bootstrap).
        let mut keys: Vec<&[u8]> = hits.iter().map(|(k, _)| k.as_slice()).collect();
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(keys, sorted);
        keys.sort();
        assert_eq!(
            keys,
            [b"alpha".as_ref(), b"beta".as_ref(), b"gamma".as_ref()]
        );
    }

    #[test]
    fn scan_zero_hit_prefix_returns_empty() {
        let (backend, _dir) = temp_backend();
        backend
            .put_batch(&[(b"alpha".to_vec(), b"1".to_vec())])
            .unwrap();

        // A prefix that sorts after every stored key (and cannot be a
        // prefix of any stored key) must return an empty result, not error.
        let hits = backend.scan(b"zzz").unwrap();
        assert!(hits.is_empty());

        // A prefix on an empty store must also return empty.
        let (empty_backend, _empty_dir) = temp_backend();
        let hits = empty_backend.scan(b"anything").unwrap();
        assert!(hits.is_empty());
    }

    #[test]
    fn scan_prefix_bounds_the_range() {
        // Regression test for the earlier O(n) implementation that iterated
        // the full table regardless of prefix.
        let (backend, _dir) = temp_backend();
        let pairs = vec![
            (b"post:1".to_vec(), b"p1".to_vec()),
            (b"post:2".to_vec(), b"p2".to_vec()),
            (b"user:1".to_vec(), b"u1".to_vec()),
            (b"user:2".to_vec(), b"u2".to_vec()),
            (b"zzz".to_vec(), b"z".to_vec()),
        ];
        backend.put_batch(&pairs).unwrap();

        let posts = backend.scan(b"post:").unwrap();
        assert_eq!(posts.len(), 2);
        assert!(posts.iter().all(|(k, _)| k.starts_with(b"post:")));

        let users = backend.scan(b"user:").unwrap();
        assert_eq!(users.len(), 2);
        assert!(users.iter().all(|(k, _)| k.starts_with(b"user:")));
    }

    #[test]
    fn scan_all_0xff_prefix_is_open_ended() {
        let (backend, _dir) = temp_backend();
        backend.put(&[0xff, 0xff, 0xff], b"sentinel").unwrap();
        backend.put(b"unrelated", b"nope").unwrap();

        let hits = backend.scan(&[0xff, 0xff]).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits.as_slice()[0].0, vec![0xff, 0xff, 0xff]);
    }

    #[test]
    fn batch_put_empty_slice_is_a_noop() {
        let (backend, _dir) = temp_backend();
        backend.put_batch(&[]).unwrap();
        assert!(backend.scan(&[]).unwrap().is_empty());
    }

    // `next_prefix_increments_and_trims` — moved to `redb_backend.rs` in G2-B
    // alongside the helper it exercises.
}
