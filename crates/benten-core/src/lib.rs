//! # benten-core
//!
//! Core types and content-addressed hashing for the Benten graph engine.
//!
//! This crate defines the Phase 1 hash-path surface:
//!
//! - [`Value`] — the graph value type. Variants: `null`, `bool`, `int`,
//!   `float`, `text`, `bytes`, `list`, and `map`. The `Float` variant rejects
//!   `NaN` and `±Infinity` at `canonical_bytes` time and normalizes `-0.0` to
//!   `+0.0`; see the [`value`] module docs for the full contract.
//! - [`Node`] — a content-addressed graph Node (label list + ordered property
//!   map). Content-addressed via BLAKE3 over DAG-CBOR; see ENGINE-SPEC §7.
//! - [`Edge`] — a content-addressed edge (source, target, label, properties).
//!   Hashed independently of `Node`; edges are excluded from Node hash input.
//! - [`Cid`] — a thin CIDv1 newtype (multicodec `0x71` dag-cbor, multihash
//!   `0x1e` blake3) produced by [`Node::cid`] / [`Edge::cid`].
//!
//! Version chains ship in **two coexisting shapes** (R4 triage cov-f3; R5 G7
//! picks a canonical one):
//!
//! - [`crate::Anchor`] + [`crate::append_version`] / [`crate::current_version`]
//!   / [`crate::walk_versions`] — thin `u64`-id surface keyed by a
//!   process-unique monotonic counter. Simplest shape; no prior-head
//!   declaration; cannot detect concurrent-fork hazards.
//! - [`version::Anchor`] + [`version::append_version`] /
//!   [`version::walk_versions`] — prior-head-threaded surface. Each append
//!   names the head the caller observed, so concurrent writers forking the
//!   chain surface as [`version::VersionError::Branched`] /
//!   [`version::VersionError::UnknownPrior`].
//!
//! The [`Node::anchor_id`] field is version-chain identity; it is **excluded
//! from the content hash** so the same content under a different anchor
//! produces the same CID.
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

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use serde::{Deserialize, Serialize};

pub mod edge;
pub mod error_code;
pub mod value;
pub mod version;

pub use edge::Edge;
pub use error_code::ErrorCode;
pub use value::Value;

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
        // Canonicalize the property tree up-front: reject NaN / ±Inf so the
        // rejection surfaces as a real `CoreError` (with a stable
        // `E_VALUE_FLOAT_*` code) rather than a `Serialize` variant holding a
        // `serde` error message, and normalize `-0.0 → +0.0` so the CID is
        // stable across the sign of zero.
        let mut canonical_props = BTreeMap::new();
        for (k, v) in &self.properties {
            canonical_props.insert(k.clone(), v.to_canonical()?);
        }
        // We intentionally re-serialize only labels + properties rather than
        // the full Node. This is belt-and-suspenders: `#[serde(skip)]` on
        // `anchor_id` already excludes it, but going through a dedicated
        // struct makes the hash input contract explicit.
        let view = NodeHashView {
            labels: &self.labels,
            properties: &canonical_props,
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
///
/// ## Ordering note
///
/// `Ord` / `PartialOrd` is a **byte-lexicographic** comparison over the
/// fixed CIDv1 layout. It is well-defined and stable (the layout never
/// varies in length), which makes `Cid` suitable as a key for ordered
/// containers such as `BTreeMap`. The ordering carries **no semantic
/// meaning**: two CIDs where `a < b` says nothing about content relationship,
/// causality, version precedence, or any other graph-level property — those
/// must be derived from the graph itself.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
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
    /// **Phase 2 deliverable.** The parsing path needs a multibase decoder
    /// which lands with the `cid`-crate migration (C4). In the interim this
    /// returns a typed, recoverable error rather than panicking so production
    /// call sites can degrade gracefully.
    ///
    /// # Errors
    ///
    /// Always returns [`CoreError::CidParse`] in Phase 1.
    pub fn from_str(_s: &str) -> Result<Self, CoreError> {
        Err(CoreError::CidParse(
            "Cid::from_str is a Phase 2 deliverable (needs multibase decoder; see C4)",
        ))
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

// ---------------------------------------------------------------------------
// Anchor + version-chain helpers (C6)
//
// R4 triage (M21): `benten_core::version::*` is the canonical prior-head-
// threaded surface (see `src/version.rs`). The `u64`-id-based Anchor + free
// functions exposed here at the crate root are the thinner compatibility
// surface for the Phase 1 "simple" case where callers don't need to detect
// concurrent appends. R5 keeps both; R5 G7 picks a canonical shape once the
// evaluator lands (cov-f3 residual — `TODO(phase-2)`).
//
// State storage: each u64-id anchor owns a `Vec<Cid>` of appended version
// CIDs (oldest-first), held in a process-wide spinlocked table keyed by
// `Anchor::id`. This matches the Cid-head surface's storage strategy and is
// sufficient for Phase 1 (in-process only).
// ---------------------------------------------------------------------------

/// The `CURRENT` edge label — anchor → current-version Node pointer.
pub const LABEL_CURRENT: &str = "CURRENT";

/// The `NEXT_VERSION` edge label — previous-version → next-version Node.
pub const LABEL_NEXT_VERSION: &str = "NEXT_VERSION";

/// Top-level opt-in version-chain Anchor identity (u64-id shape).
///
/// Each call to [`Anchor::new`] allocates a fresh monotonically-increasing
/// id from a process-wide counter, so two independent anchors never collide.
/// The id itself is not content-addressed (see ENGINE-SPEC §7) — it is
/// identity only, and [`Node::anchor_id`] is excluded from the Node CID.
///
/// `TODO(phase-2)`: `id` is `pub` today because the G1 tests assert
/// `assert_ne!(a.id, b.id)` against freshly-constructed anchors. Tightening
/// to `pub(crate)` + `fn id(&self) -> u64` is deferred to Phase 2; external
/// callers should use [`Anchor::new`] only (hand-constructed `Anchor { id: 0 }`
/// would collide with the sentinel reservation).
#[derive(Debug, Clone, PartialEq)]
pub struct Anchor {
    /// Monotonic process-unique id. Allocated by [`Anchor::new`] only.
    pub id: u64,
}

/// Counter for [`Anchor::new`]. Starts at 1 so `0` remains a sentinel value
/// for future "unset" / "null-anchor" encodings if they become useful.
///
/// `Ordering::Relaxed` is the correct ordering here: the counter's sole
/// correctness requirement is that `fetch_add` produces a distinct value for
/// each call within the process. No other state is synchronized through the
/// counter (the `U64_CHAINS` table is separately protected by a `Mutex`), so
/// the stronger Acquire/Release / SeqCst orderings would be paying for a
/// happens-before edge nothing consumes.
static ANCHOR_COUNTER: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(1);

/// Per-process u64-id version-chain table. `BTreeMap<id, Vec<Cid>>` keyed by
/// anchor id; vec stores version CIDs in oldest-first insertion order.
///
/// `TODO(phase-2-anchorstore)`: this table grows unbounded for the life of
/// the process — every [`Anchor::new`] + [`append_version`] call adds
/// entries, and there is no `drop_anchor` or GC. Fine for Phase 1 (short
/// test runs, bounded integration tests); a long-running bench or the
/// eventual evaluator with churn would want G7's caller-owned `AnchorStore`.
static U64_CHAINS: spin::Lazy<spin::Mutex<BTreeMap<u64, Vec<Cid>>>> =
    spin::Lazy::new(|| spin::Mutex::new(BTreeMap::new()));

impl Anchor {
    /// Allocate a fresh Anchor with a distinct id. Distinct calls never
    /// produce equal ids (monotonic u64 counter, wraps after 2^64-1 calls —
    /// practically unreachable).
    #[must_use]
    pub fn new() -> Self {
        let id = ANCHOR_COUNTER.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        Self { id }
    }
}

impl Default for Anchor {
    fn default() -> Self {
        Self::new()
    }
}

/// Append a Version Node to an Anchor's chain, returning the Node's CID.
///
/// The appended Node's CID becomes the anchor's new [`current_version`].
/// Per ENGINE-SPEC §6 / §7, chain membership is expressed via edges
/// (`CURRENT`, `NEXT_VERSION`); the Node's own content hash is unaffected
/// by position in the chain.
///
/// # Errors
///
/// Propagates [`CoreError::Serialize`] from [`Node::cid`] if the version
/// Node fails to encode.
pub fn append_version(anchor: &Anchor, version: &Node) -> Result<Cid, CoreError> {
    let cid = version.cid()?;
    let mut table = U64_CHAINS.lock();
    table.entry(anchor.id).or_default().push(cid.clone());
    Ok(cid)
}

/// Resolve the Anchor's current (latest) version Cid.
///
/// # Errors
///
/// Returns [`CoreError::NotFound`] if the anchor has no appended versions.
pub fn current_version(anchor: &Anchor) -> Result<Cid, CoreError> {
    let table = U64_CHAINS.lock();
    table
        .get(&anchor.id)
        .and_then(|chain| chain.last().cloned())
        .ok_or(CoreError::NotFound)
}

/// Walk an Anchor's version chain, yielding Version Node CIDs in oldest-first
/// order.
///
/// # Errors
///
/// Currently infallible (returns an empty `Vec` for a never-appended anchor).
/// The `Result` return type reserves space for future revocation / backend
/// failures without a breaking API change.
pub fn walk_versions(anchor: &Anchor) -> Result<Vec<Cid>, CoreError> {
    let table = U64_CHAINS.lock();
    Ok(table.get(&anchor.id).cloned().unwrap_or_default())
}

/// Format any `Display`able error into an owned `String`.
///
/// Centralized so call sites in the hash path (e.g. [`Node::canonical_bytes`]
/// / [`Edge::canonical_bytes`]) don't each need to silence the
/// `expect`-on-`write!` idiom — writing into a freshly-allocated `String`
/// cannot fail, but the workspace's `clippy::unwrap_used` +
/// `clippy::expect_used` lints deny the ergonomic escapes. This helper
/// swallows the infallible `Result` at one site instead of N.
pub(crate) fn format_err<E: fmt::Display>(e: &E) -> String {
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
