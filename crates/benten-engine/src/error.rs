//! `EngineError` and its conversions.
//!
//! Extracted from `lib.rs` by R6 Wave 2 (R-major-01) so the top-level
//! orchestrator crate reads top-to-bottom (builder → engine → primitive host
//! → supporting types). The `EngineError` shape is unchanged; only the
//! file it lives in moved.

use benten_caps::CapError;
use benten_core::CoreError;
pub use benten_errors::ErrorCode;
use benten_eval::RegistrationError;
use benten_graph::GraphError;

/// Errors produced by the engine orchestrator.
///
/// `#[non_exhaustive]` (R6b bp-17) — engine error variants will grow as
/// Phase 2 primitives land (STREAM back-pressure, WAIT timeouts, SANDBOX
/// fuel exhaustion); downstream matchers must include `_ =>` so adding
/// variants is a minor version bump.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum EngineError {
    #[error("core: {0}")]
    Core(#[from] CoreError),

    #[error("graph: {0}")]
    Graph(#[from] GraphError),

    #[error("capability: {0}")]
    Cap(#[from] CapError),

    /// Structural-invariant rejection. Boxed so `Result<T, EngineError>`
    /// stays below clippy's `result_large_err` 128-byte threshold —
    /// `RegistrationError` itself carries ~360 bytes of diagnostic context
    /// (paths, expected/actual CIDs, per-invariant counts). Mini-review
    /// findings `g6-cr-1` / `g6-cag-7`.
    ///
    /// R6FP catch-up EH4: `#[from]` enables `?` auto-conversion from
    /// `Box<RegistrationError>` and (per thiserror semantics) automatically
    /// participates in the standard `std::error::Error::source()` chain so
    /// `anyhow` / `eyre` / log renderers see the underlying invariant
    /// context. Format flipped from `{0:?}` Debug to `{0}` Display now that
    /// `RegistrationError` impls Display (one-line catalog code +
    /// first-available diagnostic field).
    #[error("invariant: {0}")]
    Invariant(#[from] Box<RegistrationError>),

    /// Handler ID already registered with different content.
    #[error("duplicate handler: {handler_id}")]
    DuplicateHandler { handler_id: String },

    /// `Engine::builder().production()` called without an explicit
    /// capability policy. R1 SC2: fail-early guardrail.
    #[error(
        "no capability policy configured for .production() builder — call .capability_policy(...) or drop .production()"
    )]
    NoCapabilityPolicyConfigured,

    /// `.production()` combined with `.without_caps()` — mutually exclusive.
    /// Production mode requires a real capability policy; `.without_caps()`
    /// explicitly tears one down. Picking both silently dropped the policy
    /// before this guard — see code-reviewer finding `g7-cr-1`.
    #[error(
        "production mode requires capabilities — .production() and .without_caps() are mutually exclusive"
    )]
    ProductionRequiresCaps,

    /// Thin engine (without_ivm or without_caps) was asked to do something
    /// that requires the disabled subsystem. The honest-no boundary — thinness
    /// tests assert we error here rather than silently no-op.
    #[error("subsystem disabled: {subsystem}")]
    SubsystemDisabled { subsystem: &'static str },

    /// Read against a view whose incremental state is stale.
    #[error("IVM view stale: {view_id}")]
    IvmViewStale { view_id: String },

    /// Read against a view id that was never registered.
    #[error("unknown view: {view_id}")]
    UnknownView { view_id: String },

    /// Nested transaction attempted.
    #[error("nested transaction not supported")]
    NestedTransactionNotSupported,

    /// Feature deferred to a future group / phase. Used for primitive
    /// dispatch surfaces (`register_crud`, `call`, `trace`, `*` version
    /// chain, `*` principals) that need the evaluator integration the
    /// present G7 does not land.
    #[error("not implemented in Phase 1: {feature}")]
    NotImplemented { feature: &'static str },

    /// Generic wrapped error carrying a stable catalog code.
    #[error("{message}")]
    Other { code: ErrorCode, message: String },
}

impl EngineError {
    /// Stable catalog code as [`ErrorCode`]. Consumers that want the string
    /// form call `err.error_code().as_str()`.
    #[must_use]
    pub fn error_code(&self) -> ErrorCode {
        match self {
            EngineError::Core(e) => e.code(),
            EngineError::Graph(e) => e.code(),
            EngineError::Cap(e) => e.code(),
            EngineError::Invariant(e) => e.code(),
            // r6-err-4: promoted from `ErrorCode::Unknown(...)` strings to
            // first-class catalog variants so the drift detector and TS
            // codegen see them via the catalog path.
            EngineError::DuplicateHandler { .. } => ErrorCode::DuplicateHandler,
            EngineError::NoCapabilityPolicyConfigured => ErrorCode::NoCapabilityPolicyConfigured,
            EngineError::ProductionRequiresCaps => ErrorCode::ProductionRequiresCaps,
            EngineError::SubsystemDisabled { .. } => ErrorCode::SubsystemDisabled,
            EngineError::IvmViewStale { .. } => ErrorCode::IvmViewStale,
            EngineError::UnknownView { .. } => ErrorCode::UnknownView,
            EngineError::NestedTransactionNotSupported => ErrorCode::NestedTransactionNotSupported,
            EngineError::NotImplemented { .. } => ErrorCode::NotImplemented,
            EngineError::Other { code, .. } => code.clone(),
        }
    }

    /// Stable catalog code as [`ErrorCode`]. Phase 2a consolidation: the
    /// return type changed from `&'static str` to `ErrorCode` so Phase-2a
    /// tests can pattern-match and/or compare equal against `ErrorCode`
    /// variants directly. For the string form, call `.code().as_str()` or
    /// use the legacy [`Self::code_as_str`] helper.
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        self.error_code()
    }

    /// Legacy `'static str` code accessor retained for call sites that
    /// serialise the code directly (napi wire, drift detector).
    #[must_use]
    pub fn code_as_str(&self) -> &'static str {
        self.error_code().as_static_str()
    }

    /// Phase 2a dx-r1 (addendum): edge-label the error routes through.
    #[must_use]
    pub fn routed_edge_label(&self) -> Option<&'static str> {
        self.error_code().routed_edge_label()
    }
}
