//! # benten-core
//!
//! Core types and content-addressed hashing for the Benten graph engine.
//!
//! This crate is deliberately minimal for the Phase 1 stack spike. It defines:
//!
//! - [`Value`] — the graph value type. The spike covers `null`, `bool`, `int`,
//!   `text`, `bytes`, `list`, and `map`. A `Float(f64)` variant is intentionally
//!   deferred to Phase 1 proper (it needs NaN rejection, shortest-form encoding,
//!   and a dedicated proptest before it can enter the hash path; see
//!   [`SPIKE-phase-1-stack-RESULTS.md`](../../../SPIKE-phase-1-stack-RESULTS.md)
//!   critic triage).
//! - [`Node`] — a content-addressed graph Node (label list + ordered property map).
//! - [`Cid`] — a CIDv1 newtype (multicodec `0x71` dag-cbor, multihash `0x1e` blake3)
//!   produced by [`Node::cid`].
//!
//! Version-chain primitives from ENGINE-SPEC §6 (Anchor / Version Node /
//! `CURRENT` / `NEXT_VERSION` edges) are **not** implemented in the spike. The
//! `Node::anchor_id` field is a placeholder for future use; the Anchor type,
//! edge labels, and version-walking helpers will land in Phase 1 proper. The
//! `benten-core` crate already excludes `anchor_id` from the content hash, so
//! a future Anchor wrapper can attach version chains without disturbing the
//! hash invariant.
//!
//! ## What gets hashed
//!
//! Per [`docs/ENGINE-SPEC.md`](../../../docs/ENGINE-SPEC.md) Section 7, the CID is
//! computed over **labels and properties only**. Anchor IDs, timestamps, and
//! edges are explicitly excluded. This is the non-negotiable invariant the spike
//! validates: the same labels + properties on two different machines or runs
//! produce the same CID.
//!
//! ## Determinism guarantees
//!
//! 1. The load-bearing guarantee is `serde_ipld_dagcbor`'s encode-time
//!    canonicalization: it emits DAG-CBOR canonical form with RFC 7049
//!    length-first key sort regardless of the source type's iteration order.
//!    [`Value::Map`]'s [`alloc::collections::BTreeMap`] backing is a
//!    belt-and-suspenders defense; the on-wire bytes (and therefore the CID)
//!    are determined by the CBOR canonicalization, not by the map's iteration
//!    order.
//! 2. The CID encoding (version byte `0x01`, multicodec `0x71`, multihash
//!    `0x1e` + length `0x20` + 32-byte digest) is fixed by this crate and is
//!    wire-compatible with the IPLD CIDv1 spec.
//!
//! The spike validates these properties in three ways:
//! - **D1 intra-process** — unit test in this crate
//! - **D2 cross-process** — integration test against a committed fixture
//! - **D3 wasm32-unknown-unknown** — `cargo check` in CI (compile-check only;
//!   a runtime WASM check against the fixture is a Phase 1 CI follow-up)

#![forbid(unsafe_code)]
#![no_std]
#![allow(clippy::todo, reason = "R3 red-phase stubs; R5 removes todos")]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Multicodec / multihash constants
// ---------------------------------------------------------------------------

/// IPLD CIDv1 version byte.
pub const CID_V1: u8 = 0x01;

/// Multicodec code for `dag-cbor` (per the multicodec table).
pub const MULTICODEC_DAG_CBOR: u8 = 0x71;

/// Multihash code for BLAKE3 256-bit (per the multicodec / multihash tables).
pub const MULTIHASH_BLAKE3: u8 = 0x1e;

/// Length in bytes of a BLAKE3 256-bit digest.
pub const BLAKE3_DIGEST_LEN: u8 = 32;

/// Total length in bytes of a Benten CIDv1 (BLAKE3 + dag-cbor):
/// version (1) + codec (1) + multihash-code (1) + digest-length (1) + digest (32).
pub const CID_LEN: usize = 1 + 1 + 1 + 1 + BLAKE3_DIGEST_LEN as usize;

// ---------------------------------------------------------------------------
// Value
// ---------------------------------------------------------------------------

/// A graph Value. This is the subset of DAG-CBOR we expose in the spike.
///
/// Maps use [`BTreeMap`] so in-memory iteration order is deterministic;
/// the on-wire canonical form is separately enforced by `serde_ipld_dagcbor`
/// at encode time (DAG-CBOR length-first key sort).
///
/// `#[serde(untagged)]` is safe here because DAG-CBOR's major-type tagging
/// makes each variant's wire encoding unambiguous: a boolean cannot
/// deserialize as an integer because CBOR major type 7 (simple) and major
/// types 0/1 (unsigned/negative integer) are distinct. A round-trip proptest
/// (hash → decode → hash → assert equal CID) is a Phase 1 follow-up that
/// will validate this assumption empirically; see
/// [`SPIKE-phase-1-stack-RESULTS.md`](../../../SPIKE-phase-1-stack-RESULTS.md)
/// critic triage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    /// CBOR null.
    Null,
    /// CBOR boolean.
    Bool(bool),
    /// CBOR signed integer (-2^63 .. 2^63-1).
    Int(i64),
    /// CBOR 64-bit float. NaN and ±Infinity must be rejected at serialization
    /// time (see [`CoreError::FloatNan`] / [`CoreError::FloatNonFinite`]).
    ///
    /// **Phase 1 G1-A stub** — the concrete encoding contract and the
    /// NaN/Inf rejection path ship in Phase 1 proper. Today this variant is
    /// reachable but unused.
    Float(f64),
    /// CBOR text string (UTF-8).
    Text(String),
    /// CBOR byte string.
    Bytes(Vec<u8>),
    /// CBOR array.
    List(Vec<Value>),
    /// CBOR map with text keys (DAG-CBOR restricts map keys to strings).
    Map(BTreeMap<String, Value>),
}

impl Value {
    /// Convenience constructor for text values.
    pub fn text(s: impl Into<String>) -> Self {
        Value::Text(s.into())
    }
}

// ---------------------------------------------------------------------------
// Node
// ---------------------------------------------------------------------------

/// A graph Node. Content-addressed: the CID is derived purely from `labels`
/// and `properties`. The optional `anchor_id` is stored alongside but is
/// explicitly excluded from the hash (per `ENGINE-SPEC.md` Section 7), because
/// external edges point to anchors while content hashes must remain stable
/// across renames.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    /// Zero or more labels classifying the Node (e.g., `["Post"]`,
    /// `["User", "Admin"]`).
    pub labels: Vec<String>,

    /// Ordered property map. `BTreeMap` guarantees deterministic iteration.
    pub properties: BTreeMap<String, Value>,

    /// Optional anchor ID (version-chain identity). **Not hashed.** Kept out
    /// of the content hash so that the same content under a different anchor
    /// produces the same CID.
    #[serde(skip)]
    pub anchor_id: Option<u64>,
}

impl Node {
    /// Create a new Node with the given labels and properties. `anchor_id` is
    /// left unset; callers that want version chains assign it separately.
    pub fn new(labels: Vec<String>, properties: BTreeMap<String, Value>) -> Self {
        Self {
            labels,
            properties,
            anchor_id: None,
        }
    }

    /// Convenience empty-Node constructor. No labels, no properties.
    /// Used by integration tests calling `engine.call(handler, op, Node::empty())`.
    #[must_use]
    pub fn empty() -> Self {
        Self::new(Vec::new(), BTreeMap::new())
    }

    /// Produce the canonical DAG-CBOR byte string used as the hash input.
    ///
    /// Hash input is a two-field map: `{"labels": [...], "properties": {...}}`.
    /// `anchor_id` is excluded via `#[serde(skip)]` on the field.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Serialize`] if `serde_ipld_dagcbor` cannot encode
    /// the Node (e.g., non-UTF-8 in a text field, which the type system
    /// already prevents, or integer overflow in the CBOR encoder).
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CoreError> {
        // We intentionally re-serialize only labels + properties rather than
        // the full Node. This is belt-and-suspenders: `#[serde(skip)]` on
        // `anchor_id` already excludes it, but going through a dedicated
        // struct makes the hash input contract explicit.
        let view = NodeHashView {
            labels: &self.labels,
            properties: &self.properties,
        };
        serde_ipld_dagcbor::to_vec(&view).map_err(|e| CoreError::Serialize(format_err(&e)))
    }

    /// Compute the CIDv1 for this Node.
    ///
    /// The CID is: `[0x01, 0x71, 0x1e, 0x20, <32-byte BLAKE3 digest>]`
    /// (version, codec, hash-code, hash-length, digest).
    ///
    /// # Errors
    ///
    /// Propagates [`CoreError::Serialize`] from [`Node::canonical_bytes`].
    pub fn cid(&self) -> Result<Cid, CoreError> {
        let bytes = self.canonical_bytes()?;
        let digest = blake3::hash(&bytes);
        Ok(Cid::from_blake3_digest(*digest.as_bytes()))
    }
}

/// Private serde view that encodes exactly the hash-input fields.
/// Using a struct (not the map literal) keeps the field order and names
/// explicit; `serde_ipld_dagcbor` sorts them by length-first order at encode
/// time, so `"labels"` (6) precedes `"properties"` (10).
#[derive(Serialize)]
struct NodeHashView<'a> {
    labels: &'a Vec<String>,
    properties: &'a BTreeMap<String, Value>,
}

// ---------------------------------------------------------------------------
// Cid
// ---------------------------------------------------------------------------

/// A Benten content identifier — CIDv1 with multicodec `dag-cbor` and
/// multihash `blake3`.
///
/// This is intentionally a thin newtype for the spike. Phase 1 proper will
/// migrate to the `cid` crate for full IPLD interop; the byte layout is
/// compatible, so the migration is a drop-in.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Cid(
    // serde does not derive `Serialize`/`Deserialize` for `[u8; N]` where N > 32
    // (the trait is only impl'd for arrays up to length 32). Wrap in `Vec<u8>`
    // for the serde path but validate the length in the accessor so the invariant
    // is preserved.
    #[serde(with = "serde_bytes_fixed")] [u8; CID_LEN],
);

mod serde_bytes_fixed {
    //! Serialize a fixed-size byte array as a CBOR byte string; deserialize
    //! and validate the length.
    use super::CID_LEN;
    use alloc::vec::Vec;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub(super) fn serialize<S: Serializer>(bytes: &[u8; CID_LEN], s: S) -> Result<S::Ok, S::Error> {
        serde_bytes::Bytes::new(bytes).serialize(s)
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; CID_LEN], D::Error> {
        let v: Vec<u8> = serde_bytes::ByteBuf::deserialize(d)?.into_vec();
        if v.len() != CID_LEN {
            return Err(serde::de::Error::invalid_length(v.len(), &"36"));
        }
        let mut out = [0u8; CID_LEN];
        out.copy_from_slice(&v);
        Ok(out)
    }
}

impl Cid {
    /// Construct a Benten CIDv1 from a 32-byte BLAKE3 digest.
    pub fn from_blake3_digest(digest: [u8; 32]) -> Self {
        let mut buf = [0u8; CID_LEN];
        buf[0] = CID_V1;
        buf[1] = MULTICODEC_DAG_CBOR;
        buf[2] = MULTIHASH_BLAKE3;
        buf[3] = BLAKE3_DIGEST_LEN;
        buf[4..].copy_from_slice(&digest);
        Cid(buf)
    }

    /// Parse a Benten CIDv1 from raw bytes. Returns an error if the length or
    /// any of the fixed header bytes do not match Benten's profile.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::InvalidCid`] with a reason if the bytes are
    /// malformed.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CoreError> {
        if bytes.len() != CID_LEN {
            return Err(CoreError::InvalidCid("wrong length"));
        }
        if bytes[0] != CID_V1 {
            return Err(CoreError::InvalidCid("wrong CID version"));
        }
        if bytes[1] != MULTICODEC_DAG_CBOR {
            return Err(CoreError::InvalidCid("wrong multicodec"));
        }
        if bytes[2] != MULTIHASH_BLAKE3 {
            return Err(CoreError::InvalidCid("wrong multihash code"));
        }
        if bytes[3] != BLAKE3_DIGEST_LEN {
            return Err(CoreError::InvalidCid("wrong digest length"));
        }
        let mut buf = [0u8; CID_LEN];
        buf.copy_from_slice(bytes);
        Ok(Cid(buf))
    }

    /// Raw bytes, suitable for storage keys or wire transmission.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; CID_LEN] {
        &self.0
    }

    /// Parse a base32-multibase-prefixed CIDv1 string (e.g.
    /// `"bafyr4i..."`). See [`Cid::to_base32`] for the inverse.
    ///
    /// **Phase 1 G1 stub** — lands with the `cid`-crate migration (see C4).
    pub fn from_str(_s: &str) -> Result<Self, CoreError> {
        todo!("Cid::from_str — G1 (Phase 1)")
    }

    /// Base32 (RFC 4648, lowercase, no padding) string accessor, prefixed with
    /// multibase `b` per the multibase spec. This is the standard IPLD string
    /// representation of a CIDv1 and what the spike reports as the canonical
    /// test Node hash.
    #[must_use]
    pub fn to_base32(&self) -> String {
        let mut out = String::with_capacity(1 + (CID_LEN * 8).div_ceil(5));
        out.push('b');
        base32_lower_nopad_encode(&self.0, &mut out);
        out
    }
}

impl fmt::Display for Cid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_base32())
    }
}

/// Minimal base32 lowercase, no padding, writing to a [`String`].
///
/// We roll our own tiny encoder to avoid pulling in a multibase/base32 crate
/// for the spike. The alphabet is the lowercase form of the RFC 4648 standard
/// base32 alphabet (`a-z` + `2-7`), which is what the IPLD multibase prefix
/// `b` specifies. This is NOT the RFC 4648 Extended Hex alphabet (`0-9` + `a-v`).
fn base32_lower_nopad_encode(input: &[u8], out: &mut String) {
    const ALPHABET: &[u8; 32] = b"abcdefghijklmnopqrstuvwxyz234567";
    let mut buffer: u32 = 0;
    let mut bits: u32 = 0;
    for &byte in input {
        buffer = (buffer << 8) | u32::from(byte);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            let idx = ((buffer >> bits) & 0x1f) as usize;
            out.push(ALPHABET[idx] as char);
        }
    }
    if bits > 0 {
        let idx = ((buffer << (5 - bits)) & 0x1f) as usize;
        out.push(ALPHABET[idx] as char);
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors produced by `benten-core`.
///
/// We use `thiserror` for ergonomic `Display`/`Error` impls. The spike surface
/// is deliberately small; Phase 1 proper will expand this to cover version-
/// chain and edge errors.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    /// DAG-CBOR serialization failed. Carries a human-readable message since
    /// `serde_ipld_dagcbor::EncodeError` is generic over the writer type and
    /// doesn't implement `Clone`.
    #[error("dag-cbor serialization failed: {0}")]
    Serialize(String),

    /// The bytes supplied to [`Cid::from_bytes`] are not a valid Benten CIDv1.
    #[error("invalid CID: {0}")]
    InvalidCid(&'static str),

    /// A `Value::Float(f64::NAN)` was submitted for hashing (Phase 1 G1-A).
    #[error("float NaN is not permitted in the hash path")]
    FloatNan,

    /// A `Value::Float(±Infinity)` was submitted for hashing (Phase 1 G1-A).
    #[error("non-finite float is not permitted in the hash path")]
    FloatNonFinite,

    /// A concurrent append created a branched version chain (C6).
    #[error("version chain has diverging branches")]
    VersionBranched,

    /// String couldn't parse into a `Cid`.
    #[error("failed to parse CID string: {0}")]
    CidParse(&'static str),

    /// CIDv1 with an unsupported multicodec (must be dag-cbor / 0x71).
    #[error("unsupported multicodec in CID")]
    CidUnsupportedCodec,

    /// CIDv1 with an unsupported multihash (must be blake3 / 0x1e).
    #[error("unsupported multihash in CID")]
    CidUnsupportedHash,

    /// Generic not-found error (version-chain anchor, etc.). **Phase 1 stub.**
    #[error("not found")]
    NotFound,
}

impl CoreError {
    /// Map a `CoreError` variant to its ERROR-CATALOG stable code.
    ///
    /// **Phase 1 G1-A stub** — implementation lands in Phase 1 proper.
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        match self {
            CoreError::FloatNan => ErrorCode::ValueFloatNan,
            CoreError::FloatNonFinite => ErrorCode::ValueFloatNonFinite,
            CoreError::CidParse(_) | CoreError::InvalidCid(_) => ErrorCode::CidParse,
            CoreError::CidUnsupportedCodec => ErrorCode::CidUnsupportedCodec,
            CoreError::CidUnsupportedHash => ErrorCode::CidUnsupportedHash,
            CoreError::VersionBranched => ErrorCode::VersionBranched,
            CoreError::Serialize(_) => ErrorCode::Unknown(alloc::string::String::new()),
            CoreError::NotFound => ErrorCode::NotFound,
        }
    }
}

// ---------------------------------------------------------------------------
// ErrorCode enum (C7 — source of truth for stable catalog codes)
// ---------------------------------------------------------------------------

/// Stable error-catalog discriminants. Every `CoreError` / `GraphError` /
/// `CapError` / `EngineError` variant maps to one of these via a `.code()`
/// method so the TS layer sees the same identifier on every error.
///
/// **Phase 1 G1-A stub** — full variant list + bidirectional drift-detector
/// parity with `docs/ERROR-CATALOG.md` lands with G1-A implementation.
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
    /// Fallback for drift detector — holds the unknown raw string.
    Unknown(alloc::string::String),
}

impl ErrorCode {
    /// Return the stable string identifier (e.g. `"E_INV_CYCLE"`).
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
            ErrorCode::Unknown(s) => s.as_str(),
        }
    }

    /// Parse a stable catalog code string into an `ErrorCode`, falling back to
    /// [`ErrorCode::Unknown`] for forward-compat drift.
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
            other => ErrorCode::Unknown(alloc::string::ToString::to_string(other)),
        }
    }
}

// ---------------------------------------------------------------------------
// Edge (C2 — Phase 1 G1-B stub)
// ---------------------------------------------------------------------------

/// A graph Edge. Content-addressed over `(source_cid, target_cid, label, properties)`.
///
/// Endpoint Node CIDs are **not** affected by edge creation — the Node's CID
/// is determined only by its own labels+properties (see ENGINE-SPEC §7).
///
/// **Phase 1 G1-B stub** — real impl lands in Phase 1 proper.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    pub source: Cid,
    pub target: Cid,
    pub label: String,
    pub properties: Option<BTreeMap<String, Value>>,
}

impl Edge {
    /// Construct a new Edge.
    #[must_use]
    pub fn new(
        _source: Cid,
        _target: Cid,
        _label: impl Into<String>,
        _properties: Option<BTreeMap<String, Value>>,
    ) -> Self {
        todo!("Edge::new — G1-B (Phase 1)")
    }

    /// Canonical CBOR bytes for hashing (source, target, label, properties).
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CoreError> {
        todo!("Edge::canonical_bytes — G1-B (Phase 1)")
    }

    /// Content-addressed Edge CID.
    pub fn cid(&self) -> Result<Cid, CoreError> {
        todo!("Edge::cid — G1-B (Phase 1)")
    }
}

// ---------------------------------------------------------------------------
// Anchor + version-chain helpers (C6 — Phase 1 G1-B stub)
// ---------------------------------------------------------------------------

/// The `CURRENT` edge label — anchor → current-version Node pointer.
pub const LABEL_CURRENT: &str = "CURRENT";

/// The `NEXT_VERSION` edge label — previous-version → next-version Node.
pub const LABEL_NEXT_VERSION: &str = "NEXT_VERSION";

/// An opt-in version-chain Anchor Node identity.
///
/// **Phase 1 G1-B stub** — real impl lands in Phase 1 proper.
#[derive(Debug, Clone, PartialEq)]
pub struct Anchor {
    pub id: u64,
}

impl Anchor {
    #[must_use]
    pub fn new() -> Self {
        todo!("Anchor::new — G1-B (Phase 1)")
    }
}

impl Default for Anchor {
    fn default() -> Self {
        Self::new()
    }
}

/// Append a Version Node to an Anchor, returning the updated CURRENT Cid.
///
/// **Phase 1 G1-B stub.**
pub fn append_version(_anchor: &Anchor, _version: &Node) -> Result<Cid, CoreError> {
    todo!("append_version — G1-B (Phase 1)")
}

/// Resolve the Anchor's current (latest) version Cid via the CURRENT edge.
///
/// **Phase 1 G1-B stub.**
pub fn current_version(_anchor: &Anchor) -> Result<Cid, CoreError> {
    todo!("current_version — G1-B (Phase 1)")
}

/// Walk an Anchor's version chain, yielding Version Node CIDs in oldest-first order.
///
/// **Phase 1 G1-B stub.**
pub fn walk_versions(_anchor: &Anchor) -> Result<Vec<Cid>, CoreError> {
    todo!("walk_versions — G1-B (Phase 1)")
}

/// Alternative version-chain surface used by the branched-chain edge-case
/// tests (`version_branched.rs`). Shape mirrors the prior-CID-threaded
/// protocol: each `append_version(anchor, prior_head, new_head)` requires
/// the caller to name the head they're building on. Concurrent appenders
/// naming the same prior head fork the chain -> `VersionError::Branched`.
///
/// **Phase 1 G1-B stub.** Distinct from the top-level `Anchor` /
/// `append_version` pair — they co-exist for Phase 1, with the final
/// canonical shape chosen post-R5 once the evaluator lands.
pub mod version {
    use super::Cid;
    use alloc::string::String;

    /// Alternative Anchor shape that stores the current head CID inline.
    #[derive(Debug, Clone, PartialEq)]
    pub struct Anchor {
        pub head: Cid,
    }

    impl Anchor {
        /// Construct an anchor rooted at `head`. **Phase 1 G1-B stub.**
        #[must_use]
        pub fn new(head: Cid) -> Self {
            Self { head }
        }
    }

    /// Error surface for the prior-threaded append API.
    #[derive(Debug, thiserror::Error)]
    pub enum VersionError {
        /// Two appends against the same prior head -> chain forks.
        #[error("chain branched; seen head {seen:?}")]
        Branched { seen: Cid, attempted: Cid },

        /// Caller supplied a prior head the anchor has never seen.
        #[error("unknown prior head")]
        UnknownPrior { supplied: Cid },

        /// Other internal error.
        #[error("version error: {0}")]
        Other(String),
    }

    /// Append `new_head` against `prior_head`. **Phase 1 G1-B stub.**
    pub fn append_version(
        _anchor: &Anchor,
        _prior_head: &Cid,
        _new_head: &Cid,
    ) -> Result<(), VersionError> {
        todo!("version::append_version — G1-B (Phase 1)")
    }

    /// Walk the chain from oldest to newest, yielding CIDs. **Phase 1 G1-B stub.**
    pub fn walk_versions(_anchor: &Anchor) -> alloc::vec::IntoIter<Cid> {
        todo!("version::walk_versions — G1-B (Phase 1)")
    }
}

/// Format any `Display`able error into an owned `String`. Kept out of the
/// error constructor so we don't accidentally pull `alloc::format!` in from
/// a place that can't see `alloc`.
fn format_err<E: fmt::Display>(e: &E) -> String {
    use core::fmt::Write as _;
    let mut s = String::new();
    // Writing to a String cannot fail; ignore the Result to avoid `expect`
    // which is denied by our workspace lints.
    let _ = write!(&mut s, "{e}");
    s
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    reason = "tests and benches may use unwrap per workspace policy"
)]
mod tests {
    use super::*;
    // Single source of truth for the canonical fixture — re-use the public
    // constructor from `testing` rather than defining a second private copy
    // that could drift.
    use super::testing::canonical_test_node;
    extern crate std;
    use alloc::string::ToString;
    use alloc::vec;

    /// D1 — Intra-process determinism: hashing the same Node twice in the
    /// same process produces identical CIDs.
    #[test]
    fn d1_intra_process_determinism() {
        let a = canonical_test_node();
        let b = canonical_test_node();
        let cid_a = a.cid().unwrap();
        let cid_b = b.cid().unwrap();
        assert_eq!(cid_a, cid_b, "same Node must hash to the same CID");
        assert_eq!(cid_a.to_base32(), cid_b.to_base32());
    }

    /// `anchor_id` must not affect the CID — it's identity, not content.
    #[test]
    fn anchor_id_excluded_from_hash() {
        let mut a = canonical_test_node();
        let mut b = canonical_test_node();
        a.anchor_id = Some(1);
        b.anchor_id = Some(999_999);
        assert_eq!(a.cid().unwrap(), b.cid().unwrap());
    }

    /// Property insertion order must not affect the CID — `BTreeMap` +
    /// DAG-CBOR canonical form guarantee this.
    #[test]
    fn property_order_does_not_affect_hash() {
        let mut props_forward = BTreeMap::new();
        props_forward.insert("a".to_string(), Value::Int(1));
        props_forward.insert("b".to_string(), Value::Int(2));
        props_forward.insert("c".to_string(), Value::Int(3));

        let mut props_reverse = BTreeMap::new();
        props_reverse.insert("c".to_string(), Value::Int(3));
        props_reverse.insert("b".to_string(), Value::Int(2));
        props_reverse.insert("a".to_string(), Value::Int(1));

        let n1 = Node::new(vec!["T".to_string()], props_forward);
        let n2 = Node::new(vec!["T".to_string()], props_reverse);
        assert_eq!(n1.cid().unwrap(), n2.cid().unwrap());
    }

    /// CID header bytes follow the fixed `CIDv1` + dag-cbor + blake3 profile.
    #[test]
    fn cid_header_bytes() {
        let cid = canonical_test_node().cid().unwrap();
        let bytes = cid.as_bytes();
        assert_eq!(bytes[0], CID_V1);
        assert_eq!(bytes[1], MULTICODEC_DAG_CBOR);
        assert_eq!(bytes[2], MULTIHASH_BLAKE3);
        assert_eq!(bytes[3], BLAKE3_DIGEST_LEN);
        assert_eq!(bytes.len(), CID_LEN);
    }

    /// Round-trip the raw bytes through `Cid::from_bytes`.
    #[test]
    fn cid_bytes_roundtrip() {
        let cid = canonical_test_node().cid().unwrap();
        let parsed = Cid::from_bytes(cid.as_bytes()).unwrap();
        assert_eq!(cid, parsed);
    }

    /// Base32 multibase prefix must be `b` and the string must be nonempty.
    #[test]
    fn cid_base32_format() {
        let s = canonical_test_node().cid().unwrap().to_base32();
        assert!(s.starts_with('b'), "multibase prefix must be 'b'");
        assert!(s.len() > 10);
    }

    /// Public canonical-CID exposure used by the D2 cross-process fixture test
    /// in `tests/d2_cross_process.rs` (that test lives in the integration-tests
    /// directory so it runs as a separate binary, exercising the "new process"
    /// property).
    #[test]
    fn canonical_cid_is_exposed() {
        let cid = canonical_test_node().cid().unwrap();
        assert_eq!(cid.as_bytes().len(), CID_LEN);
    }
}

// ---------------------------------------------------------------------------
// Public test helper (used by integration tests)
// ---------------------------------------------------------------------------

pub mod testing {
    //! Test helpers shared between unit tests and integration tests.
    //!
    //! Re-exported canonical test Node constructor, used by the D2 cross-process
    //! integration test in `tests/d2_cross_process.rs`. We expose this
    //! unconditionally (rather than behind a `cfg(test)` or a `testing` feature)
    //! because the function is tiny and its presence costs nothing at runtime.

    use super::{BTreeMap, Node, Value};
    use alloc::string::ToString;
    use alloc::vec;

    /// The canonical test Node — identical content across every spike run.
    ///
    /// Used by the D2 cross-process determinism test to assert the CID
    /// remains stable across process boundaries.
    #[must_use]
    pub fn canonical_test_node() -> Node {
        let mut props = BTreeMap::new();
        props.insert("title".to_string(), Value::text("Hello, Benten"));
        props.insert("published".to_string(), Value::Bool(true));
        props.insert("views".to_string(), Value::Int(42));
        props.insert(
            "tags".to_string(),
            Value::List(vec![Value::text("rust"), Value::text("graph")]),
        );
        Node::new(vec!["Post".to_string()], props)
    }
}
