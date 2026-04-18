//! Decode a JSON subgraph spec into a `benten_engine::SubgraphSpec`.
//!
//! The JS-side shape is minimal; fuller DSL surfaces land in Phase 2 when the
//! TypeScript `@benten/engine` wrapper exposes a builder.
//!
//! ```json
//! {
//!   "handlerId": "create_post",
//!   "primitives": [
//!     { "kind": "write", "label": "post",
//!       "properties": { "title": "..." } },
//!     { "kind": "respond" }
//!   ]
//! }
//! ```

use benten_core::Value;
use benten_engine::{SubgraphSpec, WriteSpec};
use napi::bindgen_prelude::*;

use crate::node::{json_to_props, value_to_json};

/// Convert the JS-side JSON shape into a `SubgraphSpec`.
///
/// Unknown primitive kinds / missing fields surface as `InvalidArg`.
pub(crate) fn json_to_subgraph_spec(v: serde_json::Value) -> napi::Result<SubgraphSpec> {
    let obj = match v {
        serde_json::Value::Object(m) => m,
        _ => {
            return Err(napi::Error::new(
                Status::InvalidArg,
                "spec: must be an object",
            ));
        }
    };
    let handler_id = obj
        .get("handlerId")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if handler_id.is_empty() {
        return Err(napi::Error::new(
            Status::InvalidArg,
            "spec.handlerId: required non-empty string",
        ));
    }
    let primitives = match obj.get("primitives") {
        Some(serde_json::Value::Array(arr)) => arr.clone(),
        Some(_) => {
            return Err(napi::Error::new(
                Status::InvalidArg,
                "spec.primitives: must be an array",
            ));
        }
        None => Vec::new(),
    };
    let mut builder = SubgraphSpec::builder().handler_id(&handler_id);
    for prim in primitives {
        let pm = match prim {
            serde_json::Value::Object(m) => m,
            _ => {
                return Err(napi::Error::new(
                    Status::InvalidArg,
                    "primitive: must be an object",
                ));
            }
        };
        let kind = pm
            .get("kind")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_lowercase();
        match kind.as_str() {
            "write" => {
                let label = pm
                    .get("label")
                    .and_then(|v| v.as_str())
                    .unwrap_or("post")
                    .to_string();
                let properties = pm
                    .get("properties")
                    .cloned()
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
                let prop_map = json_to_props(properties)?;
                let requires: Vec<String> = pm
                    .get("requires")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|x| x.as_str().map(str::to_string))
                            .collect()
                    })
                    .unwrap_or_default();
                builder = builder.write(move |mut w: WriteSpec| {
                    w = w.label(&label);
                    for (k, v) in prop_map {
                        w = w.property(&k, v);
                    }
                    for scope in requires {
                        w = w.requires(&scope);
                    }
                    w
                });
            }
            "respond" => {
                builder = builder.respond();
            }
            other => {
                return Err(napi::Error::new(
                    Status::InvalidArg,
                    format!("primitive.kind: unsupported kind `{other}`"),
                ));
            }
        }
    }
    Ok(builder.build())
}

/// Project an `Outcome` into JSON for the JS side.
///
/// Shape: `{ ok, edge?, errorCode?, errorMessage?, createdCid?, list?,
/// completedIterations?, successfulWriteCount }`.
pub(crate) fn outcome_to_json(outcome: &benten_engine::Outcome) -> serde_json::Value {
    let mut out = serde_json::Map::new();
    let ok = outcome.is_ok_edge();
    out.insert("ok".to_string(), serde_json::Value::Bool(ok));
    if let Some(edge) = outcome.edge_taken() {
        out.insert("edge".to_string(), serde_json::Value::String(edge));
    }
    if let Some(code) = outcome.error_code() {
        out.insert(
            "errorCode".to_string(),
            serde_json::Value::String(code.to_string()),
        );
    }
    if let Some(msg) = outcome.error_message() {
        out.insert("errorMessage".to_string(), serde_json::Value::String(msg));
    }
    if let Some(cid) = outcome.created_cid() {
        let s = cid.to_base32();
        // Both `createdCid` (spec) and the shorter `cid` alias so the R3 test
        // `expect(typeof outcome.cid).toBe("string")` is satisfied without
        // forcing a rename on the TS side.
        out.insert(
            "createdCid".to_string(),
            serde_json::Value::String(s.clone()),
        );
        out.insert("cid".to_string(), serde_json::Value::String(s));
    }
    if let Some(list) = outcome.as_list() {
        let json_list = list
            .iter()
            .map(|n| {
                let mut obj = serde_json::Map::new();
                obj.insert(
                    "labels".to_string(),
                    serde_json::Value::Array(
                        n.labels
                            .iter()
                            .cloned()
                            .map(serde_json::Value::String)
                            .collect(),
                    ),
                );
                obj.insert(
                    "properties".to_string(),
                    serde_json::Value::Object(
                        n.properties
                            .iter()
                            .map(|(k, v)| (k.clone(), value_to_json(v)))
                            .collect(),
                    ),
                );
                serde_json::Value::Object(obj)
            })
            .collect();
        out.insert("list".to_string(), serde_json::Value::Array(json_list));
    }
    if let Some(iter) = outcome.completed_iterations() {
        out.insert(
            "completedIterations".to_string(),
            serde_json::Value::Number(u64::from(iter).into()),
        );
    }
    out.insert(
        "successfulWriteCount".to_string(),
        serde_json::Value::Number(u64::from(outcome.successful_write_count()).into()),
    );
    // Silence unused-import for release builds.
    let _ = Value::Null;
    serde_json::Value::Object(out)
}
