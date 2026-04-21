//! # benten-errors
//!
//! Stable error-catalog discriminants for the Benten graph engine.
//!
//! This crate sits at the root of the workspace dependency graph: it has zero
//! dependencies on other Benten crates. Every other crate (`benten-core`,
//! `benten-graph`, `benten-caps`, `benten-ivm`, `benten-eval`, `benten-engine`,
//! the napi bindings) imports [`ErrorCode`] from here and maps its own
//! error variants to the stable catalog codes via a `.code()` accessor.
//!
//! Extracted from `benten-core::error_code` in Phase 1 (closes SECURITY-POSTURE
//! compromise #3) so the catalog enum no longer forces a `benten-core` edge on
//! any crate that only needs the stable string identifiers.
//!
//! ## Stability contract
//!
//! The string forms returned by [`ErrorCode::as_str`] (`"E_VALUE_FLOAT_NAN"`,
//! `"E_CAP_DENIED"`, …) are **frozen**. Drift between this enum and
//! `docs/ERROR-CATALOG.md` is detected by the G8 drift lint
//! (`scripts/drift-detect.ts`).
//!
//! Adding a variant requires:
//! 1. Append a `match` arm in [`ErrorCode::as_str`], [`ErrorCode::as_static_str`],
//!    and [`ErrorCode::from_str`].
//! 2. Reserve the code in the catalog doc.
//! 3. Update any `.code()` mapper in the owning crate that may produce it.
//!
//! [`ErrorCode::from_str`] round-trips [`ErrorCode::as_str`] for every known
//! variant and returns [`ErrorCode::Unknown`] for unrecognized codes so a
//! future server emitting a newer code doesn't crash an older client.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![no_std]

extern crate alloc;

use alloc::string::{String, ToString};

/// Stable error-catalog discriminants.
///
/// The set mirrors `docs/ERROR-CATALOG.md`. See the crate-level docs for the
/// adding-a-variant checklist.
///
/// `#[non_exhaustive]` (R6b bp-17) so downstream consumers must include a
/// fallback `_ =>` arm — adding a new catalog code in a later phase is a
/// minor version bump rather than a breaking change. The existing
/// `ErrorCode::Unknown(String)` variant covers forward-compat on the
/// parse-side; `non_exhaustive` covers forward-compat on the match-side.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorCode {
    /// Registration-time: subgraph contains a cycle (invariant 1 violation).
    InvCycle,
    /// Registration-time: subgraph exceeds configured max depth (invariant 2).
    InvDepthExceeded,
    /// Registration-time: a node exceeds max fan-out (invariant 3).
    InvFanoutExceeded,
    /// Registration-time: subgraph exceeds max total node count (invariant 5).
    InvTooManyNodes,
    /// Registration-time: subgraph exceeds max total edge count (invariant 6).
    InvTooManyEdges,
    /// Registration-time: determinism classification conflict (invariant 9).
    InvDeterminism,
    /// Registration-time: computed content hash mismatch (invariant 10).
    InvContentHash,
    /// Registration-time catch-all for invariants lacking a distinct code.
    InvRegistration,
    /// Registration-time: nested `ITERATE` deeper than the policy allows
    /// (a Phase-1 stopgap for invariant 8's nesting aspect).
    InvIterateNestDepth,
    /// Registration-time: `ITERATE` node is missing its required `max` bound.
    InvIterateMaxMissing,
    /// Runtime cumulative-iteration-budget exhaustion.
    ///
    /// Phase 1 surfaces this when the iterative evaluator's per-run step
    /// counter reaches the default iteration budget. Distinct from
    /// [`ErrorCode::InvIterateNestDepth`] (registration-time stopgap for
    /// invariant 8's nesting-depth aspect). Phase 2 replaces the scalar
    /// budget with multiplicative-through-CALL accounting but keeps this
    /// same code.
    InvIterateBudget,
    /// Capability policy denied a write (generic `E_CAP_DENIED`).
    CapDenied,
    /// Capability policy denied a READ (option-A existence leak, see
    /// SECURITY-POSTURE.md §Compromise 2).
    CapDeniedRead,
    /// Phase 3 sync revocation code (distinct from `CapRevokedMidEval`).
    CapRevoked,
    /// Mid-evaluation revocation surfaced at an ITERATE batch boundary.
    CapRevokedMidEval,
    /// Capability backend returned `NotImplemented` (e.g. UCAN stub in Phase 1).
    CapNotImplemented,
    /// Capability-chain attenuation failed across a CALL (outer grant does not
    /// subsume callee requires).
    CapAttenuation,
    /// Optimistic-concurrency write conflict at commit.
    WriteConflict,
    /// Consumer read a View whose maintenance lag exceeds the freshness bound.
    IvmViewStale,
    /// Transaction aborted (explicit rollback, or closure returned `Err`).
    TxAborted,
    /// Nested transactions are not supported (redb constraint).
    NestedTransactionNotSupported,
    /// Primitive not implemented at the evaluator (WAIT / STREAM / SUBSCRIBE /
    /// SANDBOX in Phase 1).
    PrimitiveNotImplemented,
    /// A user subgraph attempted to write a system-zone-labelled Node.
    SystemZoneWrite,
    /// `Value::Float` rejected a NaN payload at canonical-bytes time.
    ValueFloatNan,
    /// `Value::Float` rejected a `±Infinity` payload at canonical-bytes time.
    ValueFloatNonFinite,
    /// Failed to parse a CID string (multicodec / multihash / base32).
    CidParse,
    /// CID uses a multicodec Benten does not support (non-`dag-cbor`).
    CidUnsupportedCodec,
    /// CID uses a multihash Benten does not support (non-BLAKE3).
    CidUnsupportedHash,
    /// Version chain append detected a concurrent fork.
    VersionBranched,
    /// Backend lookup miss (storage-layer not-found distinct from `NotFound`).
    BackendNotFound,
    /// TRANSFORM expression failed to parse.
    TransformSyntax,
    /// Input exceeded a configured size limit (property count, byte cap, …).
    InputLimit,
    /// Generic not-found (version-chain anchor miss, etc.).
    NotFound,
    /// Storage-layer internal failure (redb I/O, decode). Stable code for
    /// the `GraphError::Redb` / `GraphError::Decode` variants. Replaces the
    /// prior `Unknown("graph_internal")` so consumers can match on a typed
    /// catalog code rather than a lowercase ad-hoc string.
    ///
    /// Phase 2 will refactor `GraphError::Redb(String)` into a
    /// `#[source]`-preserving `redb::Error` chain; this code survives the
    /// refactor.
    GraphInternal,
    /// DAG-CBOR serialization failure at the hash path (e.g. encoder
    /// integer-overflow). Distinct from the catalog's registration-time
    /// invariants; the payload is a human-readable message held on the
    /// corresponding `CoreError::Serialize` variant.
    Serialize,
    /// Handler id already registered with different content (engine-layer).
    DuplicateHandler,
    /// `Engine::builder().production()` invoked without an explicit capability
    /// policy (R1 SC2 fail-early guardrail).
    NoCapabilityPolicyConfigured,
    /// `.production()` combined with `.without_caps()` — mutually exclusive.
    ProductionRequiresCaps,
    /// Operation requires an IVM or capability subsystem that was disabled
    /// at builder time (`.without_ivm()` / `.without_caps()` thinness paths).
    SubsystemDisabled,
    /// Read against a view id that was never registered.
    UnknownView,
    /// Feature deferred to a future group / phase. Used for surfaces that
    /// depend on evaluator integration not yet wired in Phase 1.
    NotImplemented,
    /// IVM view was queried with a filter pattern the view does not maintain.
    /// Runtime-query-shape error distinct from `E_INV_REGISTRATION`.
    IvmPatternMismatch,
    /// Caller-supplied prior head was never observed by the version anchor.
    /// Surfaces from the prior-head-threaded `benten_core::version::append_version`.
    VersionUnknownPrior,
    /// Fallback for drift detector — holds the unknown raw string so it can
    /// be rendered without lossy conversion.
    Unknown(String),
}

impl ErrorCode {
    /// Return the stable string identifier (e.g. `"E_INV_CYCLE"`).
    ///
    /// For [`ErrorCode::Unknown`] the stored string is returned verbatim;
    /// every known variant delegates through [`ErrorCode::as_static_str`]
    /// so the 44-arm catalog mapping lives in exactly one place
    /// (5d-K triple-match dedup). `&'static str` coerces to the shorter
    /// `&self`-bound `&str` without runtime cost.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            ErrorCode::Unknown(s) => s.as_str(),
            known => known.as_static_str(),
        }
    }

    /// Return the stable string identifier with a `'static` lifetime when
    /// the variant is a known catalog code; returns `"E_UNKNOWN"` for the
    /// forward-compat [`ErrorCode::Unknown`] variant because its payload is
    /// an owned `String` and cannot be promoted to `'static` without
    /// leaking.
    ///
    /// This is the single source of truth the engine's
    /// `EngineError::code()` delegates through, replacing the former
    /// `static_for` match duplicate (r6-err-10).
    #[must_use]
    pub fn as_static_str(&self) -> &'static str {
        match self {
            ErrorCode::InvCycle => "E_INV_CYCLE",
            ErrorCode::InvDepthExceeded => "E_INV_DEPTH_EXCEEDED",
            ErrorCode::InvFanoutExceeded => "E_INV_FANOUT_EXCEEDED",
            ErrorCode::InvTooManyNodes => "E_INV_TOO_MANY_NODES",
            ErrorCode::InvTooManyEdges => "E_INV_TOO_MANY_EDGES",
            ErrorCode::InvDeterminism => "E_INV_DETERMINISM",
            ErrorCode::InvContentHash => "E_INV_CONTENT_HASH",
            ErrorCode::InvRegistration => "E_INV_REGISTRATION",
            ErrorCode::InvIterateNestDepth => "E_INV_ITERATE_NEST_DEPTH",
            ErrorCode::InvIterateMaxMissing => "E_INV_ITERATE_MAX_MISSING",
            ErrorCode::InvIterateBudget => "E_INV_ITERATE_BUDGET",
            ErrorCode::CapDenied => "E_CAP_DENIED",
            ErrorCode::CapDeniedRead => "E_CAP_DENIED_READ",
            ErrorCode::CapRevoked => "E_CAP_REVOKED",
            ErrorCode::CapRevokedMidEval => "E_CAP_REVOKED_MID_EVAL",
            ErrorCode::CapNotImplemented => "E_CAP_NOT_IMPLEMENTED",
            ErrorCode::CapAttenuation => "E_CAP_ATTENUATION",
            ErrorCode::WriteConflict => "E_WRITE_CONFLICT",
            ErrorCode::IvmViewStale => "E_IVM_VIEW_STALE",
            ErrorCode::TxAborted => "E_TX_ABORTED",
            ErrorCode::NestedTransactionNotSupported => "E_NESTED_TRANSACTION_NOT_SUPPORTED",
            ErrorCode::PrimitiveNotImplemented => "E_PRIMITIVE_NOT_IMPLEMENTED",
            ErrorCode::SystemZoneWrite => "E_SYSTEM_ZONE_WRITE",
            ErrorCode::ValueFloatNan => "E_VALUE_FLOAT_NAN",
            ErrorCode::ValueFloatNonFinite => "E_VALUE_FLOAT_NONFINITE",
            ErrorCode::CidParse => "E_CID_PARSE",
            ErrorCode::CidUnsupportedCodec => "E_CID_UNSUPPORTED_CODEC",
            ErrorCode::CidUnsupportedHash => "E_CID_UNSUPPORTED_HASH",
            ErrorCode::VersionBranched => "E_VERSION_BRANCHED",
            ErrorCode::BackendNotFound => "E_BACKEND_NOT_FOUND",
            ErrorCode::TransformSyntax => "E_TRANSFORM_SYNTAX",
            ErrorCode::InputLimit => "E_INPUT_LIMIT",
            ErrorCode::NotFound => "E_NOT_FOUND",
            ErrorCode::Serialize => "E_SERIALIZE",
            ErrorCode::GraphInternal => "E_GRAPH_INTERNAL",
            ErrorCode::DuplicateHandler => "E_DUPLICATE_HANDLER",
            ErrorCode::NoCapabilityPolicyConfigured => "E_NO_CAPABILITY_POLICY_CONFIGURED",
            ErrorCode::ProductionRequiresCaps => "E_PRODUCTION_REQUIRES_CAPS",
            ErrorCode::SubsystemDisabled => "E_SUBSYSTEM_DISABLED",
            ErrorCode::UnknownView => "E_UNKNOWN_VIEW",
            ErrorCode::NotImplemented => "E_NOT_IMPLEMENTED",
            ErrorCode::IvmPatternMismatch => "E_IVM_PATTERN_MISMATCH",
            ErrorCode::VersionUnknownPrior => "E_VERSION_UNKNOWN_PRIOR",
            ErrorCode::Unknown(_) => "E_UNKNOWN",
        }
    }

    /// Parse a stable catalog code string into an [`ErrorCode`], falling back
    /// to [`ErrorCode::Unknown`] with the raw string preserved so forward-
    /// compatible deserialization never panics.
    #[must_use]
    pub fn from_str(s: &str) -> ErrorCode {
        match s {
            "E_INV_CYCLE" => ErrorCode::InvCycle,
            "E_INV_DEPTH_EXCEEDED" => ErrorCode::InvDepthExceeded,
            "E_INV_FANOUT_EXCEEDED" => ErrorCode::InvFanoutExceeded,
            "E_INV_TOO_MANY_NODES" => ErrorCode::InvTooManyNodes,
            "E_INV_TOO_MANY_EDGES" => ErrorCode::InvTooManyEdges,
            "E_INV_DETERMINISM" => ErrorCode::InvDeterminism,
            "E_INV_CONTENT_HASH" => ErrorCode::InvContentHash,
            "E_INV_REGISTRATION" => ErrorCode::InvRegistration,
            "E_INV_ITERATE_NEST_DEPTH" => ErrorCode::InvIterateNestDepth,
            "E_INV_ITERATE_MAX_MISSING" => ErrorCode::InvIterateMaxMissing,
            "E_INV_ITERATE_BUDGET" => ErrorCode::InvIterateBudget,
            "E_CAP_DENIED" => ErrorCode::CapDenied,
            "E_CAP_DENIED_READ" => ErrorCode::CapDeniedRead,
            "E_CAP_REVOKED" => ErrorCode::CapRevoked,
            "E_CAP_REVOKED_MID_EVAL" => ErrorCode::CapRevokedMidEval,
            "E_CAP_NOT_IMPLEMENTED" => ErrorCode::CapNotImplemented,
            "E_CAP_ATTENUATION" => ErrorCode::CapAttenuation,
            "E_WRITE_CONFLICT" => ErrorCode::WriteConflict,
            "E_IVM_VIEW_STALE" => ErrorCode::IvmViewStale,
            "E_TX_ABORTED" => ErrorCode::TxAborted,
            "E_NESTED_TRANSACTION_NOT_SUPPORTED" => ErrorCode::NestedTransactionNotSupported,
            "E_PRIMITIVE_NOT_IMPLEMENTED" => ErrorCode::PrimitiveNotImplemented,
            "E_SYSTEM_ZONE_WRITE" => ErrorCode::SystemZoneWrite,
            "E_VALUE_FLOAT_NAN" => ErrorCode::ValueFloatNan,
            "E_VALUE_FLOAT_NONFINITE" => ErrorCode::ValueFloatNonFinite,
            "E_CID_PARSE" => ErrorCode::CidParse,
            "E_CID_UNSUPPORTED_CODEC" => ErrorCode::CidUnsupportedCodec,
            "E_CID_UNSUPPORTED_HASH" => ErrorCode::CidUnsupportedHash,
            "E_VERSION_BRANCHED" => ErrorCode::VersionBranched,
            "E_BACKEND_NOT_FOUND" => ErrorCode::BackendNotFound,
            "E_TRANSFORM_SYNTAX" => ErrorCode::TransformSyntax,
            "E_INPUT_LIMIT" => ErrorCode::InputLimit,
            "E_NOT_FOUND" => ErrorCode::NotFound,
            "E_SERIALIZE" => ErrorCode::Serialize,
            "E_GRAPH_INTERNAL" => ErrorCode::GraphInternal,
            "E_DUPLICATE_HANDLER" => ErrorCode::DuplicateHandler,
            "E_NO_CAPABILITY_POLICY_CONFIGURED" => ErrorCode::NoCapabilityPolicyConfigured,
            "E_PRODUCTION_REQUIRES_CAPS" => ErrorCode::ProductionRequiresCaps,
            "E_SUBSYSTEM_DISABLED" => ErrorCode::SubsystemDisabled,
            "E_UNKNOWN_VIEW" => ErrorCode::UnknownView,
            "E_NOT_IMPLEMENTED" => ErrorCode::NotImplemented,
            "E_IVM_PATTERN_MISMATCH" => ErrorCode::IvmPatternMismatch,
            "E_VERSION_UNKNOWN_PRIOR" => ErrorCode::VersionUnknownPrior,
            other => ErrorCode::Unknown(other.to_string()),
        }
    }
}
