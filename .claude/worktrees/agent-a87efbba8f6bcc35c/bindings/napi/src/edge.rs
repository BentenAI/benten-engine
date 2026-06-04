//! JSON projection of `benten_core::Edge`.
//!
//! Exposed as `{ source: string, target: string, label: string,
//! properties?: object }`.

use benten_core::Edge;

use crate::json_build::ObjBuilder;
use crate::node::value_map_to_json;

pub(crate) fn edge_to_json(edge: &Edge) -> serde_json::Value {
    // 3 mandatory fields + optional `properties` (refinement-audit
    // #1052: prime for the max-field case so `properties` never
    // triggers a rehash).
    ObjBuilder::with_capacity(4)
        .cid("source", &edge.source)
        .cid("target", &edge.target)
        .str("label", edge.label.clone())
        .opt_raw(
            "properties",
            edge.properties.as_ref().map(value_map_to_json),
        )
        .build()
}
