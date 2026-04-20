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

use benten_core::{Cid, Node};
use benten_errors::ErrorCode;

/// Options passed to `Engine::create_view`. Currently a placeholder shape so
/// `Default::default()` resolves unambiguously at the call site.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ViewCreateOptions;

/// Options passed to `Engine::read_view_with`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadViewOptions {
    pub allow_stale: bool,
}

impl ReadViewOptions {
    #[must_use]
    pub fn strict() -> Self {
        Self { allow_stale: false }
    }

    #[must_use]
    pub fn allow_stale() -> Self {
        Self { allow_stale: true }
    }
}

/// Extension trait used by `tx_atomicity` integration tests.
pub trait OutcomeExt {
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
    pub fn routed_through_edge(&self, edge: &str) -> bool {
        self.edge.as_deref() == Some(edge)
    }

    #[must_use]
    pub fn edge_taken(&self) -> Option<String> {
        self.edge.clone()
    }

    pub fn error_code(&self) -> Option<&str> {
        self.error_code.as_deref()
    }

    pub fn error_message(&self) -> Option<String> {
        self.error_message.clone()
    }

    #[must_use]
    pub fn is_ok_edge(&self) -> bool {
        matches!(self.edge.as_deref(), Some("OK" | "ok") | None) && self.error_code.is_none()
    }

    #[must_use]
    pub fn as_list(&self) -> Option<Vec<Node>> {
        self.list.clone()
    }

    #[must_use]
    pub fn created_cid(&self) -> Option<Cid> {
        self.created_cid.clone()
    }

    #[must_use]
    pub fn completed_iterations(&self) -> Option<u32> {
        self.completed_iterations
    }

    #[must_use]
    pub fn successful_write_count(&self) -> u32 {
        self.successful_write_count
    }

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

#[derive(Debug, Clone)]
pub struct TraceStep {
    pub(crate) duration_us: u64,
    pub(crate) node_cid: Cid,
    pub(crate) primitive: String,
}

impl TraceStep {
    #[must_use]
    pub fn duration_us(&self) -> u64 {
        self.duration_us
    }

    #[must_use]
    pub fn node_cid(&self) -> &Cid {
        &self.node_cid
    }

    /// Primitive-kind label for the step (e.g. `"read"`, `"write"`,
    /// `"respond"`). Empty when the Phase-1 synthetic step cannot
    /// attribute a primitive.
    #[must_use]
    pub fn primitive(&self) -> &str {
        &self.primitive
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

/// Predecessor adjacency for trace assertions. **Phase 1 stub.**
///
/// TODO(phase-2-trace): carries a `BTreeMap<Cid, Vec<Cid>>` when the
/// evaluator surfaces real predecessor metadata (R-nit-07).
#[derive(Debug, Default)]
pub struct HandlerPredecessors {
    #[allow(
        dead_code,
        reason = "Phase-1 stub retains the shape; Phase-2 populates predecessor adjacency"
    )]
    pub(crate) _phase1_stub: (),
}

impl HandlerPredecessors {
    pub fn predecessors_of(&self, _node_cid: &Cid) -> &[Cid] {
        &[]
    }
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
