//! Subgraph types: `PrimitiveKind`, `OperationNode`, `NodeHandle`, `Subgraph`,
//! `SubgraphBuilder`.
//!
//! Phase-2b G12-C-cont (Phase 2b R6 A1 closure) relocates these types from
//! `benten-eval` to `benten-core`. The relocation eliminates the dual
//! `Subgraph` definition (Phase-2a stub in core, production in eval) that
//! was Inv-13-unsafe — two handlers with the same `(handler_id,
//! deterministic)` pair could collide under the stub encoding even when their
//! node/edge sets differed.
//!
//! # Authoritative canonical-bytes encoding
//!
//! `Subgraph::cid()` hashes BLAKE3 over the canonical DAG-CBOR bytes produced
//! by [`canonical_subgraph_bytes`] (q.v.). The encoding is:
//!
//! ```text
//! { handler_id, nodes-sorted, edges-sorted }
//! ```
//!
//! `nodes` is sorted by `(id, kind_tag)` and `edges` by `(from, to, label)`
//! before encoding so two builders that produce the same final structure via
//! different construction orders hash to byte-identical CIDs (Invariant-10
//! order-independence).
//!
//! # What stays in benten-eval
//!
//! Validation (`validate`), per-handle multiplicative-budget walk
//! (`cumulative_budget_*_for_test`), and Mermaid rendering (`to_mermaid`) are
//! exposed via the `SubgraphExt` extension trait in `benten-eval` so the
//! invariants module surface (~2,000 LOC of `crates/benten-eval/src/
//! invariants/`) does NOT pull into `benten-core`. The arch-1 invariant
//! (`benten-core` MUST NOT depend on `benten-eval`) stays intact.

use crate::{Cid, CoreError, Value};
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

/// Property key under which a primitive node declares it carries an Inv-14
/// AttributionFrame from the dispatcher. The eval-side `SubgraphBuilder`
/// stamps this property `Value::Bool(true)` on every emitted `OperationNode`
/// by default; tests that want to probe the Inv-14 reject path construct
/// `OperationNode`s directly (bypassing the builder).
///
/// Located in benten-core so the core-side `SubgraphBuilder` can stamp the
/// same default without depending on `benten-eval::invariants::attribution`.
pub const ATTRIBUTION_PROPERTY_KEY: &str = "attribution";

/// The 12 operation primitives. Phase 1 executes 8 (READ, WRITE, TRANSFORM,
/// BRANCH, ITERATE, CALL, RESPOND, EMIT). Phase 2 enables WAIT, STREAM,
/// SUBSCRIBE, SANDBOX. The vocabulary is closed by ENGINE-SPEC §3;
/// `non_exhaustive` guards against a future Phase-2+ decision to add a 13th
/// primitive without forcing a major-version bump across downstream
/// matchers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PrimitiveKind {
    /// READ a Node from the graph.
    Read,
    /// WRITE a Node to the graph.
    Write,
    /// TRANSFORM applies a pure expression.
    Transform,
    /// BRANCH selects between typed-error edges by predicate.
    Branch,
    /// ITERATE walks a bounded loop.
    Iterate,
    /// WAIT suspends the handler on a timer or signal.
    Wait,
    /// CALL dispatches to another handler.
    Call,
    /// RESPOND terminates the handler with an output.
    Respond,
    /// EMIT publishes a topic message.
    Emit,
    /// SANDBOX hosts WASM execution (Phase 2b).
    Sandbox,
    /// SUBSCRIBE registers reactive change notification.
    Subscribe,
    /// STREAM emits partial output with back-pressure.
    Stream,
}

impl PrimitiveKind {
    /// True if the primitive's executor is implemented in Phase 1.
    /// Phase 2 primitives (WAIT, STREAM, SUBSCRIBE-as-user-op, SANDBOX)
    /// pass structural validation but return `E_PRIMITIVE_NOT_IMPLEMENTED`
    /// at call time.
    #[must_use]
    pub fn is_phase_1_executable(&self) -> bool {
        matches!(
            self,
            PrimitiveKind::Read
                | PrimitiveKind::Write
                | PrimitiveKind::Transform
                | PrimitiveKind::Branch
                | PrimitiveKind::Iterate
                | PrimitiveKind::Call
                | PrimitiveKind::Respond
                | PrimitiveKind::Emit
        )
    }

    /// Determinism classification (Invariant 9).
    ///
    /// Returns `true` if this primitive's **output-to-caller** is a pure
    /// function of its inputs — repeat executions with identical inputs
    /// produce identical return values, with no wall-clock / RNG / network
    /// non-determinism leaking into the returned `Value`.
    ///
    /// Primitives with side effects (WRITE, RESPOND) are still classified
    /// `true` under this semantic: their observable return to the caller
    /// is determined only by inputs, and the side-effect itself (a storage
    /// mutation, a response emit) is separately tracked by the engine.
    #[must_use]
    pub fn is_deterministic(&self) -> bool {
        match self {
            PrimitiveKind::Read
            | PrimitiveKind::Write
            | PrimitiveKind::Transform
            | PrimitiveKind::Branch
            | PrimitiveKind::Iterate
            | PrimitiveKind::Call
            | PrimitiveKind::Respond => true,
            PrimitiveKind::Emit
            | PrimitiveKind::Wait
            | PrimitiveKind::Sandbox
            | PrimitiveKind::Subscribe
            | PrimitiveKind::Stream => false,
        }
    }

    /// The set of typed error edges a primitive may emit (by label).
    #[must_use]
    pub fn error_edges(&self) -> &'static [&'static str] {
        match self {
            PrimitiveKind::Read => &["ON_NOT_FOUND", "ON_EMPTY", "ON_DENIED", "ON_ERROR"],
            PrimitiveKind::Write => &["ON_CONFLICT", "ON_DENIED", "ON_ERROR"],
            PrimitiveKind::Transform => &["ON_ERROR"],
            PrimitiveKind::Branch => &["ON_DEFAULT"],
            PrimitiveKind::Iterate => &["ON_LIMIT", "ON_ERROR"],
            PrimitiveKind::Call => &["ON_DENIED", "ON_LIMIT", "ON_ERROR"],
            PrimitiveKind::Respond => &[],
            PrimitiveKind::Emit => &["ON_ERROR"],
            PrimitiveKind::Sandbox => &["ON_ERROR", "ON_FUEL", "ON_TIMEOUT", "ON_OUTPUT_LIMIT"],
            PrimitiveKind::Subscribe => &["ON_ERROR"],
            PrimitiveKind::Stream => &["ON_ERROR", "ON_BACKPRESSURE"],
            PrimitiveKind::Wait => &["ON_TIMEOUT", "ON_ERROR"],
        }
    }

    /// Stable string tag used in canonical DAG-CBOR encoding. Frozen — any
    /// rename here is a CID-breaking change.
    #[must_use]
    pub fn canonical_tag(&self) -> &'static str {
        match self {
            PrimitiveKind::Read => "READ",
            PrimitiveKind::Write => "WRITE",
            PrimitiveKind::Transform => "TRANSFORM",
            PrimitiveKind::Branch => "BRANCH",
            PrimitiveKind::Iterate => "ITERATE",
            PrimitiveKind::Wait => "WAIT",
            PrimitiveKind::Call => "CALL",
            PrimitiveKind::Respond => "RESPOND",
            PrimitiveKind::Emit => "EMIT",
            PrimitiveKind::Sandbox => "SANDBOX",
            PrimitiveKind::Subscribe => "SUBSCRIBE",
            PrimitiveKind::Stream => "STREAM",
        }
    }
}

/// Operation Node — the subgraph-level unit of execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationNode {
    /// Node id, unique within the enclosing subgraph.
    pub id: String,
    /// Operation primitive this node executes.
    pub kind: PrimitiveKind,
    /// Property map (used by validators, primitive executors, and the
    /// canonical-bytes encoder).
    pub properties: BTreeMap<String, Value>,
}

impl OperationNode {
    /// Construct a new OperationNode with empty properties.
    #[must_use]
    pub fn new(id: impl Into<String>, kind: PrimitiveKind) -> Self {
        Self {
            id: id.into(),
            kind,
            properties: BTreeMap::new(),
        }
    }

    /// Builder-style: set a property and return self.
    #[must_use]
    pub fn with_property(mut self, k: impl Into<String>, v: Value) -> Self {
        self.properties.insert(k.into(), v);
        self
    }

    /// Read a property by key.
    #[must_use]
    pub fn property(&self, k: &str) -> Option<&Value> {
        self.properties.get(k)
    }

    /// Alias for [`Self::kind`] — back-compat name used by Phase 2a tests.
    #[must_use]
    pub fn primitive_kind(&self) -> PrimitiveKind {
        self.kind
    }
}

/// Opaque handle returned by `SubgraphBuilder` when adding nodes. Tests
/// use these as arguments to `add_edge`, `transform`, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeHandle(pub u32);

/// A subgraph (set of OperationNodes + directed edges between them).
///
/// **Phase 2b G12-C-cont relocation** brings this type from `benten-eval` to
/// `benten-core`. Fields are `pub` so the `benten-eval` invariants module
/// (which previously reached into `pub(crate)` siblings) can keep working
/// without an accessor-everything cascade. Mutation discipline is enforced
/// at registration time by Inv-13 (`benten-graph::immutability`); the public
/// fields preserve the field-access ergonomics existing eval-side
/// `validate_subgraph`, `system_zone`, and `mermaid` walkers depend on.
///
/// # Canonical-bytes encoding
///
/// `Subgraph::to_dagcbor` / [`canonical_subgraph_bytes`] use the
/// `CanonView` schema (sorted nodes + sorted edges + handler_id +
/// deterministic). The auto-derived `Serialize` / `Deserialize`
/// implementations are a SECONDARY (non-canonical) serialization for
/// debugging / non-CID-bearing transport — DO NOT route content-addressed
/// bytes through `serde_ipld_dagcbor::to_vec(&subgraph)` directly; always
/// go through `to_dagcbor`/`canonical_subgraph_bytes` so the CID matches
/// the `from_dagcbor` decode.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Subgraph {
    /// Stable handler-registration identity.
    pub handler_id: String,
    /// Operation nodes in builder-emission order. Sorted on canonical-bytes
    /// emit for Invariant-10 order-independence.
    pub nodes: Vec<OperationNode>,
    /// Directed edges as `(from_id, to_id, label)`. Sorted on canonical-bytes
    /// emit for Invariant-10 order-independence.
    pub edges: Vec<(String, String, String)>,
    /// Invariant 9 — declared determinism context. Preserved across the
    /// builder-to-finalized projection (5d-J workstream 4) so the
    /// `validate_subgraph` path can re-run the per-primitive determinism
    /// check on a round-tripped Subgraph. Defaults `false` (unconstrained).
    pub deterministic: bool,
}

impl Subgraph {
    /// Construct an empty Subgraph with the given handler id.
    #[must_use]
    pub fn new(handler_id: impl Into<String>) -> Self {
        Self {
            handler_id: handler_id.into(),
            nodes: Vec::new(),
            edges: Vec::new(),
            deterministic: false,
        }
    }

    /// Phase 2a G3-B test helper: empty Subgraph with the given handler id.
    /// Alias for [`Subgraph::new`].
    #[must_use]
    pub fn empty_for_test(handler_id: impl Into<String>) -> Self {
        Self::new(handler_id)
    }

    /// True when the builder declared this handler deterministic via
    /// [`SubgraphBuilder::declare_deterministic`]. Invariant 9 rejects any
    /// non-deterministic primitive inside a deterministic handler at both
    /// builder and finalized-subgraph validation time.
    #[must_use]
    pub fn is_declared_deterministic(&self) -> bool {
        self.deterministic
    }

    /// Whether the Subgraph is classified deterministic (alias).
    #[must_use]
    pub fn is_deterministic(&self) -> bool {
        self.deterministic
    }

    /// Declare this finalized Subgraph's determinism context after the fact.
    pub fn set_deterministic(&mut self, value: bool) {
        self.deterministic = value;
    }

    /// Read-only accessor for the subgraph's [`OperationNode`]s.
    #[must_use]
    pub fn nodes(&self) -> &[OperationNode] {
        &self.nodes
    }

    /// Read-only accessor for the subgraph's `(from, to, label)` edges.
    #[must_use]
    pub fn edges(&self) -> &[(String, String, String)] {
        &self.edges
    }

    /// Read-only accessor for the subgraph's stable handler id.
    #[must_use]
    pub fn handler_id(&self) -> &str {
        &self.handler_id
    }

    /// Mutable accessor for the first `OperationNode`. Used by
    /// `benten-engine`'s `dispatch_call` to backfill properties on a
    /// synthesized READ / WRITE node after the builder has finalized the
    /// shape.
    pub fn first_op_mut(&mut self) -> Option<&mut OperationNode> {
        self.nodes.first_mut()
    }

    /// Mutable accessor for the (only) `Write` primitive node. Used by the
    /// `crud:create` / `crud:delete` dispatch shims in `benten-engine`.
    pub fn write_op_mut(&mut self) -> Option<&mut OperationNode> {
        self.nodes
            .iter_mut()
            .find(|n| matches!(n.kind, PrimitiveKind::Write))
    }

    /// Mutable accessor for the `OperationNode` whose id matches `id`.
    pub fn op_by_id_mut(&mut self, id: &str) -> Option<&mut OperationNode> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    /// Phase 2a G3-B test helper: look up a node by its handle.
    #[must_use]
    pub fn node_by_handle(&self, h: NodeHandle) -> Option<&OperationNode> {
        self.nodes.get(h.0 as usize)
    }

    /// Phase 2a G4-A test helper: return the `NodeHandle` for an operation
    /// node id.
    #[must_use]
    pub fn handle_of(&self, id: &str) -> NodeHandle {
        let idx = self.nodes.iter().position(|n| n.id == id).unwrap_or(0);
        NodeHandle(u32::try_from(idx).unwrap_or(u32::MAX))
    }

    /// Builder-style: append a node and return self.
    #[must_use]
    pub fn with_node(mut self, n: OperationNode) -> Self {
        self.nodes.push(n);
        self
    }

    /// Builder-style: append an edge and return self.
    #[must_use]
    pub fn with_edge(
        mut self,
        from: impl Into<String>,
        to: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
        self.edges.push((from.into(), to.into(), label.into()));
        self
    }

    /// Number of OperationNodes in the subgraph (diagnostic helper).
    #[must_use]
    pub fn primitive_count(&self) -> usize {
        self.nodes.len()
    }

    /// Canonical DAG-CBOR encoding. Nodes sorted by `(id, kind_tag)`; edges
    /// by `(from, to, label)`.
    ///
    /// # Errors
    /// Returns [`CoreError::Serialize`] on DAG-CBOR failure.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CoreError> {
        canonical_subgraph_bytes(self)
    }

    /// Content-addressed CID for the subgraph (Invariant 10).
    ///
    /// Hashes BLAKE3 over the canonical-bytes encoding (nodes + edges sorted
    /// so the CID is order-independent).
    ///
    /// # Errors
    /// Returns [`CoreError::Serialize`] if DAG-CBOR encoding fails.
    pub fn cid(&self) -> Result<Cid, CoreError> {
        let bytes = self.canonical_bytes()?;
        let digest = blake3::hash(&bytes);
        Ok(Cid::from_blake3_digest(*digest.as_bytes()))
    }

    /// G12-C: DAG-CBOR encode (alias for [`Subgraph::canonical_bytes`]).
    /// The shorter name (no underscore) matches the round-trip-test
    /// convention `to_dagcbor` ↔ `from_dagcbor`.
    ///
    /// # Errors
    /// Returns [`CoreError::Serialize`] on encode failure.
    pub fn to_dag_cbor(&self) -> Result<Vec<u8>, CoreError> {
        self.canonical_bytes()
    }

    /// G12-C: alias for [`Subgraph::to_dag_cbor`].
    ///
    /// # Errors
    /// Returns [`CoreError::Serialize`] on encode failure.
    pub fn to_dagcbor(&self) -> Result<Vec<u8>, CoreError> {
        self.canonical_bytes()
    }

    /// G12-C: load a Subgraph from DAG-CBOR bytes (no CID check). Decodes
    /// the canonical-bytes shape produced by [`canonical_subgraph_bytes`].
    ///
    /// # Errors
    /// Returns [`CoreError::Serialize`] on decode failure.
    pub fn load_verified(bytes: &[u8]) -> Result<Self, CoreError> {
        let owned: CanonViewOwned = serde_ipld_dagcbor::from_slice(bytes)
            .map_err(|e| CoreError::Serialize(format!("{e}")))?;
        Self::from_canonical_owned(owned)
    }

    /// G12-C: alias for [`Subgraph::load_verified`].
    ///
    /// # Errors
    /// Returns [`CoreError::Serialize`] on decode failure.
    pub fn from_dagcbor(bytes: &[u8]) -> Result<Self, CoreError> {
        Self::load_verified(bytes)
    }

    /// Phase-2b benten-core-migration: load a Subgraph from bytes + an
    /// expected CID. Mismatch fires `E_INV_CONTENT_HASH`. Mirrors
    /// [`crate::Node::load_verified`]: hash first, then decode.
    ///
    /// # Errors
    /// - [`CoreError::ContentHashMismatch`] on CID mismatch.
    /// - [`CoreError::Serialize`] on decode failure.
    pub fn load_verified_with_cid(expected_cid: &Cid, bytes: &[u8]) -> Result<Self, CoreError> {
        let digest = blake3::hash(bytes);
        let recomputed = Cid::from_blake3_digest(*digest.as_bytes());
        if &recomputed != expected_cid {
            return Err(CoreError::ContentHashMismatch {
                path: "subgraph",
                expected: *expected_cid,
                actual: recomputed,
            });
        }
        Self::load_verified(bytes)
    }
}

/// Produce the canonical DAG-CBOR byte encoding of a subgraph.
///
/// Nodes and edges are sorted before encoding so the resulting bytes depend
/// only on the final set of nodes and edges, not on construction order. This
/// is the key property Invariant 10 guarantees — two builders that produce
/// the same final structure via different paths hash to the same CID.
///
/// **G12-C-cont authoritative encoding shape (strict superset of both prior
/// shapes — Inv-13-correct):**
///
/// ```text
/// { handler_id: text,
///   nodes:       [{ id, kind, properties } ... sorted by (id, kind)],
///   edges:       [{ from, to, label } ... sorted by (from, to, label)],
///   deterministic: bool }
/// ```
///
/// The encoding includes `deterministic` so the precursor PR #20 contract
/// (Subgraph CID differs when `deterministic` flag differs) survives the
/// type relocation, AND includes `nodes`/`edges` so the production-shape
/// Inv-13 collision-stability contract holds (two handlers with the same
/// `(handler_id, deterministic)` pair but different node sets produce
/// different CIDs).
///
/// # Errors
/// Returns [`CoreError::Serialize`] on DAG-CBOR failure.
pub fn canonical_subgraph_bytes(sg: &Subgraph) -> Result<Vec<u8>, CoreError> {
    let view = sg.canonical_view();
    serde_ipld_dagcbor::to_vec(&view).map_err(|e| CoreError::Serialize(format!("{e}")))
}

impl Subgraph {
    fn canonical_view(&self) -> CanonView<'_> {
        let mut nodes: Vec<CanonNodeRef<'_>> = self
            .nodes
            .iter()
            .map(|n| CanonNodeRef {
                id: &n.id,
                kind: n.kind.canonical_tag(),
                properties: &n.properties,
            })
            .collect();
        nodes.sort_by(|a, b| (a.id, a.kind).cmp(&(b.id, b.kind)));

        let mut edges: Vec<CanonEdgeRef<'_>> = self
            .edges
            .iter()
            .map(|(f, t, l)| CanonEdgeRef {
                from: f,
                to: t,
                label: l,
            })
            .collect();
        edges.sort_by(|a, b| (a.from, a.to, a.label).cmp(&(b.from, b.to, b.label)));

        CanonView {
            handler_id: &self.handler_id,
            nodes,
            edges,
            deterministic: self.deterministic,
        }
    }

    fn from_canonical_owned(owned: CanonViewOwned) -> Result<Self, CoreError> {
        let nodes: Result<Vec<OperationNode>, CoreError> = owned
            .nodes
            .into_iter()
            .map(|n| {
                Ok(OperationNode {
                    id: n.id,
                    kind: PrimitiveKind::from_canonical_tag(&n.kind)?,
                    properties: n.properties,
                })
            })
            .collect();
        let nodes = nodes?;
        let edges: Vec<(String, String, String)> = owned
            .edges
            .into_iter()
            .map(|e| (e.from, e.to, e.label))
            .collect();
        Ok(Self {
            handler_id: owned.handler_id,
            nodes,
            edges,
            deterministic: owned.deterministic,
        })
    }
}

impl PrimitiveKind {
    /// Inverse of [`PrimitiveKind::canonical_tag`]. Unknown tag fails with
    /// [`CoreError::Serialize`] — pre-fix-pass behaviour silently mapped
    /// unknown tags to `Read`, masking encoder drift in the rare case that
    /// the caller used `Subgraph::load_verified` (no-CID variant) where the
    /// hash check could not reject corrupted-tag bytes first. Brought to
    /// attention as G12-C-cont fix-pass A.8 (cag-mr-g12c-cont-2).
    fn from_canonical_tag(tag: &str) -> Result<Self, CoreError> {
        match tag {
            "READ" => Ok(Self::Read),
            "WRITE" => Ok(Self::Write),
            "TRANSFORM" => Ok(Self::Transform),
            "BRANCH" => Ok(Self::Branch),
            "ITERATE" => Ok(Self::Iterate),
            "WAIT" => Ok(Self::Wait),
            "CALL" => Ok(Self::Call),
            "RESPOND" => Ok(Self::Respond),
            "EMIT" => Ok(Self::Emit),
            "SANDBOX" => Ok(Self::Sandbox),
            "SUBSCRIBE" => Ok(Self::Subscribe),
            "STREAM" => Ok(Self::Stream),
            other => Err(CoreError::Serialize(format!(
                "unknown PrimitiveKind canonical tag: {other:?}"
            ))),
        }
    }
}

#[derive(Serialize)]
struct CanonNodeRef<'a> {
    id: &'a str,
    kind: &'static str,
    properties: &'a BTreeMap<String, Value>,
}

#[derive(Serialize)]
struct CanonEdgeRef<'a> {
    from: &'a str,
    to: &'a str,
    label: &'a str,
}

#[derive(Serialize)]
struct CanonView<'a> {
    handler_id: &'a str,
    nodes: Vec<CanonNodeRef<'a>>,
    edges: Vec<CanonEdgeRef<'a>>,
    deterministic: bool,
}

#[derive(Deserialize)]
struct CanonNodeOwned {
    id: String,
    kind: String,
    properties: BTreeMap<String, Value>,
}

#[derive(Deserialize)]
struct CanonEdgeOwned {
    from: String,
    to: String,
    label: String,
}

#[derive(Deserialize)]
struct CanonViewOwned {
    handler_id: String,
    nodes: Vec<CanonNodeOwned>,
    edges: Vec<CanonEdgeOwned>,
    deterministic: bool,
}

// ---------------------------------------------------------------------------
// SubgraphBuilder
// ---------------------------------------------------------------------------

/// Ergonomic builder used by the invariant edge-case tests, registration
/// negative-contract tests, and the engine's `IntoSubgraphSpec` conversion
/// path.
///
/// **G12-C-cont relocation:** `SubgraphBuilder` moves to `benten-core` along
/// with `Subgraph`. The builder produces well-formed `Subgraph` values via
/// `build_unvalidated_for_test` (always succeeds) and `build_validated`
/// (delegates to `benten-eval::SubgraphExt::validate` post-build). The
/// invariants module surface stays in `benten-eval`; the builder's
/// `build_validated` re-runs the validator via the extension trait,
/// preserving the failure semantics callsites expect.
///
/// Because `benten-core` MUST NOT depend on `benten-eval` (arch-1), the
/// `build_validated` method here is the **unvalidated** body — eval-side
/// callers should use `eval::SubgraphExt::build_validated` (which wraps
/// the builder + validator). Existing test callsites import the builder
/// from `benten_eval::` (which re-exports), so the trait method shadows
/// the inherent unvalidated body and gets validation semantics for free.
pub struct SubgraphBuilder {
    /// Handler id for this subgraph (stamped onto the produced [`Subgraph`]).
    pub handler_id: String,
    /// Operation nodes accumulated by `read` / `write` / `transform` / etc.
    pub nodes: Vec<OperationNode>,
    /// Parallel fanout declared on a node via `iterate_parallel`. Indexed by
    /// NodeHandle position. A value > 1 contributes to Invariant-3 fan-out
    /// as if the node had that many outgoing edges.
    pub parallel_fanout: Vec<usize>,
    /// Per-node iterate nest-depth (zero for non-iterate nodes; otherwise
    /// 1 + depth of the upstream iterate chain).
    pub iterate_depth: Vec<usize>,
    /// Handle-typed edges (resolved into `(String, String, String)` at
    /// `build_*_for_test` time via the internal `node_id` walker).
    pub edges: Vec<(NodeHandle, NodeHandle, String)>,
    /// Test-only synthetic cross-edges the edge-count invariant must see.
    pub extra_edges: usize,
    /// Invariant-9 declared determinism flag.
    pub deterministic: bool,
}

impl SubgraphBuilder {
    /// Construct an empty builder for the named handler.
    #[must_use]
    pub fn new(handler_id: impl Into<String>) -> Self {
        Self {
            handler_id: handler_id.into(),
            nodes: Vec::new(),
            parallel_fanout: Vec::new(),
            iterate_depth: Vec::new(),
            edges: Vec::new(),
            extra_edges: 0,
            deterministic: false,
        }
    }

    /// Declare the subgraph's determinism-context flag (Invariant 9).
    pub fn declare_deterministic(&mut self, value: bool) -> &mut Self {
        self.deterministic = value;
        self
    }

    fn push(&mut self, mut op: OperationNode) -> NodeHandle {
        // G11-A EVAL wave-1 (D12.7 Decision 1): the `SubgraphBuilder` is the
        // canonical Inv-14 attribution stamp surface. Every OperationNode it
        // emits declares `attribution: true` by default; tests that want to
        // probe the Inv-14 reject path construct `OperationNode`s directly
        // (bypassing the builder).
        op.properties
            .entry(ATTRIBUTION_PROPERTY_KEY.to_string())
            .or_insert(Value::Bool(true));
        let h = NodeHandle(u32::try_from(self.nodes.len()).unwrap_or(u32::MAX));
        self.nodes.push(op);
        self.parallel_fanout.push(1);
        self.iterate_depth.push(0);
        h
    }

    fn push_chained(&mut self, op: OperationNode, prev: NodeHandle, nest: usize) -> NodeHandle {
        let h = self.push(op);
        self.iterate_depth[h.0 as usize] = nest;
        self.edges.push((prev, h, "next".into()));
        h
    }

    /// Append a READ primitive.
    pub fn read(&mut self, id: impl Into<String>) -> NodeHandle {
        self.push(OperationNode::new(id, PrimitiveKind::Read))
    }

    /// Append a WRITE primitive.
    pub fn write(&mut self, id: impl Into<String>) -> NodeHandle {
        self.push(OperationNode::new(id, PrimitiveKind::Write))
    }

    /// Push an operation node of an arbitrary [`PrimitiveKind`] onto the
    /// builder, returning its handle. This is the single push-only entry
    /// point for all 12 kinds; existing kind-specific helpers (`read`,
    /// `write`, etc.) chain on a previous node, while this raw-push variant
    /// is the lowest-level constructor.
    pub fn push_primitive(&mut self, id: impl Into<String>, kind: PrimitiveKind) -> NodeHandle {
        self.push(OperationNode::new(id, kind))
    }

    /// Append a TRANSFORM after `prev`.
    pub fn transform(&mut self, prev: NodeHandle, _expr: &str) -> NodeHandle {
        let id = format!("transform_{}", self.nodes.len());
        let nest = self.iterate_depth_of(prev);
        self.push_chained(OperationNode::new(id, PrimitiveKind::Transform), prev, nest)
    }

    /// Append a BRANCH after `prev`.
    pub fn branch(&mut self, prev: NodeHandle, _expr: &str) -> NodeHandle {
        let id = format!("branch_{}", self.nodes.len());
        let nest = self.iterate_depth_of(prev);
        self.push_chained(OperationNode::new(id, PrimitiveKind::Branch), prev, nest)
    }

    /// Add a CALL operation that dispatches to the named handler. The
    /// `handler_id` is stamped onto the new CALL node's `handler` property.
    pub fn call_handler(&mut self, prev: NodeHandle, handler_id: &str) -> NodeHandle {
        let id = format!("call_{}", self.nodes.len());
        let op = OperationNode::new(id, PrimitiveKind::Call)
            .with_property("handler", Value::text(handler_id.to_string()));
        let nest = self.iterate_depth_of(prev);
        self.push_chained(op, prev, nest)
    }

    /// Append an ITERATE with declared `max` bound.
    pub fn iterate(&mut self, prev: NodeHandle, _body: &str, max: u64) -> NodeHandle {
        let id = format!("iterate_{}", self.nodes.len());
        let op = OperationNode::new(id, PrimitiveKind::Iterate)
            .with_property("max", Value::Int(i64::try_from(max).unwrap_or(i64::MAX)));
        let nest = self.iterate_depth_of(prev) + 1;
        self.push_chained(op, prev, nest)
    }

    /// Append a SANDBOX operation that targets `module`.
    pub fn sandbox(&mut self, prev: NodeHandle, module: &str) -> NodeHandle {
        let id = format!("sandbox_{}", self.nodes.len());
        let op = OperationNode::new(id, PrimitiveKind::Sandbox)
            .with_property("module", Value::text(module.to_string()));
        let nest = self.iterate_depth_of(prev);
        self.push_chained(op, prev, nest)
    }

    /// Append a RESPOND terminator after `prev`.
    pub fn respond(&mut self, prev: NodeHandle) -> NodeHandle {
        let id = format!("respond_{}", self.nodes.len());
        let nest = self.iterate_depth_of(prev);
        self.push_chained(OperationNode::new(id, PrimitiveKind::Respond), prev, nest)
    }

    /// Append an EMIT for the named topic after `prev`.
    pub fn emit(&mut self, prev: NodeHandle, _topic: &str) -> NodeHandle {
        let id = format!("emit_{}", self.nodes.len());
        let nest = self.iterate_depth_of(prev);
        self.push_chained(OperationNode::new(id, PrimitiveKind::Emit), prev, nest)
    }

    /// Phase 2a G3-B (dx-r1-8): WAIT signal variant. Sets the `signal`
    /// property on the created node.
    pub fn wait_signal(&mut self, prev: NodeHandle, signal_name: impl Into<String>) -> NodeHandle {
        let id = format!("wait_{}", self.nodes.len());
        let op = OperationNode::new(id, PrimitiveKind::Wait)
            .with_property("signal", Value::text(signal_name));
        let nest = self.iterate_depth_of(prev);
        self.push_chained(op, prev, nest)
    }

    /// Phase 2a G3-B: WAIT signal variant with explicit timeout.
    pub fn wait_signal_with_timeout(
        &mut self,
        prev: NodeHandle,
        signal_name: impl Into<String>,
        timeout: core::time::Duration,
    ) -> NodeHandle {
        let h = self.wait_signal(prev, signal_name);
        let idx = h.0 as usize;
        let ms = i64::try_from(timeout.as_millis()).unwrap_or(i64::MAX);
        if let Some(n) = self.nodes.get_mut(idx) {
            n.properties.insert("timeout_ms".into(), Value::Int(ms));
        }
        h
    }

    /// Phase 2a G3-B: WAIT duration variant.
    pub fn wait_duration(
        &mut self,
        prev: NodeHandle,
        duration: core::time::Duration,
    ) -> NodeHandle {
        let id = format!("wait_{}", self.nodes.len());
        let ms = i64::try_from(duration.as_millis()).unwrap_or(i64::MAX);
        let op = OperationNode::new(id, PrimitiveKind::Wait)
            .with_property("duration_ms", Value::Int(ms));
        let nest = self.iterate_depth_of(prev);
        self.push_chained(op, prev, nest)
    }

    /// Phase 2a G4-A / Code-as-graph Major #2: CALL with an explicit
    /// `isolated` flag. `isolated: true` resets the multiplicative budget
    /// to the callee grant's declared bound.
    pub fn call_with_isolated(
        &mut self,
        prev: NodeHandle,
        handler: &str,
        isolated: bool,
    ) -> NodeHandle {
        let id = format!("call_{}", self.nodes.len());
        let op = OperationNode::new(id, PrimitiveKind::Call)
            .with_property("handler", Value::text(handler.to_string()))
            .with_property("isolated", Value::Bool(isolated));
        let nest = self.iterate_depth_of(prev);
        self.push_chained(op, prev, nest)
    }

    /// Phase 2a test-only property setter — used by
    /// `wait_signal_shape_optional_typing` to inject malformed payloads.
    pub fn set_property_for_test(&mut self, h: NodeHandle, key: &str, value: Value) -> &mut Self {
        if let Some(n) = self.nodes.get_mut(h.0 as usize) {
            n.properties.insert(key.to_string(), value);
        }
        self
    }

    /// Append an ITERATE with declared `parallel` fan-out (for Invariant-3
    /// fan-out checks).
    pub fn iterate_parallel(&mut self, prev: NodeHandle, _body: &str, max: usize) -> NodeHandle {
        let id = format!("iterate_par_{}", self.nodes.len());
        let op = OperationNode::new(id, PrimitiveKind::Iterate).with_property(
            "parallel",
            Value::Int(i64::try_from(max).unwrap_or(i64::MAX)),
        );
        let nest = self.iterate_depth_of(prev) + 1;
        let h = self.push_chained(op, prev, nest);
        self.parallel_fanout[h.0 as usize] = max;
        h
    }

    /// Append an explicit edge between two existing handles.
    pub fn add_edge(&mut self, from: NodeHandle, to: NodeHandle) -> &mut Self {
        self.edges.push((from, to, "next".into()));
        self
    }

    fn iterate_depth_of(&self, h: NodeHandle) -> usize {
        self.iterate_depth.get(h.0 as usize).copied().unwrap_or(0)
    }

    fn node_id(&self, h: NodeHandle) -> String {
        self.nodes
            .get(h.0 as usize)
            .map_or_else(|| format!("n{}", h.0), |n| n.id.clone())
    }

    /// Build without running validation — used by negative tests that want
    /// to submit invalid subgraphs to the engine's registration path AND by
    /// the core-side round-trip schema-pin tests that exercise encode/decode
    /// without crossing the arch-1 dep boundary into `benten-eval`.
    ///
    /// Eval-side callers reach for
    /// `benten_eval::SubgraphBuilderExt::build_validated` (a trait method)
    /// which adds the invariants pass and returns a `RegistrationError`-typed
    /// result.
    pub fn build_unvalidated_for_test(self) -> Subgraph {
        let edges = self.materialize_edges();
        Subgraph {
            nodes: self.nodes,
            edges,
            handler_id: self.handler_id,
            deterministic: self.deterministic,
        }
    }

    fn materialize_edges(&self) -> Vec<(String, String, String)> {
        let mut out: Vec<(String, String, String)> = self
            .edges
            .iter()
            .map(|(f, t, l)| (self.node_id(*f), self.node_id(*t), l.clone()))
            .collect();
        // Synthetic cross-edges for the Invariant-6 test. Each extra edge
        // references a synthetic placeholder node id so it counts toward the
        // edge total without disturbing the node count.
        for i in 0..self.extra_edges {
            out.push((
                format!("__extra_src_{i}"),
                format!("__extra_dst_{i}"),
                "extra".to_string(),
            ));
        }
        out
    }

    /// Test-only escape hatch: forcibly insert N additional cross-edges into
    /// the subgraph so the edge-count invariant trips. Used by
    /// `invariants_5_6_counts.rs`.
    pub fn force_add_cross_edges_for_testing(&mut self, n: usize) -> &mut Self {
        self.extra_edges = self.extra_edges.saturating_add(n);
        self
    }

    /// Borrow the builder's internal state for the validator. Crate-internal
    /// to benten-core; the eval-side validator reaches in via `nodes()` /
    /// `edges()` / `parallel_fanout()` accessors below.
    #[must_use]
    pub fn nodes_for_validator(&self) -> &[OperationNode] {
        &self.nodes
    }

    /// Validator accessor: parallel-fanout slice indexed by handle position.
    #[must_use]
    pub fn parallel_fanout_for_validator(&self) -> &[usize] {
        &self.parallel_fanout
    }

    /// Validator accessor: per-node iterate nest-depth.
    #[must_use]
    pub fn iterate_depth_for_validator(&self) -> &[usize] {
        &self.iterate_depth
    }

    /// Validator accessor: handle-typed edges.
    #[must_use]
    pub fn edges_for_validator(&self) -> &[(NodeHandle, NodeHandle, String)] {
        &self.edges
    }

    /// Validator accessor: synthetic cross-edge count.
    #[must_use]
    pub fn extra_edges_for_validator(&self) -> usize {
        self.extra_edges
    }

    /// Validator accessor: declared determinism flag.
    #[must_use]
    pub fn deterministic_for_validator(&self) -> bool {
        self.deterministic
    }

    /// Validator accessor: handler id.
    #[must_use]
    pub fn handler_id_for_validator(&self) -> &str {
        &self.handler_id
    }
}

// G12-C-cont fix-pass A.5 (arch-mr-g12c-cont-3): the inherent
// `build_validated_for_corruption_test` body that previously lived here on
// `benten-core::NodeHandle` was deleted in favour of the eval-side
// `NodeHandleExt::build_validated_for_corruption_test` extension trait
// (`benten-eval/src/subgraph_ext.rs`). Per the G4-A C1 / sec-r6r2-02 precedent,
// test-only construction surfaces should not be `pub` on benten-core without
// a cfg-gate; consolidating into the eval-side extension trait keeps the
// surface in one place + lets callsites import via `use benten_eval::NodeHandleExt;`
// (which the only callsite — `crates/benten-eval/tests/invariants_9_10_12.rs`
// — already does).
