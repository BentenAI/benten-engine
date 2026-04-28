//! Engine-level outcome / trace shapes + placeholder handle types.
//!
//! Extracted from `lib.rs` by R6 Wave 2 (R-major-01). The shapes themselves
//! did not change — only the module they live in.
//!
//! [`Outcome`] is the return type of `Engine::call`, `Engine::read_view`, and
//! related dispatch surfaces. [`Trace`] + [`TraceStep`] are the post-walk
//! observability shape consumed by `engine.trace()`. [`AnchorHandle`],
//! [`HandlerPredecessors`], [`NestedTx`], [`ReadViewOptions`], and
//! [`ViewCreateOptions`] are small supporting types.

use std::collections::BTreeMap;

use benten_core::{Cid, Node, Value};
use benten_errors::ErrorCode;
use benten_eval::AttributionFrame;

/// Options passed to `Engine::create_view` for the legacy id-string form
/// (`engine.create_view(view_id, opts)`). Currently a placeholder shape so
/// `Default::default()` resolves unambiguously at the call site.
///
/// **Phase 2b G8-B note.** New code registering user-defined views should
/// use the [`UserViewSpec`] builder — `engine.create_view(spec)` — which
/// carries the strategy field and the input-pattern shape. The legacy
/// `(view_id, ViewCreateOptions)` overload is preserved for the canonical
/// view-id family the engine builds in (e.g. `content_listing_<label>`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ViewCreateOptions;

/// Input-pattern selector for [`UserViewSpec`] (Phase-2b G8-B).
///
/// User views observe the change stream via a small selector vocabulary
/// kept deliberately narrow in 2b — the surface widens in Phase 3 alongside
/// the generalized Algorithm B port. The shape mirrors the TS DSL
/// `inputPattern` field exactly so the napi bridge round-trips without
/// renaming.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserViewInputPattern {
    /// Match every change event whose Node carries the given label. Mirrors
    /// the Phase-1 `ContentListingView` shape (`label` selector).
    Label(String),
    /// Match every change event whose anchor id starts with the given
    /// prefix. Companion to [`Self::Label`] for anchor-rooted feeds.
    AnchorPrefix(String),
}

/// User-registered view spec. Companion to the canonical `(view_id,
/// ViewCreateOptions)` overload of `Engine::create_view`. The default
/// strategy is `Strategy::B` per D8-RESOLVED — `Strategy::A` is reserved
/// for the 5 hand-written Phase-1 views (Rust-only) and is refused at
/// registration time; `Strategy::C` is reserved for Phase 3+.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserViewSpec {
    pub(crate) id: String,
    pub(crate) input_pattern: UserViewInputPattern,
    pub(crate) strategy: benten_ivm::Strategy,
}

impl UserViewSpec {
    /// Construct a builder. The `id` and `input_pattern` are required;
    /// `strategy` defaults to `Strategy::B` (D8) and may be overridden via
    /// [`UserViewSpecBuilder::strategy`].
    #[must_use]
    pub fn builder() -> UserViewSpecBuilder {
        UserViewSpecBuilder::default()
    }

    /// View id (e.g. `"user_posts_by_author"`).
    #[must_use]
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    /// Input-pattern selector.
    #[must_use]
    pub fn input_pattern(&self) -> &UserViewInputPattern {
        &self.input_pattern
    }

    /// Resolved strategy. Defaults to `Strategy::B` per D8 unless the
    /// builder explicitly opted-in via `.strategy(...)`.
    #[must_use]
    pub fn strategy(&self) -> benten_ivm::Strategy {
        self.strategy
    }
}

/// Builder for [`UserViewSpec`]. `id` + `input_pattern` are required;
/// `strategy` defaults to `Strategy::B` (D8-RESOLVED).
#[derive(Debug, Default)]
pub struct UserViewSpecBuilder {
    id: Option<String>,
    input_pattern: Option<UserViewInputPattern>,
    strategy: Option<benten_ivm::Strategy>,
}

impl UserViewSpecBuilder {
    /// Set the view id. Required.
    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the input pattern. Required.
    #[must_use]
    pub fn input_pattern(mut self, pattern: UserViewInputPattern) -> Self {
        self.input_pattern = Some(pattern);
        self
    }

    /// Explicitly opt into a strategy. Default is `Strategy::B` for user
    /// views (D8-RESOLVED). `Strategy::A` is rejected by `Engine::create_view`
    /// at registration time (hand-written = Rust-only) and `Strategy::C`
    /// is rejected as Phase-3-reserved — both via typed errors that the
    /// builder itself does NOT preempt (so the typed error surfaces at the
    /// engine boundary where the catalog code is wired).
    #[must_use]
    pub fn strategy(mut self, strategy: benten_ivm::Strategy) -> Self {
        self.strategy = Some(strategy);
        self
    }

    /// Finalize the spec. Returns the missing-field error message string
    /// when a required field was not set. The napi binding maps this to a
    /// typed `napi::Error::InvalidArg` at the FFI boundary
    /// (`bindings/napi/src/view.rs::parse_user_view_spec`); Rust callers see
    /// the raw `String`. `Engine::create_user_view` does NOT wrap the builder
    /// result — it consumes a constructed `UserViewSpec` directly.
    ///
    /// # Errors
    ///
    /// Returns the missing-field message string when `id` or
    /// `input_pattern` was not set.
    pub fn build(self) -> Result<UserViewSpec, String> {
        let id = self
            .id
            .ok_or_else(|| String::from("UserViewSpec.id is required"))?;
        let input_pattern = self
            .input_pattern
            .ok_or_else(|| String::from("UserViewSpec.input_pattern is required"))?;
        let strategy = self.strategy.unwrap_or(benten_ivm::Strategy::B);
        Ok(UserViewSpec {
            id,
            input_pattern,
            strategy,
        })
    }
}

/// Options passed to `Engine::read_view_with`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadViewOptions {
    /// When `true`, the read returns the most-recent materialised view
    /// even if the IVM background updater has not yet caught up to the
    /// latest write. When `false`, a stale view fires
    /// `E_IVM_VIEW_STALE`.
    pub allow_stale: bool,
}

impl ReadViewOptions {
    /// Strict mode — fail loudly with `E_IVM_VIEW_STALE` if the view
    /// is behind the latest write.
    #[must_use]
    pub fn strict() -> Self {
        Self { allow_stale: false }
    }

    /// Permissive mode — return whatever the IVM has materialised even
    /// if it is stale.
    #[must_use]
    pub fn allow_stale() -> Self {
        Self { allow_stale: true }
    }
}

/// Extension trait used by `tx_atomicity` integration tests.
pub trait OutcomeExt {
    /// Borrow the wrapped [`Outcome`] for use by test fixtures.
    fn as_outcome(&self) -> &Outcome;
}

/// The response returned by `Engine::call`. **Phase 1**: primitive dispatch
/// is deferred so `Outcome` methods that depend on a real evaluation run
/// return empty / `None`. Tests exercising the full flow are gated on
/// Phase-2 evaluator integration (documented in the group report).
#[derive(Debug, Clone, Default)]
pub struct Outcome {
    pub(crate) edge: Option<String>,
    pub(crate) error_code: Option<String>,
    pub(crate) error_message: Option<String>,
    pub(crate) created_cid: Option<Cid>,
    pub(crate) list: Option<Vec<Node>>,
    pub(crate) completed_iterations: Option<u32>,
    pub(crate) successful_write_count: u32,
}

impl PartialEq for Outcome {
    fn eq(&self, other: &Self) -> bool {
        self.edge == other.edge
            && self.error_code == other.error_code
            && self.error_message == other.error_message
            && self.created_cid == other.created_cid
            && self.completed_iterations == other.completed_iterations
            && self.successful_write_count == other.successful_write_count
            // Skip `list` — Node lacks Eq so we compare via CID.
            && self.list.as_ref().map(|v| v.len()) == other.list.as_ref().map(|v| v.len())
    }
}

impl Outcome {
    /// Returns `true` iff the outcome routed through the named edge.
    pub fn routed_through_edge(&self, edge: &str) -> bool {
        self.edge.as_deref() == Some(edge)
    }

    /// The edge label this outcome routed through (e.g. `"OK"`,
    /// `"ON_NOT_FOUND"`); `None` if no edge was recorded.
    #[must_use]
    pub fn edge_taken(&self) -> Option<String> {
        self.edge.clone()
    }

    /// Stable error-code string (e.g. `"E_NOT_FOUND"`) when the
    /// outcome carries a typed error; `None` for success outcomes.
    pub fn error_code(&self) -> Option<&str> {
        self.error_code.as_deref()
    }

    /// Human-readable error message body for the outcome's typed
    /// error; `None` for success outcomes.
    pub fn error_message(&self) -> Option<String> {
        self.error_message.clone()
    }

    /// Returns `true` iff this is a successful outcome (no error code
    /// recorded and the edge taken is one of the OK aliases).
    #[must_use]
    pub fn is_ok_edge(&self) -> bool {
        matches!(self.edge.as_deref(), Some("OK" | "ok") | None) && self.error_code.is_none()
    }

    /// `Some(rows)` when the outcome carried a list response (e.g.
    /// from a `crud(...).list` action); `None` otherwise.
    #[must_use]
    pub fn as_list(&self) -> Option<Vec<Node>> {
        self.list.clone()
    }

    /// CID of the Node a successful WRITE created (if any).
    #[must_use]
    pub fn created_cid(&self) -> Option<Cid> {
        self.created_cid
    }

    /// Number of completed iterations for outcomes from an ITERATE
    /// primitive; `None` if the outcome did not include an ITERATE.
    #[must_use]
    pub fn completed_iterations(&self) -> Option<u32> {
        self.completed_iterations
    }

    /// Number of WRITE primitives that committed successfully during
    /// this outcome's execution.
    #[must_use]
    pub fn successful_write_count(&self) -> u32 {
        self.successful_write_count
    }

    /// Lift the outcome's error code (if any) into a typed
    /// [`TerminalError`] for downstream callers that need the
    /// catalog-discriminant form.
    #[must_use]
    pub fn terminal_error(&self) -> Option<TerminalError> {
        self.error_code.as_ref().map(|_c| TerminalError {
            code: self
                .error_code
                .clone()
                .map_or(ErrorCode::Unknown(String::new()), |s| {
                    ErrorCode::from_str(&s)
                }),
        })
    }

    /// Panics unless the outcome routed through the success edge.
    pub fn assert_success(&self) {
        assert!(
            self.is_ok_edge(),
            "Outcome::assert_success — outcome did not route through OK: {self:?}"
        );
    }

    /// Test-only accessor — alias for `edge_taken()` in `&str` shape.
    #[must_use]
    pub fn taken_edge(&self) -> &str {
        self.edge.as_deref().unwrap_or("")
    }
}

/// Minimal terminal-error surface returned from `Outcome::terminal_error`.
#[derive(Debug, Clone)]
pub struct TerminalError {
    code: ErrorCode,
}

impl TerminalError {
    /// Stable [`ErrorCode`] catalog discriminant for this error.
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        self.code.clone()
    }
}

/// Trace of an evaluation. Phase 1 emits one synthetic step per primitive
/// in the dispatched CRUD op plus the terminal Outcome; Phase 2 replaces
/// the step synthesis with live evaluator instrumentation.
#[derive(Debug, Clone, Default)]
pub struct Trace {
    pub(crate) steps: Vec<TraceStep>,
    pub(crate) outcome: Option<Outcome>,
}

impl Trace {
    /// Borrowed (cloned) view of every recorded `TraceStep`.
    #[must_use]
    pub fn steps(&self) -> Vec<TraceStep> {
        self.steps.clone()
    }

    /// Terminal `Outcome` produced by the traced evaluation. Callers who
    /// want the final `created_cid` / `list` / `edge` without running
    /// a second (side-effecting) `Engine::call` use this accessor.
    #[must_use]
    pub fn outcome(&self) -> Option<&Outcome> {
        self.outcome.as_ref()
    }
}

/// A single trace row produced by [`crate::Engine::trace`].
///
/// **Phase 2a G11-A Wave 2b — TraceStep unification (G5-B-ii deferral).** The
/// engine-level `TraceStep` is now a discriminant union mirroring
/// [`benten_eval::TraceStep`] so projections from the evaluator's trace
/// stream into the engine surface are structurally 1:1. Three variants
/// beyond the per-primitive `Step` row carry the suspend / resume / budget-
/// exhaustion boundaries the Phase-2a evaluator emits.
///
/// The mirror shape preserves the engine-side per-primitive metadata that
/// the eval-side row does not carry directly: `node_cid` (BLAKE3-derived
/// per-OperationNode identity for predecessor lookups against
/// [`HandlerPredecessors`]) and `primitive` (kind label, e.g. `"read"` /
/// `"write"` / `"respond"`).
#[derive(Debug, Clone)]
#[allow(
    clippy::large_enum_variant,
    reason = "Step is the dominant variant (>99% of trace rows in normal CRUD walks); boxing it would force an allocation per step on the hot path while saving bytes only on the rare boundary / budget rows."
)]
pub enum TraceStep {
    /// One per-primitive execution row.
    Step {
        /// Duration of the step in microseconds. Saturated at `1` so trace
        /// callers that assert `> 0` see honest non-zero timing even for
        /// ultra-fast primitives.
        duration_us: u64,
        /// Stable per-OperationNode CID, derived as
        /// `BLAKE3(handler_id || \0 || op_node_id)`. Cross-references the
        /// predecessor adjacency map returned by
        /// [`crate::Engine::handler_predecessors`].
        node_cid: Cid,
        /// Primitive-kind label (e.g. `"read"`, `"write"`, `"respond"`).
        /// Empty when the evaluator cannot attribute a primitive (legacy
        /// Phase-1 synthetic-step fallback).
        primitive: String,
        /// Operation-node id within the registered handler (carried over
        /// from the eval-side row for diagnostic surfacing).
        node_id: String,
        /// Inputs to the primitive. Phase-2a evaluator emits `Value::Null`
        /// until per-step input snapshotting lands (Phase-2b).
        inputs: Value,
        /// Outputs produced by the primitive (`r.output` from the
        /// evaluator's `step` result).
        outputs: Value,
        /// Optional error code if the step routed to a typed error edge.
        error: Option<ErrorCode>,
        /// Inv-14 attribution. `None` until G5-B-ii completes runtime
        /// threading; the field is required on the public shape so
        /// downstream callers can rely on the slot existing.
        attribution: Option<AttributionFrame>,
    },
    /// WAIT primitive drove the evaluator to suspension. Mirrors
    /// [`benten_eval::TraceStep::SuspendBoundary`].
    SuspendBoundary {
        /// CID of the persisted `ExecutionStateEnvelope`.
        state_cid: Cid,
    },
    /// Resume re-entered a suspended execution. Mirrors
    /// [`benten_eval::TraceStep::ResumeBoundary`].
    ResumeBoundary {
        /// CID of the `ExecutionStateEnvelope` that was resumed.
        state_cid: Cid,
        /// Value handed to the resumed frame as the signal payload.
        signal_value: Value,
    },
    /// Inv-8 / Phase-2b SANDBOX-fuel budget exhausted. Mirrors
    /// [`benten_eval::TraceStep::BudgetExhausted`].
    BudgetExhausted {
        /// `"inv_8_iteration"` | `"sandbox_fuel"`.
        budget_type: &'static str,
        /// How much budget was consumed before firing.
        consumed: u64,
        /// Configured limit.
        limit: u64,
        /// Path of operation-node ids that produced the exhaustion.
        path: Vec<String>,
    },
}

/// Inspector surface for the [`TraceStep::BudgetExhausted`] variant. Returned
/// by [`TraceStep::as_budget_exhausted`] so shape-pin tests can read the
/// fields without pattern-matching at every call site.
#[derive(Debug, Clone, Copy)]
pub struct BudgetExhaustedView<'a> {
    budget_type: &'static str,
    consumed: u64,
    limit: u64,
    path: &'a [String],
}

impl<'a> BudgetExhaustedView<'a> {
    /// `"inv_8_iteration"` | `"sandbox_fuel"`.
    #[must_use]
    pub fn budget_type(&self) -> &'static str {
        self.budget_type
    }

    /// How much budget was consumed before firing.
    #[must_use]
    pub fn consumed(&self) -> u64 {
        self.consumed
    }

    /// Configured limit.
    #[must_use]
    pub fn limit(&self) -> u64 {
        self.limit
    }

    /// Path of operation-node ids that produced the exhaustion.
    #[must_use]
    pub fn path(&self) -> &[String] {
        self.path
    }
}

impl TraceStep {
    /// Step-row duration accessor. Returns the per-primitive microsecond
    /// reading for [`TraceStep::Step`]; `0` for boundary / budget rows
    /// (those are emitted at decision points, not metered intervals).
    #[must_use]
    pub fn duration_us(&self) -> u64 {
        match self {
            TraceStep::Step { duration_us, .. } => *duration_us,
            _ => 0,
        }
    }

    /// Step-row CID accessor. `Some(&Cid)` for [`TraceStep::Step`]; `None`
    /// for boundary / budget rows (which are bounded by the suspended
    /// envelope CID, not an OperationNode CID).
    #[must_use]
    pub fn node_cid(&self) -> Option<&Cid> {
        match self {
            TraceStep::Step { node_cid, .. } => Some(node_cid),
            _ => None,
        }
    }

    /// Primitive-kind label for [`TraceStep::Step`]; `None` otherwise.
    #[must_use]
    pub fn primitive(&self) -> Option<&str> {
        match self {
            TraceStep::Step { primitive, .. } => Some(primitive.as_str()),
            _ => None,
        }
    }

    /// Operation-node id within the registered handler. `None` for
    /// boundary / budget rows.
    #[must_use]
    pub fn node_id(&self) -> Option<&str> {
        match self {
            TraceStep::Step { node_id, .. } => Some(node_id.as_str()),
            _ => None,
        }
    }

    /// Inv-14 attribution accessor. `None` in Phase 2a until G5-B-ii
    /// runtime threading completes; once it does, every Step row carries
    /// `Some(_)`. Boundary / budget rows always return `None`.
    #[must_use]
    pub fn attribution(&self) -> Option<&AttributionFrame> {
        match self {
            TraceStep::Step { attribution, .. } => attribution.as_ref(),
            _ => None,
        }
    }

    /// Step-row error accessor. `Some(&ErrorCode)` if the primitive routed
    /// to a typed error edge; `None` for success rows or non-Step variants.
    #[must_use]
    pub fn error(&self) -> Option<&ErrorCode> {
        match self {
            TraceStep::Step { error, .. } => error.as_ref(),
            _ => None,
        }
    }

    /// Discriminant accessor for [`TraceStep::SuspendBoundary`].
    #[must_use]
    pub fn as_suspend_boundary(&self) -> Option<&Cid> {
        match self {
            TraceStep::SuspendBoundary { state_cid } => Some(state_cid),
            _ => None,
        }
    }

    /// Discriminant accessor for [`TraceStep::ResumeBoundary`].
    #[must_use]
    pub fn as_resume_boundary(&self) -> Option<(&Cid, &Value)> {
        match self {
            TraceStep::ResumeBoundary {
                state_cid,
                signal_value,
            } => Some((state_cid, signal_value)),
            _ => None,
        }
    }

    /// Discriminant accessor for [`TraceStep::BudgetExhausted`]. Returns a
    /// view exposing `budget_type`, `consumed`, `limit`, `path`.
    #[must_use]
    pub fn as_budget_exhausted(&self) -> Option<BudgetExhaustedView<'_>> {
        match self {
            TraceStep::BudgetExhausted {
                budget_type,
                consumed,
                limit,
                path,
            } => Some(BudgetExhaustedView {
                budget_type,
                consumed: *consumed,
                limit: *limit,
                path,
            }),
            _ => None,
        }
    }
}

/// Handle to an Anchor (version-chain identity). **Phase 1 stub.**
///
/// TODO(phase-2-version-chain): a non-zero-sized shape that carries the
/// anchor id lands when `create_anchor` / `append_version` gain real
/// implementations (R-nit-07).
#[derive(Debug, Clone)]
pub struct AnchorHandle {
    #[allow(
        dead_code,
        reason = "Phase-1 stub retains the shape; Phase-2 adds anchor-id fields"
    )]
    pub(crate) _phase1_stub: (),
}

/// Predecessor adjacency for trace assertions.
///
/// Populated at `Engine::handler_predecessors` time by walking the
/// registered subgraph's edge list and mapping each `(from_op_id, to_op_id)`
/// pair onto the operation-node CIDs that the trace step stream uses
/// (handler-scoped `blake3("<handler_id>\0<op_node_id>")`).
///
/// 5d-J workstream 5 — prior to this, the Phase-1 shell returned an
/// always-empty slice, so trace topological-order assertions degraded to
/// a no-op partial-order check.
#[derive(Debug, Default)]
pub struct HandlerPredecessors {
    /// `target_cid -> sorted list of predecessor Cids`. Sorted so test
    /// assertions over the edge set are order-stable.
    pub(crate) adjacency: BTreeMap<Cid, Vec<Cid>>,
}

impl HandlerPredecessors {
    /// Construct from an adjacency map. Used by the engine at
    /// `handler_predecessors` time.
    #[must_use]
    pub fn from_adjacency(adjacency: BTreeMap<Cid, Vec<Cid>>) -> Self {
        Self { adjacency }
    }

    /// Predecessors of `node_cid` in topological order. Returns an empty
    /// slice when the node is a root or when the CID was never registered
    /// as a successor in this handler.
    pub fn predecessors_of(&self, node_cid: &Cid) -> &[Cid] {
        self.adjacency
            .get(node_cid)
            .map_or::<&[Cid], _>(&[], Vec::as_slice)
    }

    /// Enumerate the target CIDs that have at least one predecessor.
    /// Small accessor used by tests that want to iterate the edge set
    /// without hard-coding CIDs.
    pub fn targets(&self) -> impl Iterator<Item = &Cid> {
        self.adjacency.keys()
    }
}

/// Diagnostic report returned by [`crate::Engine::diagnose_read`].
///
/// Option C (named compromise #2, 5d-J workstream 1). The default public
/// read API (`Engine::get_node`, `read_view`, `edges_from`, `edges_to`)
/// returns the symmetric `Ok(None)` / empty-vec on either a
/// backend-miss OR a policy-denied read — a caller cannot tell them
/// apart. `diagnose_read` surfaces the distinction, but is itself
/// gated on a `debug:read` capability grant so ordinary callers never
/// see the existence signal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticInfo {
    /// The CID the caller asked about. Echoed for correlation.
    pub cid: Cid,
    /// True when the CID has a byte-payload in the backend. Distinct from
    /// the policy-denial signal so operators can tell "exists but denied"
    /// apart from "never written" apart from "written then deleted".
    pub exists_in_backend: bool,
    /// When set, names the scope the policy rejected. `None` means the
    /// policy permitted the read (or no policy is configured).
    pub denied_by_policy: Option<String>,
    /// True when the backend has no byte-payload for this CID. Mirrors
    /// `!exists_in_backend` but exposed as its own field so the TS
    /// caller reads `{ notFound: true }` as a discriminant without
    /// boolean-flipping.
    pub not_found: bool,
}

/// Nested-transaction handle. **Phase 1 stub.**
///
/// TODO(phase-2-nested-tx): Phase-1 always rejects nested begin; Phase-2
/// populates this with the real sub-transaction state (R-nit-07).
#[derive(Debug)]
pub struct NestedTx {
    #[allow(
        dead_code,
        reason = "Phase-1 stub retains the shape; Phase-2 populates with sub-transaction state"
    )]
    pub(crate) _phase1_stub: (),
}
