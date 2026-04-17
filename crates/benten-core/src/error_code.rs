//! Stable error-catalog discriminants ([`ErrorCode`]) + the
//! [`CoreError::code`](crate::CoreError::code) mapping.
//!
//! Every [`CoreError`], `GraphError`, `CapError`, and
//! `EngineError` variant maps to one of these via a `.code()` method so the TS
//! layer sees a stable identifier on every error. The string forms
//! (`"E_VALUE_FLOAT_NAN"` etc.) are frozen — drift between this enum and
//! `docs/ERROR-CATALOG.md` is detected by a G8 lint.
//!
//! [`ErrorCode::from_str`] round-trips [`ErrorCode::as_str`] for every known
//! variant and returns [`ErrorCode::Unknown`] for unrecognized codes so a
//! future server emitting a newer code doesn't crash an older client.

use alloc::string::{String, ToString};

use crate::CoreError;

/// Stable error-catalog discriminants.
///
/// The set mirrors `docs/ERROR-CATALOG.md`. Adding a variant requires:
/// 1. Append a `match` arm in both [`ErrorCode::as_str`] and
///    [`ErrorCode::from_str`].
/// 2. Reserve the code in the catalog doc.
/// 3. Update any [`CoreError::code`](crate::CoreError::code)-style mappers that
///    may produce it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCode {
    InvCycle,
    InvDepthExceeded,
    InvFanoutExceeded,
    InvTooManyNodes,
    InvTooManyEdges,
    InvDeterminism,
    InvContentHash,
    InvRegistration,
    InvIterateNestDepth,
    InvIterateMaxMissing,
    CapDenied,
    CapDeniedRead,
    /// Phase 3 sync revocation code (distinct from `CapRevokedMidEval`).
    CapRevoked,
    CapRevokedMidEval,
    CapNotImplemented,
    CapAttenuation,
    WriteConflict,
    IvmViewStale,
    TxAborted,
    NestedTransactionNotSupported,
    PrimitiveNotImplemented,
    SystemZoneWrite,
    ValueFloatNan,
    ValueFloatNonFinite,
    CidParse,
    CidUnsupportedCodec,
    CidUnsupportedHash,
    VersionBranched,
    BackendNotFound,
    TransformSyntax,
    InputLimit,
    /// Generic not-found (version-chain anchor miss, etc.).
    NotFound,
    /// DAG-CBOR serialization failure at the hash path (e.g. encoder
    /// integer-overflow). Distinct from the catalog's registration-time
    /// invariants; the payload is a human-readable message held on the
    /// corresponding [`CoreError::Serialize`] variant.
    Serialize,
    /// Fallback for drift detector — holds the unknown raw string so it can
    /// be rendered without lossy conversion.
    Unknown(String),
}

impl ErrorCode {
    /// Return the stable string identifier (e.g. `"E_INV_CYCLE"`).
    ///
    /// For [`ErrorCode::Unknown`] the stored string is returned verbatim.
    #[must_use]
    pub fn as_str(&self) -> &str {
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
            ErrorCode::Unknown(s) => s.as_str(),
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
            other => ErrorCode::Unknown(other.to_string()),
        }
    }
}

impl CoreError {
    /// Map this [`CoreError`] variant to its stable ERROR-CATALOG code.
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        match self {
            CoreError::FloatNan => ErrorCode::ValueFloatNan,
            CoreError::FloatNonFinite => ErrorCode::ValueFloatNonFinite,
            CoreError::CidParse(_) | CoreError::InvalidCid(_) => ErrorCode::CidParse,
            CoreError::CidUnsupportedCodec => ErrorCode::CidUnsupportedCodec,
            CoreError::CidUnsupportedHash => ErrorCode::CidUnsupportedHash,
            CoreError::VersionBranched => ErrorCode::VersionBranched,
            CoreError::Serialize(_) => ErrorCode::Serialize,
            CoreError::NotFound => ErrorCode::NotFound,
        }
    }
}
