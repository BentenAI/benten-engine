//! Label and property-value indexes for `RedbBackend`.
//!
//! R1 triage `P1.graph.indexes-on-write` (G5, landed here as part of G2-B).
//! Two `MultimapTable`s are maintained as a side-effect of every
//! [`RedbBackend::put_node`](crate::RedbBackend::put_node) and
//! [`RedbBackend::delete_node`](crate::RedbBackend::delete_node):
//!
//! 1. **Label index** — `label_bytes -> cid_bytes`. One entry per `(node, label)`
//!    pair, so a multi-label Node shows up under every one of its labels.
//! 2. **Property-value index** — `(label, prop_name, value_bytes) -> cid_bytes`.
//!    One entry per `(node, label, prop_name)` triple. Property lookups are
//!    therefore always scoped by label; cross-label property queries are out of
//!    scope for Phase 1.
//!
//! ## Value encoding
//!
//! Index-key bytes for a `Value` come from the same DAG-CBOR encoder the Node
//! and Edge stores already use (`serde_ipld_dagcbor::to_vec`). This guarantees:
//! - deterministic encoding (DAG-CBOR is canonical by construction);
//! - bit-exact equality semantics (two `Value::Int(10)` instances encode to the
//!   same bytes; `Value::Int(10)` and `Value::Text("10")` do not);
//! - no new serialization format to test or migrate.
//!
//! The `label` and `prop_name` parts of the property key are encoded as
//! length-prefixed UTF-8 so that `(label="Po", prop="stviews")` and
//! `(label="Post", prop="views")` cannot collide under concatenation.

use benten_core::{Cid, CoreError, Value};

/// Redb multimap table holding `label_bytes -> node_cid_bytes`. One multimap
/// entry per `(node, label)` pair. Multi-label nodes produce one entry per
/// label.
///
/// G13-C wave-3: gated to NON `wasm32-unknown-unknown` per `br-r1-1` BLOCKER.
#[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
pub(crate) const LABEL_INDEX_TABLE: redb::MultimapTableDefinition<&[u8], &[u8]> =
    redb::MultimapTableDefinition::new("benten_label_index");

/// Redb multimap table holding `property_key -> node_cid_bytes`. The property
/// key packs `(label, prop_name, value_bytes)` together — see
/// [`property_index_key`] for the layout.
///
/// G13-C wave-3: gated to NON `wasm32-unknown-unknown` per `br-r1-1` BLOCKER.
#[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
pub(crate) const PROP_INDEX_TABLE: redb::MultimapTableDefinition<&[u8], &[u8]> =
    redb::MultimapTableDefinition::new("benten_prop_index");

/// Encode a `Value` to its canonical DAG-CBOR bytes for use as an index key
/// component. Shares the *canonical* form used by `Node::canonical_bytes`
/// (the CID input) so that equality in the index aligns with equality in
/// canonical storage.
///
/// refinement-audit-2026-05 (ST-GRAPH lane, adjacent to #548): the prior
/// body called `serde_ipld_dagcbor::to_vec(value)` on the RAW value
/// without first normalizing via [`Value::to_canonical`]. That broke the
/// docstring's own promised invariant for `Value::Float(-0.0)`:
/// `Node::canonical_bytes` collapses `-0.0 → +0.0` (so two Nodes that
/// differ only by the sign of a zero share ONE CID), but the index key
/// did NOT — a Node stored under a `+0.0` property was unfindable by a
/// `-0.0` query (and vice-versa) even though they are the SAME content-
/// addressed Node. Canonicalizing here realigns the index with the CID
/// contract. The pre-existing
/// `indexes_float_zero_parity::neg_zero_and_pos_zero_share_cid_and_share_index_bucket`
/// test pins this (it only ever passed before because it wrote BOTH signs,
/// masking the asymmetry).
///
/// # Errors
/// - [`CoreError::FloatNan`] / [`CoreError::FloatNonFinite`] if the value
///   contains a non-finite float (same rejection `Node::canonical_bytes`
///   applies — an un-indexable value is not silently mis-indexed).
/// - [`CoreError::Serialize`] if `serde_ipld_dagcbor::to_vec` fails —
///   in practice impossible for the canonicalized `Value` type but
///   surfaced for safety.
pub(crate) fn value_index_bytes(value: &Value) -> Result<Vec<u8>, CoreError> {
    let canonical = value.to_canonical()?;
    serde_ipld_dagcbor::to_vec(&canonical).map_err(|e| CoreError::Serialize(format!("value: {e}")))
}

/// Pack `(label, prop_name, value_bytes)` into a single key for
/// [`PROP_INDEX_TABLE`]. Length-prefixes each segment so that different
/// `(label, prop_name)` splits cannot alias into the same key.
///
/// Format: `u32_be(label.len) || label || u32_be(prop.len) || prop || value_bytes`.
///
/// u32 is ample (labels and property names are ≤ 64 KiB in practice) and
/// big-endian keeps keys comparable under redb's lexicographic ordering if we
/// ever want range scans.
///
/// # Panics
/// Panics (via the explicit `len.try_into()` check, #548 / META #629) if a
/// label or property name is ≥ 4 GiB. The prior `len() as u32` cast
/// silently truncated such inputs — a >4 GiB label whose low 32 bits
/// collided with a short label's length would pack into the SAME key
/// prefix, aliasing two distinct `(label, prop)` splits into one index
/// bucket (a content-addressed-index integrity break). A panic on a
/// ≥4 GiB string is the correct fail-stop: such an input is never a
/// legitimate Node label (the engine rejects oversized labels far earlier)
/// and silently mis-indexing is strictly worse than aborting.
pub(crate) fn property_index_key(label: &str, prop_name: &str, value_bytes: &[u8]) -> Vec<u8> {
    let label_bytes = label.as_bytes();
    let prop_bytes = prop_name.as_bytes();
    let label_len: u32 = label_bytes
        .len()
        .try_into()
        .expect("property_index_key: label length must fit in u32 (≥4 GiB label rejected — #548)");
    let prop_len: u32 = prop_bytes.len().try_into().expect(
        "property_index_key: property-name length must fit in u32 (≥4 GiB name rejected — #548)",
    );
    let mut out =
        Vec::with_capacity(4 + label_bytes.len() + 4 + prop_bytes.len() + value_bytes.len());
    out.extend_from_slice(&label_len.to_be_bytes());
    out.extend_from_slice(label_bytes);
    out.extend_from_slice(&prop_len.to_be_bytes());
    out.extend_from_slice(prop_bytes);
    out.extend_from_slice(value_bytes);
    out
}

/// Deserialize a CID from bytes yielded by the multimap tables. A corrupt
/// index entry surfaces as [`CoreError::Serialize`] so callers can choose
/// whether to skip or fail hard.
///
/// # Errors
/// Returns the underlying [`CoreError`] if [`Cid::from_bytes`] rejects the
/// index-stored bytes.
pub(crate) fn cid_from_index_bytes(bytes: &[u8]) -> Result<Cid, CoreError> {
    Cid::from_bytes(bytes)
}
