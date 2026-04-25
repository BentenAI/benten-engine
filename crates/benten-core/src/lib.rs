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
#![warn(missing_docs)]
#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use serde::{Deserialize, Serialize};

pub mod edge;
pub mod value;
pub mod version;

pub use edge::Edge;
pub use value::Value;

/// Phase 2a ucca-9 / arch-r1-2 frozen shape — lifted into `benten-core` so
/// both `benten-graph::WriteAuthority` and `benten-caps::WriteAuthority`
/// re-export the SAME type (avoiding the cross-crate newtype proliferation
/// that bit `noauth_still_permits_everything.rs`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum WriteAuthority {
    /// Normal user path (default).
    User,
    /// Engine-internal privileged write.
    EnginePrivileged,
    /// Phase-3 sync-replica write.
    SyncReplica {
        /// CID of the peer that originated the replicated write.
        origin_peer: Cid,
    },
}

impl Default for WriteAuthority {
    fn default() -> Self {
        WriteAuthority::User
    }
}

/// Phase 2a C5 stub: `Subgraph` placeholder re-exposed at the benten-core
/// root so the DAG-CBOR + content-hash round-trip tests (which live in
/// `benten-core/tests/subgraph_load_verified_migration.rs` per the R2
/// partition) compile against the core surface.
///
/// The real `Subgraph` type lives in `benten-eval`; this thin shape is a
/// compile-only surrogate that carries the minimal accessors (`handler_id`,
/// `cid`, `empty_for_test`) R3 tests reference. Phase 2a G5-A migrates the
/// real Subgraph into benten-core under an opaque DAG-CBOR schema.
///
/// G11-A Wave 3a: the shim now carries a `deterministic` field and encodes
/// via canonical DAG-CBOR so the graph-layer `load_subgraph_verified`
/// round-trip can recompute a stable CID from the stored bytes. The
/// CID-from-DAG-CBOR-bytes invariant is load-bearing for the
/// `load_subgraph_verified_from_store_*` suite.
///
/// TODO(phase-2a-G5-A): move the real Subgraph here + delete this stub.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subgraph {
    handler_id: alloc::string::String,
    deterministic: bool,
}

impl Subgraph {
    /// Construct an empty Subgraph for test fixtures.
    #[must_use]
    pub fn empty_for_test(handler_id: impl Into<alloc::string::String>) -> Self {
        Self {
            handler_id: handler_id.into(),
            deterministic: false,
        }
    }

    /// The handler id this Subgraph was registered under.
    #[must_use]
    pub fn handler_id(&self) -> &str {
        &self.handler_id
    }

    /// Content-addressed CID.
    ///
    /// The CID is BLAKE3 over the canonical DAG-CBOR bytes, so a round-trip
    /// through [`Subgraph::to_dag_cbor`] + [`Subgraph::load_verified`]
    /// recomputes the identical CID — the integrity invariant the graph-
    /// layer `load_subgraph_verified` path depends on.
    ///
    /// # Errors
    /// Returns [`CoreError::Serialize`] on encode failure.
    pub fn cid(&self) -> Result<Cid, CoreError> {
        let bytes = self.to_dag_cbor()?;
        let digest = blake3::hash(&bytes);
        Ok(Cid::from_blake3_digest(*digest.as_bytes()))
    }
}

use benten_errors::ErrorCode;

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

    /// Phase 2a C4 / G2-A: load a Node from DAG-CBOR bytes, verifying that
    /// the recomputed CID matches the supplied `cid`.
    ///
    /// Contract: decode the bytes as DAG-CBOR, then re-canonicalize and
    /// re-hash the result. If the recomputed CID does not byte-match the
    /// supplied `cid`, fire [`CoreError::ContentHashMismatch`] (mapped to
    /// [`ErrorCode::InvContentHash`]) so storage-layer tamper or corruption
    /// surfaces as a distinct, typed error on the node-read path.
    ///
    /// # Errors
    /// - [`CoreError::Serialize`] if the bytes fail to decode as a Node.
    /// - [`CoreError::ContentHashMismatch`] if the recomputed CID doesn't
    ///   match the supplied one (tamper / corruption / codec drift on read).
    pub fn load_verified(cid: &Cid, bytes: &[u8]) -> Result<Self, CoreError> {
        // Hash the incoming bytes FIRST — don't attempt decode against
        // possibly-tampered bytes. A tamper that happens to corrupt the
        // CBOR structure would otherwise surface as a `Serialize` error,
        // masking the real failure (integrity). Bytes-level hash is the
        // authoritative check because, by the canonical-DAG-CBOR contract,
        // a Node's CID is a pure function of its encoded bytes.
        let digest = blake3::hash(bytes);
        let recomputed = Cid::from_blake3_digest(*digest.as_bytes());
        if &recomputed != cid {
            return Err(CoreError::ContentHashMismatch {
                path: "node",
                expected: *cid,
                actual: recomputed,
            });
        }
        // Hash matched — now decode. A decode failure here indicates
        // a genuine DAG-CBOR encoding issue (not tamper), since the bytes
        // hash to the expected CID.
        let node: Self = serde_ipld_dagcbor::from_slice(bytes)
            .map_err(|e| CoreError::Serialize(format_err(&e)))?;
        Ok(node)
    }
}

impl Subgraph {
    /// Phase 2a C5 / G5-A: mark the Subgraph deterministic.
    pub fn set_deterministic(&mut self, value: bool) {
        self.deterministic = value;
    }

    /// Phase 2a C5 / G5-A: DAG-CBOR encode. The bytes produced here are the
    /// hash-input for [`Subgraph::cid`].
    ///
    /// # Errors
    /// Returns [`CoreError::Serialize`] on encode failure.
    pub fn to_dag_cbor(&self) -> Result<Vec<u8>, CoreError> {
        serde_ipld_dagcbor::to_vec(self).map_err(|e| CoreError::Serialize(format_err(&e)))
    }

    /// Phase 2a C5 / G5-A: load a Subgraph from DAG-CBOR bytes.
    ///
    /// This is the no-CID variant — it only validates that the bytes decode
    /// cleanly as a Subgraph. Integrity enforcement (CID vs. computed-hash)
    /// is the job of [`Subgraph::load_verified_with_cid`].
    ///
    /// # Errors
    /// Returns [`CoreError::Serialize`] on decode failure.
    pub fn load_verified(bytes: &[u8]) -> Result<Self, CoreError> {
        serde_ipld_dagcbor::from_slice(bytes).map_err(|e| CoreError::Serialize(format_err(&e)))
    }

    /// Phase 2a C5 / G5-A: load a Subgraph from bytes + an expected CID.
    /// Integrity-enforcing: mismatch between the recomputed CID and
    /// `expected_cid` fires `E_INV_CONTENT_HASH`.
    ///
    /// Mirrors [`Node::load_verified`]: the bytes-level hash is the
    /// authoritative check (hash first, then decode) so a tamper that
    /// happens to corrupt the CBOR structure does not masquerade as a
    /// generic `Serialize` error.
    ///
    /// # Errors
    /// - [`CoreError::ContentHashMismatch`] if the recomputed CID does not
    ///   match `expected_cid`.
    /// - [`CoreError::Serialize`] if the (hash-matching) bytes fail to
    ///   decode as a Subgraph.
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
        serde_ipld_dagcbor::from_slice(bytes).map_err(|e| CoreError::Serialize(format_err(&e)))
    }

    /// Phase 2a C5 / G5-A: whether the Subgraph is classified deterministic.
    #[must_use]
    pub fn is_deterministic(&self) -> bool {
        self.deterministic
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
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
    /// The error arms distinguish three catalogued failure classes:
    ///
    /// - [`CoreError::InvalidCid`] (maps to `E_CID_PARSE`) — structural
    ///   failures: wrong length, wrong CID version byte, wrong advertised
    ///   digest length.
    /// - [`CoreError::CidUnsupportedCodec`] (maps to `E_CID_UNSUPPORTED_CODEC`)
    ///   — CID uses a multicodec other than `dag-cbor` (`0x71`).
    /// - [`CoreError::CidUnsupportedHash`] (maps to `E_CID_UNSUPPORTED_HASH`)
    ///   — CID uses a multihash other than BLAKE3 (`0x1e`).
    ///
    /// Splitting the three classes was a spec-to-code audit finding
    /// (r6b audit §5.4): the catalog promises distinct codes so a Phase-3
    /// sync layer or operator can tell "malformed bytes" apart from
    /// "protocol-mismatch". Before the split every case folded to
    /// `E_CID_PARSE` and the other two catalog codes were unreachable.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CoreError> {
        if bytes.len() != CID_LEN {
            return Err(CoreError::InvalidCid("wrong length"));
        }
        if bytes[0] != CID_V1 {
            return Err(CoreError::InvalidCid("wrong CID version"));
        }
        if bytes[1] != MULTICODEC_DAG_CBOR {
            return Err(CoreError::CidUnsupportedCodec);
        }
        if bytes[2] != MULTIHASH_BLAKE3 {
            return Err(CoreError::CidUnsupportedHash);
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
    /// `"bafyr4i..."`). This is the inverse of [`Cid::to_base32`].
    ///
    /// Phase 1 accepts exactly the multibase form produced by `to_base32`:
    /// the single-character `b` prefix followed by an RFC 4648 lowercase
    /// base32 body with no padding. The decoded bytes are then handed to
    /// [`Cid::from_bytes`], which enforces the Benten CIDv1 layout
    /// (`[0x01, 0x71, 0x1e, 0x20, <32-byte BLAKE3 digest>]`).
    ///
    /// # Errors
    ///
    /// - [`CoreError::CidParse`] if the string is empty, lacks the `b`
    ///   multibase prefix, or contains a character outside the base32
    ///   lowercase alphabet `a-z2-7`.
    /// - The three typed failure classes from [`Cid::from_bytes`]
    ///   ([`CoreError::InvalidCid`], [`CoreError::CidUnsupportedCodec`],
    ///   [`CoreError::CidUnsupportedHash`]) if the decoded bytes are the
    ///   wrong length or carry unexpected multicodec / multihash codes.
    pub fn from_str(s: &str) -> Result<Self, CoreError> {
        // Multibase prefix: Phase 1 accepts only `b` (base32-lower-nopad).
        // Any other leading char — including the common mistakes `B`
        // (base32 upper), `z` (base58btc), `f` (base16), `m` (base64) —
        // is rejected rather than silently accepted.
        let body = s.strip_prefix('b').ok_or(CoreError::CidParse(
            "CID string must use multibase base32-lower-nopad ('b' prefix)",
        ))?;
        let decoded = base32_lower_nopad_decode(body)?;
        Cid::from_bytes(&decoded)
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

/// Decode a base32 lowercase, no-padding body (no multibase prefix) into its
/// raw bytes.
///
/// Inverse of [`base32_lower_nopad_encode`]. Accepts the RFC 4648 lowercase
/// alphabet `a-z` + `2-7`; any other character (uppercase, digits `0`/`1`,
/// multibase prefix, padding `=`, whitespace) is a parse error. Trailing
/// padding bits left at the end of the last 5-bit group must be zero — a
/// well-formed encoder never emits set padding bits, so set bits indicate
/// either a hand-edited string or a different alphabet.
///
/// Kept `no_std`-compatible: returns `Vec<u8>` from `alloc`, no external deps.
fn base32_lower_nopad_decode(input: &str) -> Result<Vec<u8>, CoreError> {
    // Capacity upper bound: each char contributes 5 bits, so byte count is
    // `(chars * 5) / 8`. `div_ceil` overshoots by at most one; a one-byte
    // over-allocation is fine.
    let mut out: Vec<u8> = Vec::with_capacity(input.len() * 5 / 8);
    let mut buffer: u32 = 0;
    let mut bits: u32 = 0;
    for c in input.chars() {
        let value: u32 = match c {
            'a'..='z' => (c as u32) - ('a' as u32),
            '2'..='7' => (c as u32) - ('2' as u32) + 26,
            _ => {
                return Err(CoreError::CidParse(
                    "CID base32 body contains a character outside the lowercase \
                     alphabet a-z2-7",
                ));
            }
        };
        buffer = (buffer << 5) | value;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            // Truncating cast is intentional: `buffer >> bits` fits in 8 bits
            // after masking because only the low `bits + 8` bits of `buffer`
            // are ever populated.
            let byte = ((buffer >> bits) & 0xff) as u8;
            out.push(byte);
        }
    }
    // Any leftover bits < 5 must be zero padding. Non-zero padding bits mean
    // the encoder emitted a set bit past the end of the data, which is
    // either corruption or a different alphabet entirely.
    if bits > 0 {
        let mask = (1u32 << bits) - 1;
        if (buffer & mask) != 0 {
            return Err(CoreError::CidParse(
                "CID base32 body has non-zero trailing padding bits",
            ));
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors produced by `benten-core`.
///
/// We use `thiserror` for ergonomic `Display`/`Error` impls. The spike surface
/// is deliberately small; Phase 1 proper will expand this to cover version-
/// chain and edge errors.
/// `#[non_exhaustive]` (R6b bp-17) — future phases add variants (version-chain
/// conflict subtypes, edge-level validation errors); downstream matchers must
/// include a `_ =>` fallback so additions are a minor version bump.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
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

    /// Content-hash mismatch surfaced at a `*_verified` read entry point.
    /// Fires when the bytes stored under a CID hash to a different CID on
    /// re-read (storage tamper, on-disk corruption, or codec drift). The
    /// `path` discriminant identifies which read surface the mismatch
    /// surfaced on (`"node"` / `"subgraph"`), so diagnostics can distinguish
    /// the node-body mismatch from the subgraph-load mismatch without the
    /// caller having to inspect error-location metadata.
    #[error("content hash mismatch on {path} read: expected {expected}, got {actual}")]
    ContentHashMismatch {
        /// The read surface the mismatch surfaced on (`"node"` / `"subgraph"`).
        path: &'static str,
        /// CID the caller asked for.
        expected: Cid,
        /// CID the recomputed bytes actually hash to.
        actual: Cid,
    },
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
            CoreError::ContentHashMismatch { .. } => ErrorCode::InvContentHash,
        }
    }
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
/// The id field is `pub(crate)` so external callers cannot hand-construct an
/// anchor that collides with the counter's live range or with the reserved
/// sentinel `0`. Read access is via [`Anchor::id`].
#[derive(Debug, Clone, PartialEq)]
pub struct Anchor {
    /// Monotonic process-unique id. Allocated by [`Anchor::new`] only.
    pub(crate) id: u64,
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

    /// The anchor's process-unique identity. Stable for the life of the
    /// `Anchor` value; use as a chain-lookup key.
    #[must_use]
    pub fn id(&self) -> u64 {
        self.id
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
    table.entry(anchor.id).or_default().push(cid);
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
        .and_then(|chain| chain.last().copied())
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

    /// Round-trip through the string form: `to_base32` → `from_str` is the
    /// identity on the canonical fixture. Deeper coverage lives in
    /// `tests/cid_from_str.rs` (alphabet / prefix / length rejections).
    #[test]
    fn cid_string_roundtrip() {
        let cid = canonical_test_node().cid().unwrap();
        let encoded = cid.to_base32();
        let parsed = Cid::from_str(&encoded).unwrap();
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
