//! Higher-level stores layered on top of [`KVBackend`]:
//! [`NodeStore`] / [`EdgeStore`] (blanket impls over any `KVBackend`), plus
//! the [`ChangeSubscriber`] trait and [`ChangeEvent`] schema.
//!
//! R1 triage deliverable **`P1.graph.node-store-trait`**: node and edge CRUD
//! ride a blanket `impl<T: KVBackend>` so any future backend (in-memory mock,
//! WASM peer-fetch) gets the Benten node/edge API for free. The inherent
//! methods on `RedbBackend` remain for backward compatibility with existing
//! call sites — Rust's method resolution picks the inherent method when the
//! trait is not in scope and is unambiguous either way.
//!
//! Change-stream plumbing: [`ChangeSubscriber`] declares the trait shape here
//! in `benten-graph`; the concrete broadcast channel and the actual emission
//! on commit land in G3 per the R1 architect decision (no tokio dep in
//! benten-graph).

use benten_core::{Cid, CoreError, Edge, Node};

use crate::backend::KVBackend;

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

const NODE_PREFIX: &[u8] = b"n:";
const EDGE_PREFIX: &[u8] = b"e:";
const EDGE_SRC_PREFIX: &[u8] = b"es:";
const EDGE_TGT_PREFIX: &[u8] = b"et:";

fn node_key(cid: &Cid) -> Vec<u8> {
    let mut k = Vec::with_capacity(NODE_PREFIX.len() + cid.as_bytes().len());
    k.extend_from_slice(NODE_PREFIX);
    k.extend_from_slice(cid.as_bytes());
    k
}

fn edge_key(cid: &Cid) -> Vec<u8> {
    let mut k = Vec::with_capacity(EDGE_PREFIX.len() + cid.as_bytes().len());
    k.extend_from_slice(EDGE_PREFIX);
    k.extend_from_slice(cid.as_bytes());
    k
}

fn edge_src_index_key(source: &Cid, edge: &Cid) -> Vec<u8> {
    let mut k =
        Vec::with_capacity(EDGE_SRC_PREFIX.len() + source.as_bytes().len() + edge.as_bytes().len());
    k.extend_from_slice(EDGE_SRC_PREFIX);
    k.extend_from_slice(source.as_bytes());
    k.extend_from_slice(edge.as_bytes());
    k
}

fn edge_src_index_prefix(source: &Cid) -> Vec<u8> {
    let mut k = Vec::with_capacity(EDGE_SRC_PREFIX.len() + source.as_bytes().len());
    k.extend_from_slice(EDGE_SRC_PREFIX);
    k.extend_from_slice(source.as_bytes());
    k
}

fn edge_tgt_index_key(target: &Cid, edge: &Cid) -> Vec<u8> {
    let mut k =
        Vec::with_capacity(EDGE_TGT_PREFIX.len() + target.as_bytes().len() + edge.as_bytes().len());
    k.extend_from_slice(EDGE_TGT_PREFIX);
    k.extend_from_slice(target.as_bytes());
    k.extend_from_slice(edge.as_bytes());
    k
}

fn edge_tgt_index_prefix(target: &Cid) -> Vec<u8> {
    let mut k = Vec::with_capacity(EDGE_TGT_PREFIX.len() + target.as_bytes().len());
    k.extend_from_slice(EDGE_TGT_PREFIX);
    k.extend_from_slice(target.as_bytes());
    k
}

/// Format a decode failure into a [`CoreError`]. We reuse `CoreError::Serialize`
/// rather than introducing a parallel enum variant: the DAG-CBOR decoder
/// surfaces the same class of problem (bytes ↔ typed value), and CoreError
/// already tracks the same error category for the encode direction.
fn decode_err<E: core::fmt::Display>(e: E) -> CoreError {
    CoreError::Serialize(format!("decode: {e}"))
}

// ---------------------------------------------------------------------------
// NodeStore
// ---------------------------------------------------------------------------

/// Node-level storage API. Any [`KVBackend`] whose error type can absorb a
/// [`CoreError`] (via `From`) gets a working `NodeStore` for free via the
/// blanket impl below.
///
/// The trait sits above `KVBackend` (which is pure bytes) and owns the
/// Node ↔ bytes DAG-CBOR transition plus the `n:`-prefix key schema.
pub trait NodeStore {
    /// Error type. In practice equal to the underlying
    /// [`KVBackend::Error`] — the blanket impl forwards unchanged.
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

#[allow(
    clippy::useless_conversion,
    reason = "The `Into::into` calls below adapt CoreError to T::Error via the \
              `T::Error: From<CoreError>` bound. Clippy fires when T::Error is \
              inferred as CoreError itself (e.g., in some benchmarks); the \
              conversion is still required for any backend whose Error isn't \
              CoreError — GraphError, future in-memory mocks, peer-fetch, etc."
)]
impl<T> NodeStore for T
where
    T: KVBackend,
    T::Error: From<CoreError>,
{
    type Error = T::Error;

    fn put_node(&self, node: &Node) -> Result<Cid, Self::Error> {
        let cid = node.cid().map_err(Into::into)?;
        let bytes = node.canonical_bytes().map_err(Into::into)?;
        self.put(&node_key(&cid), &bytes)?;
        Ok(cid)
    }

    fn get_node(&self, cid: &Cid) -> Result<Option<Node>, Self::Error> {
        let Some(bytes) = self.get(&node_key(cid))? else {
            return Ok(None);
        };
        let node: Node = serde_ipld_dagcbor::from_slice(&bytes)
            .map_err(decode_err)
            .map_err(Into::into)?;
        Ok(Some(node))
    }

    fn delete_node(&self, cid: &Cid) -> Result<(), Self::Error> {
        self.delete(&node_key(cid))
    }
}

// ---------------------------------------------------------------------------
// EdgeStore
// ---------------------------------------------------------------------------

/// Edge-level storage API. Any [`KVBackend`] whose error type can absorb a
/// [`CoreError`] gets a working `EdgeStore` for free via the blanket impl.
///
/// Edges are content-addressed over `(source, target, label, properties)`;
/// `put_edge` also writes the two index keys (`es:SRC|EDGE` and
/// `et:TGT|EDGE`) so `edges_from` and `edges_to` resolve in O(matches).
pub trait EdgeStore {
    /// Error type — equal to the underlying [`KVBackend::Error`].
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

#[allow(
    clippy::useless_conversion,
    reason = "See NodeStore blanket impl — same rationale: Into::into adapts \
              CoreError through the From<CoreError> bound on T::Error."
)]
impl<T> EdgeStore for T
where
    T: KVBackend,
    T::Error: From<CoreError>,
{
    type Error = T::Error;

    fn put_edge(&self, edge: &Edge) -> Result<Cid, Self::Error> {
        let cid = edge.cid().map_err(Into::into)?;
        let bytes = edge.canonical_bytes().map_err(Into::into)?;

        // Body first, then indexes. `put_batch` would be the atomic shape,
        // but the body/index pair is idempotent (re-putting the same edge
        // writes identical bytes to the same keys), so ordering under a
        // non-transactional call is not load-bearing at Phase 1.
        self.put(&edge_key(&cid), &bytes)?;
        self.put(&edge_src_index_key(&edge.source, &cid), &[])?;
        self.put(&edge_tgt_index_key(&edge.target, &cid), &[])?;
        Ok(cid)
    }

    fn get_edge(&self, cid: &Cid) -> Result<Option<Edge>, Self::Error> {
        let Some(bytes) = self.get(&edge_key(cid))? else {
            return Ok(None);
        };
        let edge: Edge = serde_ipld_dagcbor::from_slice(&bytes)
            .map_err(decode_err)
            .map_err(Into::into)?;
        Ok(Some(edge))
    }

    fn delete_edge(&self, cid: &Cid) -> Result<(), Self::Error> {
        // Read the edge first so we know which index entries to remove.
        if let Some(edge) = self.get_edge(cid)? {
            self.delete(&edge_src_index_key(&edge.source, cid))?;
            self.delete(&edge_tgt_index_key(&edge.target, cid))?;
        }
        self.delete(&edge_key(cid))
    }

    fn edges_from(&self, source: &Cid) -> Result<Vec<Edge>, Self::Error> {
        let hits = self.scan(&edge_src_index_prefix(source))?;
        let mut out = Vec::with_capacity(hits.len());
        for (k, _v) in &*hits {
            // Edge CID is the suffix after `es:` + source-CID bytes.
            let Some(edge_cid_bytes) = k.get(EDGE_SRC_PREFIX.len() + source.as_bytes().len()..)
            else {
                continue;
            };
            let edge_cid = Cid::from_bytes(edge_cid_bytes).map_err(Into::into)?;
            if let Some(edge) = self.get_edge(&edge_cid)? {
                out.push(edge);
            }
        }
        Ok(out)
    }

    fn edges_to(&self, target: &Cid) -> Result<Vec<Edge>, Self::Error> {
        let hits = self.scan(&edge_tgt_index_prefix(target))?;
        let mut out = Vec::with_capacity(hits.len());
        for (k, _v) in &*hits {
            let Some(edge_cid_bytes) = k.get(EDGE_TGT_PREFIX.len() + target.as_bytes().len()..)
            else {
                continue;
            };
            let edge_cid = Cid::from_bytes(edge_cid_bytes).map_err(Into::into)?;
            if let Some(edge) = self.get_edge(&edge_cid)? {
                out.push(edge);
            }
        }
        Ok(out)
    }
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
