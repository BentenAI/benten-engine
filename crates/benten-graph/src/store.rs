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
// | `s:CID`  | DAG-CBOR Subgraph body keyed by its CID            |
// | `es:SRC|EDGE` | edge index: source → edge (edge CID suffix)   |
// | `et:TGT|EDGE` | edge index: target → edge (edge CID suffix)   |
//
// The edge indexes let `edges_from` / `edges_to` resolve in O(matches)
// without touching the body of every stored edge.

pub(crate) const NODE_PREFIX: &[u8] = b"n:";
pub(crate) const EDGE_PREFIX: &[u8] = b"e:";
pub(crate) const EDGE_SRC_PREFIX: &[u8] = b"es:";
pub(crate) const EDGE_TGT_PREFIX: &[u8] = b"et:";
pub(crate) const SUBGRAPH_PREFIX: &[u8] = b"s:";

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

/// `"s:" ++ cid_bytes`. Subgraph key schema — parallels the `n:` / `e:`
/// Node / Edge layout. Crate-private so the inherent `RedbBackend`
/// subgraph put/get and any future per-backend impl share one definition
/// and cannot drift.
pub(crate) fn subgraph_key(cid: &Cid) -> Vec<u8> {
    let mut k = Vec::with_capacity(SUBGRAPH_PREFIX.len() + cid.as_bytes().len());
    k.extend_from_slice(SUBGRAPH_PREFIX);
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

    /// Phase 2a G5-B-i / Code-as-graph Major #1: probe that returns the
    /// stored Node's first label so the Inv-11 runtime hook can classify
    /// resolved CIDs against the system-zone prefix set.
    ///
    /// The Inv-11 runtime hook in `benten-engine/src/primitive_host.rs`
    /// calls this on every TRANSFORM-computed READ / WRITE target so a
    /// user subgraph that routes through a computed `Value::Cid` cannot
    /// flank the registration-time literal-CID walker.
    ///
    /// # Phase-2a implementation shape
    ///
    /// Both the default [`NodeStore`] impl and the concrete
    /// [`crate::RedbBackend::get_node_label_only`] override currently
    /// perform a **full-Node DAG-CBOR decode and drop everything but the
    /// first label**. The name "label-only" refers to the *return shape*,
    /// not to a byte-bounded header read. A truly partial decoder that
    /// stops at the `labels` field is a Phase-2b perf refinement; the
    /// public signature is stable either way and the
    /// `get_node_label_only_sub_1us` criterion bench enforces the <1 µs
    /// gate against the full-decode impl today.
    ///
    /// # Errors
    /// Returns the backend's error on I/O failure or a decode error if
    /// the stored bytes cannot be parsed as a Node.
    fn get_node_label_only(&self, cid: &Cid) -> Result<Option<String>, Self::Error> {
        match self.get_node(cid)? {
            Some(node) => Ok(node.labels.into_iter().next()),
            None => Ok(None),
        }
    }
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
///
/// Node events use [`ChangeKind::Created`] / [`ChangeKind::Updated`] /
/// [`ChangeKind::Deleted`]. Edge events use the explicit
/// [`ChangeKind::EdgeCreated`] / [`ChangeKind::EdgeDeleted`] variants so
/// subscribers can route without having to inspect `edge_endpoints` and so
/// IVM views driven off edge ingress (governance inheritance, version
/// current) can fire directly on the trait path rather than degenerate to
/// identity-only acknowledgements.
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
    /// Edge created. Consumers that care about endpoints inspect
    /// [`ChangeEvent::edge_endpoints`] for the `(source, target, label)`
    /// triple. The event's own `cid` is the edge's CID.
    EdgeCreated,
    /// Edge deleted. Endpoints were captured via read-before-delete and are
    /// carried on [`ChangeEvent::edge_endpoints`] when recoverable.
    EdgeDeleted,
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
/// The `labels` field carries the full label set for the affected Node.
/// Multi-label nodes emit a single `ChangeEvent` whose `labels` vector holds
/// every label so label-filtered subscribers (IVM views, CDC consumers) can
/// route deterministically without having to re-read the Node body after the
/// commit. Delete events also populate `labels` by reading the Node before
/// deletion — empty `labels` on a delete means the target was already absent.
///
/// Edge events populate `labels` with a single-element vector (`vec![edge.label]`)
/// so the same routing API handles both node and edge events.
#[derive(Debug, Clone)]
pub struct ChangeEvent {
    /// CID of the Node (or Edge) the event concerns.
    pub cid: Cid,
    /// Full label set of the affected Node at the moment the event was
    /// emitted. For edges, a single-element vector holding the edge's label.
    /// For a delete that targeted an already-absent CID, the vector is
    /// empty (idempotent-delete miss — no labels were recoverable).
    pub labels: Vec<String>,
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
    /// Full Node content, populated for Node events
    /// ([`ChangeKind::Created`] / [`ChangeKind::Updated`] /
    /// [`ChangeKind::Deleted`]). For `Created`/`Updated`, the Node being
    /// written; for `Deleted`, the Node captured via read-before-delete
    /// (may be `None` on an idempotent-delete miss). `None` for edge events.
    ///
    /// IVM views that need property data (e.g. `createdAt`, `grantee`,
    /// `subscribes_to`) read it from here — the widen replaces the
    /// previously-degenerate identity-only trait path described in the
    /// G5 mini-review.
    pub node: Option<Node>,
    /// Edge endpoints `(source, target, label)` populated for
    /// [`ChangeKind::EdgeCreated`] / [`ChangeKind::EdgeDeleted`]. For a
    /// create, derived from the edge being written; for a delete, captured
    /// via read-before-delete. `None` for node events or when a delete
    /// missed an already-absent edge.
    pub edge_endpoints: Option<(Cid, Cid, String)>,
}

impl ChangeEvent {
    /// Construct a Node-event [`ChangeEvent`]. Shields callers (tests,
    /// integration harnesses) from the full field list.
    ///
    /// `attribution` is `(actor_cid, handler_cid, capability_grant_cid)`.
    #[must_use]
    pub fn new_node(
        cid: Cid,
        labels: Vec<String>,
        kind: ChangeKind,
        tx_id: u64,
        node: Option<Node>,
    ) -> Self {
        Self {
            cid,
            labels,
            kind,
            tx_id,
            actor_cid: None,
            handler_cid: None,
            capability_grant_cid: None,
            node,
            edge_endpoints: None,
        }
    }

    /// Construct an Edge-event [`ChangeEvent`]. `kind` must be
    /// [`ChangeKind::EdgeCreated`] or [`ChangeKind::EdgeDeleted`] — anything
    /// else is a caller-side misuse but is not runtime-checked.
    #[must_use]
    pub fn new_edge(
        cid: Cid,
        source: Cid,
        target: Cid,
        label: String,
        kind: ChangeKind,
        tx_id: u64,
    ) -> Self {
        Self {
            cid,
            labels: vec![label.clone()],
            kind,
            tx_id,
            actor_cid: None,
            handler_cid: None,
            capability_grant_cid: None,
            node: None,
            edge_endpoints: Some((source, target, label)),
        }
    }
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
            ChangeKind::EdgeCreated => "EdgeCreated",
            ChangeKind::EdgeDeleted => "EdgeDeleted",
        }
    }

    /// Convenience accessor for callers that only care about the primary
    /// label. Returns `""` when the event carries no labels (idempotent
    /// delete of an already-absent target).
    #[must_use]
    pub fn primary_label(&self) -> &str {
        self.labels.first().map_or("", String::as_str)
    }

    /// True if any of this event's labels equals `label`. Cheap helper for
    /// label-filtered subscribers (IVM views, CDC consumers).
    #[must_use]
    pub fn has_label(&self, label: &str) -> bool {
        self.labels.iter().any(|l| l == label)
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
