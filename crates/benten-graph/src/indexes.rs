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
pub(crate) const LABEL_INDEX_TABLE: redb::MultimapTableDefinition<&[u8], &[u8]> =
    redb::MultimapTableDefinition::new("benten_label_index");

/// Redb multimap table holding `property_key -> node_cid_bytes`. The property
/// key packs `(label, prop_name, value_bytes)` together — see
/// [`property_index_key`] for the layout.
pub(crate) const PROP_INDEX_TABLE: redb::MultimapTableDefinition<&[u8], &[u8]> =
    redb::MultimapTableDefinition::new("benten_prop_index");

/// Encode a `Value` to its canonical DAG-CBOR bytes for use as an index key
/// component. Shares the encoder used by Nodes and Edges so that equality in
/// the index aligns with equality in canonical storage.
///
/// # Errors
/// Returns [`CoreError::Serialize`] if `serde_ipld_dagcbor::to_vec` fails —
/// in practice impossible for the `Value` type but surfaced for safety.
pub(crate) fn value_index_bytes(value: &Value) -> Result<Vec<u8>, CoreError> {
    serde_ipld_dagcbor::to_vec(value).map_err(|e| CoreError::Serialize(format!("value: {e}")))
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
pub(crate) fn property_index_key(label: &str, prop_name: &str, value_bytes: &[u8]) -> Vec<u8> {
    let label_bytes = label.as_bytes();
    let prop_bytes = prop_name.as_bytes();
    let mut out =
        Vec::with_capacity(4 + label_bytes.len() + 4 + prop_bytes.len() + value_bytes.len());
    out.extend_from_slice(&(label_bytes.len() as u32).to_be_bytes());
    out.extend_from_slice(label_bytes);
    out.extend_from_slice(&(prop_bytes.len() as u32).to_be_bytes());
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
