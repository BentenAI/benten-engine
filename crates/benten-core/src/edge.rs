//! Graph [`Edge`] type — content-addressed over the `(source, target, label,
//! properties)` tuple.
//!
//! Per ENGINE-SPEC §7, a Node's CID is determined purely by its labels and
//! properties. Edges are content-addressed *separately*: an Edge's CID is a
//! function of its endpoints' CIDs (by value, not by reference), its label,
//! and its property map. Creating or mutating an Edge therefore never disturbs
//! the endpoint Node CIDs — a property the test suite pins via
//! `tests/edge_does_not_change_endpoint_cids.rs`.
//!
//! ## Self-loops are allowed
//!
//! Edge-level construction permits `source == target`. DAG-ness (Invariant 1)
//! is a subgraph-level structural check validated by `benten-eval`'s
//! registration-time validator, not by `Edge::new`.
//!
//! ## `None` vs empty-map properties
//!
//! DAG-CBOR distinguishes a missing field (encoded as absent) from an empty
//! map (encoded as `a0`). [`Edge`] preserves this distinction in its CID:
//! `Edge::new(src, tgt, "L", None)` and
//! `Edge::new(src, tgt, "L", Some(BTreeMap::new()))` hash to different CIDs.
//! The inner serde view used for hashing does **not** elide `None` via
//! `skip_serializing_if`, so the CBOR encoder emits `null` for the missing
//! case and `a0` for the empty-map case — a stable 1-byte difference in the
//! hash input.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use crate::{Cid, CoreError, Value, format_err};

/// A graph Edge. Content-addressed over `(source, target, label, properties)`.
///
/// Endpoint Node CIDs are never modified by Edge construction — see the
/// module docs and ENGINE-SPEC §7.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    /// Source endpoint Node CID.
    pub source: Cid,
    /// Target endpoint Node CID. May equal `source` (self-loop).
    pub target: Cid,
    /// Edge label (e.g., `"LIKES"`, `"CURRENT"`, `"NEXT_VERSION"`).
    pub label: String,
    /// Optional property map. `None` is preserved in the CID as distinct from
    /// `Some(BTreeMap::new())`.
    pub properties: Option<BTreeMap<String, Value>>,
}

impl Edge {
    /// Construct a new Edge.
    ///
    /// No validation is performed at construction time: self-loops are
    /// allowed, any label is allowed. DAG-ness and label-policy checks are
    /// subgraph-level and live in `benten-eval`.
    pub fn new(
        source: Cid,
        target: Cid,
        label: impl Into<String>,
        properties: Option<BTreeMap<String, Value>>,
    ) -> Self {
        Self {
            source,
            target,
            label: label.into(),
            properties,
        }
    }

    /// Produce the canonical DAG-CBOR byte string used as the hash input.
    ///
    /// The hash input serializes all four fields (source, target, label,
    /// properties) in declaration order via a private serde view;
    /// `serde_ipld_dagcbor`'s encoder applies DAG-CBOR
    /// length-first key-sort so the on-wire layout is canonical regardless
    /// of the struct's source-order.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Serialize`] if `serde_ipld_dagcbor` cannot encode
    /// the edge (e.g., a non-finite float in the property tree — this mirrors
    /// [`crate::Node::canonical_bytes`] in spirit, though floats in edge
    /// properties are rare).
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CoreError> {
        // Canonicalize property floats up-front (same pattern as Node) so
        // NaN / ±Inf surface as typed errors instead of a Serialize wrap.
        let canonical_props = match &self.properties {
            None => None,
            Some(map) => {
                let mut out = BTreeMap::new();
                for (k, v) in map {
                    out.insert(k.clone(), v.to_canonical()?);
                }
                Some(out)
            }
        };
        let view = EdgeHashView {
            source: &self.source,
            target: &self.target,
            label: &self.label,
            properties: &canonical_props,
        };
        serde_ipld_dagcbor::to_vec(&view).map_err(|e| CoreError::Serialize(format_err(&e)))
    }

    /// Compute the CIDv1 for this Edge.
    ///
    /// # Errors
    ///
    /// Propagates [`CoreError::Serialize`] from [`Edge::canonical_bytes`].
    pub fn cid(&self) -> Result<Cid, CoreError> {
        let bytes = self.canonical_bytes()?;
        let digest = blake3::hash(&bytes);
        Ok(Cid::from_blake3_digest(*digest.as_bytes()))
    }
}

/// Private serde view for Edge hashing. Fields are serialized in struct
/// order; `serde_ipld_dagcbor` applies DAG-CBOR canonical key sort on encode.
///
/// Note we do **not** use `#[serde(skip_serializing_if = "Option::is_none")]`
/// on `properties`: the presence/absence distinction must be preserved in
/// the CID (see module docs).
#[derive(Serialize)]
struct EdgeHashView<'a> {
    source: &'a Cid,
    target: &'a Cid,
    label: &'a String,
    properties: &'a Option<BTreeMap<String, Value>>,
}
