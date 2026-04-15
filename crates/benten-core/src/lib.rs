//! # benten-core
//!
//! Core types and content-addressed hashing for the Benten graph engine.
//!
//! This crate is deliberately minimal for the Phase 1 stack spike. It defines:
//!
//! - [`Value`] — the graph value type (DAG-CBOR representable: null, bool, integer,
//!   float, text, bytes, list, map).
//! - [`Node`] — a content-addressed graph Node (label list + ordered property map).
//! - [`Cid`] — a CIDv1 newtype (multicodec `0x71` dag-cbor, multihash `0x1e` blake3)
//!   produced by [`Node::cid`].
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
//! 1. [`Value::Map`] is backed by [`alloc::collections::BTreeMap`], so property
//!    keys are iterated in Unicode code point order.
//! 2. Serialization uses `serde_ipld_dagcbor`, which enforces DAG-CBOR canonical
//!    form (length-first key sort, no indefinite-length items, no floats-that-
//!    would-round-trip-as-ints, etc.).
//! 3. The CID encoding (version byte `0x01`, multicodec `0x71`, multihash
//!    `0x1e` + length `0x20` + 32-byte digest) is fixed by this crate and is
//!    wire-compatible with the IPLD CIDv1 spec.
//!
//! The spike ships three tests validating these properties (D1 intra-process,
//! D2 cross-process, D3 wasm32 compile-check).

#![forbid(unsafe_code)]
#![no_std]

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
/// Maps use [`BTreeMap`] so serialization is deterministic regardless of
/// insertion order. `serde_ipld_dagcbor` additionally enforces DAG-CBOR's
/// length-first key sort at encode time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    /// CBOR null.
    Null,
    /// CBOR boolean.
    Bool(bool),
    /// CBOR signed integer (-2^63 .. 2^63-1).
    Int(i64),
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

/// Minimal RFC 4648 base32 lowercase, no padding, writing to a [`String`].
///
/// We roll our own tiny encoder to avoid pulling in a multibase/base32 crate
/// for the spike. The alphabet is the RFC 4648 "Extended Hex" lowercase-
/// equivalent is NOT used here; we use the standard base32 alphabet
/// (a-z + 2-7), which matches what multibase `b` requires.
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
    extern crate std;
    use alloc::string::ToString;
    use alloc::vec;

    /// Build the canonical test Node used across the spike's determinism
    /// checks. Kept here (not hidden behind a feature) so `cargo test` and
    /// the cross-process fixture test share the same definition.
    pub(crate) fn canonical_test_node() -> Node {
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
