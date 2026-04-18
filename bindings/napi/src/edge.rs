//! JSON projection of `benten_core::Edge`.
//!
//! Exposed as `{ source: string, target: string, label: string,
//! properties?: object }`.

use benten_core::Edge;

use crate::node::value_map_to_json;

pub(crate) fn edge_to_json(edge: &Edge) -> serde_json::Value {
    let mut out = serde_json::Map::new();
    out.insert(
        "source".to_string(),
        serde_json::Value::String(edge.source.to_base32()),
    );
    out.insert(
        "target".to_string(),
        serde_json::Value::String(edge.target.to_base32()),
    );
    out.insert(
        "label".to_string(),
        serde_json::Value::String(edge.label.clone()),
    );
    if let Some(props) = &edge.properties {
        out.insert("properties".to_string(), value_map_to_json(props));
    }
    serde_json::Value::Object(out)
}
