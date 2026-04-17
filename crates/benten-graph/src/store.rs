//! Higher-level node / edge CRUD traits: [`NodeStore`] / [`EdgeStore`], plus
//! the [`ChangeSubscriber`] trait and [`ChangeEvent`] schema.
//!
//! R1 triage deliverable **`P1.graph.node-store-trait`**: node and edge CRUD
//! are expressed as traits that each backend implements directly. Phase 1
//! ships a single backend — [`RedbBackend`](crate::RedbBackend) — whose impl
//! lives in [`redb_backend`](crate::redb_backend) and is the one place that
//! maintains the label + property-value indexes atomically with the node body.
//!
//! ## Why no blanket `impl<T: KVBackend>`
//!
//! An earlier G2-A pass carried a blanket `impl<T: KVBackend> NodeStore for T`
//! that wrote only the `n:CID` body (no index maintenance). Rust method
//! resolution preferred the inherent `RedbBackend::put_node` so the common
//! case was safe — but generic dispatch (`fn f<T: NodeStore>(…)`) and explicit
//! trait paths (`<RedbBackend as NodeStore>::put_node`) silently bypassed the
//! index maintenance. The mini-review (g2-cr-1) elevated the footgun to
//! major and the fix (this file) drops the blanket. Each backend now opts
//! into `NodeStore` / `EdgeStore` explicitly, and the index contract is
//! visible at the impl site.
//!
//! A future in-memory mock backend that needs the node API without indexes
//! writes a direct `impl NodeStore for MemBackend`; that is a deliberate
//! per-backend decision, not a silent inheritance.
//!
//! Change-stream plumbing: [`ChangeSubscriber`] declares the trait shape here
//! in `benten-graph`; the concrete broadcast channel and the actual emission
//! on commit land in `benten-engine::change` per the R1 architect decision
//! (plan §ratifications line 605 — no tokio dep in benten-graph).

use benten_core::{Cid, CoreError, Edge, Node};

// ---------------------------------------------------------------------------
// Key schema
// ---------------------------------------------------------------------------
//
// Every blanket impl routes through `KVBackend::put`/`get`/`scan` with the
// prefixes below. A dedicated key-schema module would be overkill for the
// four prefixes in play; we inline them here and document the layout.
//
// | Prefix   | What it stores                                     |
// |----------|----------------------------------------------------|
// | `n:CID`  | serialized Node keyed by its CID                   |
// | `e:CID`  | serialized Edge keyed by its CID                   |
// | `es:SRC|EDGE` | edge index: source → edge (edge CID suffix)   |
// | `et:TGT|EDGE` | edge index: target → edge (edge CID suffix)   |
//
// The edge indexes let `edges_from` / `edges_to` resolve in O(matches)
// without touching the body of every stored edge.

pub(crate) const NODE_PREFIX: &[u8] = b"n:";
pub(crate) const EDGE_PREFIX: &[u8] = b"e:";
pub(crate) const EDGE_SRC_PREFIX: &[u8] = b"es:";
pub(crate) const EDGE_TGT_PREFIX: &[u8] = b"et:";

/// `"n:" ++ cid_bytes`. Single source of truth for the Node key schema —
/// crate-private so `RedbBackend`'s inherent put/delete and the trait impl
/// can share one definition and cannot drift.
pub(crate) fn node_key(cid: &Cid) -> Vec<u8> {
    let mut k = Vec::with_capacity(NODE_PREFIX.len() + cid.as_bytes().len());
    k.extend_from_slice(NODE_PREFIX);
    k.extend_from_slice(cid.as_bytes());
    k
}

pub(crate) fn edge_key(cid: &Cid) -> Vec<u8> {
    let mut k = Vec::with_capacity(EDGE_PREFIX.len() + cid.as_bytes().len());
    k.extend_from_slice(EDGE_PREFIX);
    k.extend_from_slice(cid.as_bytes());
    k
}

pub(crate) fn edge_src_index_key(source: &Cid, edge: &Cid) -> Vec<u8> {
    let mut k =
        Vec::with_capacity(EDGE_SRC_PREFIX.len() + source.as_bytes().len() + edge.as_bytes().len());
    k.extend_from_slice(EDGE_SRC_PREFIX);
    k.extend_from_slice(source.as_bytes());
    k.extend_from_slice(edge.as_bytes());
    k
}

pub(crate) fn edge_src_index_prefix(source: &Cid) -> Vec<u8> {
    let mut k = Vec::with_capacity(EDGE_SRC_PREFIX.len() + source.as_bytes().len());
    k.extend_from_slice(EDGE_SRC_PREFIX);
    k.extend_from_slice(source.as_bytes());
    k
}

pub(crate) fn edge_tgt_index_key(target: &Cid, edge: &Cid) -> Vec<u8> {
    let mut k =
        Vec::with_capacity(EDGE_TGT_PREFIX.len() + target.as_bytes().len() + edge.as_bytes().len());
    k.extend_from_slice(EDGE_TGT_PREFIX);
    k.extend_from_slice(target.as_bytes());
    k.extend_from_slice(edge.as_bytes());
    k
}

pub(crate) fn edge_tgt_index_prefix(target: &Cid) -> Vec<u8> {
    let mut k = Vec::with_capacity(EDGE_TGT_PREFIX.len() + target.as_bytes().len());
    k.extend_from_slice(EDGE_TGT_PREFIX);
    k.extend_from_slice(target.as_bytes());
    k
}

/// Format a decode failure into a [`CoreError`]. We reuse `CoreError::Serialize`
/// rather than introducing a parallel enum variant: the DAG-CBOR decoder
/// surfaces the same class of problem (bytes ↔ typed value), and CoreError
/// already tracks the same error category for the encode direction.
pub(crate) fn decode_err<E: core::fmt::Display>(e: E) -> CoreError {
    CoreError::Serialize(format!("decode: {e}"))
}

// ---------------------------------------------------------------------------
// NodeStore
// ---------------------------------------------------------------------------

/// Node-level storage API. Each backend implements this trait directly —
/// there is no blanket `impl<T: KVBackend>` on purpose (see the module-level
/// docstring for the footgun that drove the removal).
///
/// The trait sits above `KVBackend` conceptually — it owns the Node ↔ bytes
/// DAG-CBOR transition plus the `n:`-prefix key schema — but the shared
/// per-trait blanket would silently skip the label/property index
/// maintenance that the production `RedbBackend` guarantees.
pub trait NodeStore {
    /// Error type. In practice equal to the underlying
    /// `KVBackend::Error` for the concrete backend.
    type Error;

    /// Store a Node under its CID. Returns the CID for caller convenience.
    ///
    /// # Errors
    /// Returns the backend's error on I/O failure, or a serialization error
    /// (routed through [`CoreError::Serialize`]) if the Node cannot be DAG-CBOR
    /// encoded.
    fn put_node(&self, node: &Node) -> Result<Cid, Self::Error>;

    /// Retrieve a Node by CID. Returns `Ok(None)` on a clean miss.
    ///
    /// # Errors
    /// Returns the backend's error on I/O failure, or a deserialization
    /// error (routed through [`CoreError::Serialize`]) if the stored bytes
    /// cannot be parsed as a Node.
    fn get_node(&self, cid: &Cid) -> Result<Option<Node>, Self::Error>;

    /// Delete a Node by CID. Idempotent — deleting an absent CID is not an
    /// error.
    ///
    /// # Errors
    /// Returns the backend's error on I/O failure.
    fn delete_node(&self, cid: &Cid) -> Result<(), Self::Error>;
}

// ---------------------------------------------------------------------------
// EdgeStore
// ---------------------------------------------------------------------------

/// Edge-level storage API. Each backend implements this trait directly —
/// no blanket impl, for the same reason `NodeStore` doesn't have one.
///
/// Edges are content-addressed over `(source, target, label, properties)`;
/// the `RedbBackend` impl also writes the two index keys (`es:SRC|EDGE` and
/// `et:TGT|EDGE`) so `edges_from` and `edges_to` resolve in O(matches).
pub trait EdgeStore {
    /// Error type — equal to the underlying `KVBackend::Error`.
    type Error;

    /// Store an Edge and its source/target indexes. Returns the Edge CID.
    ///
    /// # Errors
    /// Returns the backend's error on I/O failure, or a serialization error
    /// if the Edge cannot be DAG-CBOR encoded.
    fn put_edge(&self, edge: &Edge) -> Result<Cid, Self::Error>;

    /// Retrieve an Edge by CID. Returns `Ok(None)` on a clean miss.
    ///
    /// # Errors
    /// Returns the backend's error on I/O failure, or a deserialization
    /// error if the stored bytes cannot be parsed as an Edge.
    fn get_edge(&self, cid: &Cid) -> Result<Option<Edge>, Self::Error>;

    /// Delete an Edge and its source/target indexes. Idempotent — deleting an
    /// absent edge is not an error.
    ///
    /// # Errors
    /// Returns the backend's error on I/O failure.
    fn delete_edge(&self, cid: &Cid) -> Result<(), Self::Error>;

    /// All edges whose `source == cid`. Resolves via the `es:` prefix index.
    ///
    /// # Errors
    /// Returns the backend's error on I/O failure or a decode error on any
    /// index entry whose body fails to parse.
    fn edges_from(&self, source: &Cid) -> Result<Vec<Edge>, Self::Error>;

    /// All edges whose `target == cid`. Resolves via the `et:` prefix index.
    ///
    /// # Errors
    /// Returns the backend's error on I/O failure or a decode error on any
    /// index entry whose body fails to parse.
    fn edges_to(&self, target: &Cid) -> Result<Vec<Edge>, Self::Error>;
}

// ---------------------------------------------------------------------------
// Change stream (trait shape only — emission is G3)
// ---------------------------------------------------------------------------

/// Category of change emitted on the change stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    /// Node created (or re-materialized at this CID after a delete).
    Created,
    /// Node updated in place — only meaningful for non-content-addressed
    /// identities (anchors, named roots). A content-addressed Node that
    /// "changes" always surfaces as a new `Created` event at the new CID.
    Updated,
    /// Node deleted.
    Deleted,
}

/// A post-commit change event. Emitted for every graph write once the redb
/// commit completes (G3 wires the actual emission). Consumed by IVM
/// subscribers (benten-ivm subscribes to this stream to maintain views).
///
/// Attribution fields (`actor_cid`, `handler_cid`, `capability_grant_cid`)
/// are `Option` because the ingest path may or may not know them — an
/// engine-API write fills them in, a bare `put_node` via the backend leaves
/// them unset.
///
/// # Field shape
///
/// Phase 1 uses `label: String` (single label) rather than `labels: Vec<String>`
/// because the IVM views and CDC consumers index on a single primary label;
/// multi-label nodes still emit one `ChangeEvent` per event with the primary
/// label filled in. R5 reconsiders for Phase 2 if the view surface demands it.
///
/// **Asymmetry note:** the on-disk label index (`LABEL_INDEX_TABLE`) emits
/// one entry per label on multi-label nodes, while this event field carries
/// only the primary label (`labels[0]`). IVM views rebuilt from the change
/// stream observe only the primary-label path; views that need non-primary
/// labels must rebuild from the label index directly. This is an accepted
/// Phase 1 gap; the hand-written IVM views do not use multi-label nodes.
#[derive(Debug, Clone)]
pub struct ChangeEvent {
    /// CID of the Node the event concerns.
    pub cid: Cid,
    /// Primary label of the affected Node, in its display form.
    pub label: String,
    /// What kind of change happened.
    pub kind: ChangeKind,
    /// Monotonically increasing transaction id (assigned by the engine at
    /// commit time). Consumers use this to reason about before/after
    /// ordering without having to reach for wall-clock timestamps.
    pub tx_id: u64,
    /// Optional actor attribution — the Node CID of the actor who performed
    /// the write, if the write came through an attributed engine path.
    pub actor_cid: Option<Cid>,
    /// Optional handler attribution — the handler subgraph CID that issued
    /// the write, if any.
    pub handler_cid: Option<Cid>,
    /// Optional capability-grant attribution — the grant CID authorizing
    /// the write, if any.
    pub capability_grant_cid: Option<Cid>,
}

impl ChangeEvent {
    /// Stable string form of the event kind, used by integration tests and
    /// debug tooling to render change streams in human-readable form.
    #[must_use]
    pub fn kind_str(&self) -> &'static str {
        match self.kind {
            ChangeKind::Created => "Created",
            ChangeKind::Updated => "Updated",
            ChangeKind::Deleted => "Deleted",
        }
    }
}

/// Abstract subscriber for change events. Decouples `benten-graph` from any
/// specific async runtime (R1 architect major #1: no `tokio` dep in the
/// graph crate).
///
/// Implementers receive events synchronously; if they need async dispatch
/// they're free to enqueue onto their own channel. Must be `Send + Sync` so
/// the engine can share subscribers across the commit thread and the IVM
/// worker thread without further wrapping.
///
/// The trait is object-safe — subscribers are typically stored as
/// `Box<dyn ChangeSubscriber>` inside the engine so heterogeneous IVM views
/// can coexist.
pub trait ChangeSubscriber: Send + Sync {
    /// Called once per committed change event. Must not panic (panics abort
    /// the engine's commit thread). Must not block indefinitely — long work
    /// belongs on the subscriber's own worker.
    fn on_change(&self, event: &ChangeEvent);
}
