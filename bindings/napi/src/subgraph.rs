//! Decode a JSON subgraph spec into a `benten_engine::SubgraphSpec`.
//!
//! Two JSON shapes are accepted at the napi boundary:
//!
//! 1. The legacy `{ handlerId, primitives: [{ kind, label, properties }] }`
//!    shape — retained for tests that construct spec payloads by hand.
//!
//! 2. The DSL `{ handlerId, nodes: [{ id, primitive, args, edges }], actions, root }`
//!    shape — what `@benten/engine`'s `Subgraph` / `SubgraphBuilder`
//!    emits. All 12 primitive kinds parse structurally; Phase-2-only
//!    primitives (`wait`, `stream`, `subscribe`, `sandbox`) register
//!    cleanly and surface `E_PRIMITIVE_NOT_IMPLEMENTED` at call time
//!    (wired by the evaluator, not here).

use benten_core::Value;
use benten_engine::{PrimitiveKind, SubgraphSpec, WriteSpec};
use napi::bindgen_prelude::*;

use crate::node::{json_to_props, value_to_json};

/// Map a DSL `primitive` string to an evaluator [`PrimitiveKind`]. Returns
/// `None` on unknown kinds so the caller can surface `InvalidArg`.
fn kind_from_str(s: &str) -> Option<PrimitiveKind> {
    match s.to_lowercase().as_str() {
        "read" => Some(PrimitiveKind::Read),
        "write" => Some(PrimitiveKind::Write),
        "transform" => Some(PrimitiveKind::Transform),
        "branch" => Some(PrimitiveKind::Branch),
        "iterate" => Some(PrimitiveKind::Iterate),
        "wait" => Some(PrimitiveKind::Wait),
        "call" => Some(PrimitiveKind::Call),
        "respond" => Some(PrimitiveKind::Respond),
        "emit" => Some(PrimitiveKind::Emit),
        "sandbox" => Some(PrimitiveKind::Sandbox),
        "subscribe" => Some(PrimitiveKind::Subscribe),
        "stream" => Some(PrimitiveKind::Stream),
        _ => None,
    }
}

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

    // Prefer the DSL `nodes` shape when present; fall back to the
    // legacy `primitives` shape otherwise.
    if obj.contains_key("nodes") {
        return decode_dsl_shape(&handler_id, &obj);
    }
    decode_legacy_shape(&handler_id, &obj)
}

/// Decode the DSL shape: `{ handlerId, nodes: [{ id, primitive, args, edges }] }`.
fn decode_dsl_shape(
    handler_id: &str,
    obj: &serde_json::Map<String, serde_json::Value>,
) -> napi::Result<SubgraphSpec> {
    let nodes = match obj.get("nodes") {
        Some(serde_json::Value::Array(arr)) => arr.clone(),
        Some(_) => {
            return Err(napi::Error::new(
                Status::InvalidArg,
                "spec.nodes: must be an array",
            ));
        }
        None => Vec::new(),
    };
    let mut builder = SubgraphSpec::builder().handler_id(handler_id);
    for (idx, node) in nodes.into_iter().enumerate() {
        let nm = match node {
            serde_json::Value::Object(m) => m,
            _ => {
                return Err(napi::Error::new(
                    Status::InvalidArg,
                    format!("nodes[{idx}]: must be an object"),
                ));
            }
        };
        let id = nm
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let prim_str = nm
            .get("primitive")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let Some(kind) = kind_from_str(&prim_str) else {
            return Err(napi::Error::new(
                Status::InvalidArg,
                format!("nodes[{idx}].primitive: unsupported kind `{prim_str}`"),
            ));
        };
        // Extract WRITE-specific args (label + properties + requires) so
        // the engine-side WriteSpec list stays populated for dispatch.
        if matches!(kind, PrimitiveKind::Write) {
            let args = nm
                .get("args")
                .cloned()
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
            let (label, properties, requires) = extract_write_args(args)?;
            let prop_map = json_to_props(properties)?;
            let effective_id = if id.is_empty() {
                format!("n{idx}")
            } else {
                id.clone()
            };
            let _ = effective_id;
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
            continue;
        }
        // Non-WRITE kinds: register structurally so the subgraph's
        // primitive list reflects the shape. Dispatch-time execution
        // of Phase-2-only primitives returns `E_PRIMITIVE_NOT_IMPLEMENTED`.
        let effective_id = if id.is_empty() {
            format!("n{idx}")
        } else {
            id.as_str().to_string()
        };
        builder = builder.primitive(&effective_id, kind);
    }
    Ok(builder.build())
}

/// Extract `(label, properties, requires)` from a WRITE node's `args`.
///
/// Supports two argument shapes:
///   * DSL `write({ label, properties, requires? })` — `args.label`,
///     `args.properties`, optional `args.requires`.
///   * Legacy `{ label, properties }` at the args root (same as above
///     but without the top-level wrapper).
fn extract_write_args(
    args: serde_json::Value,
) -> napi::Result<(String, serde_json::Value, Vec<String>)> {
    let map = match args {
        serde_json::Value::Object(m) => m,
        _ => {
            return Err(napi::Error::new(
                Status::InvalidArg,
                "write.args: must be an object",
            ));
        }
    };
    let label = map
        .get("label")
        .and_then(|v| v.as_str())
        .unwrap_or("post")
        .to_string();
    let properties = map
        .get("properties")
        .cloned()
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
    let requires: Vec<String> = map
        .get("requires")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    Ok((label, properties, requires))
}

/// Decode the legacy `{ primitives: [{ kind, ... }] }` shape.
fn decode_legacy_shape(
    handler_id: &str,
    obj: &serde_json::Map<String, serde_json::Value>,
) -> napi::Result<SubgraphSpec> {
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
    let mut builder = SubgraphSpec::builder().handler_id(handler_id);
    for (idx, prim) in primitives.into_iter().enumerate() {
        let pm = match prim {
            serde_json::Value::Object(m) => m,
            _ => {
                return Err(napi::Error::new(
                    Status::InvalidArg,
                    "primitive: must be an object",
                ));
            }
        };
        let kind_str = pm
            .get("kind")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_lowercase();
        let Some(kind) = kind_from_str(&kind_str) else {
            return Err(napi::Error::new(
                Status::InvalidArg,
                format!("primitive.kind: unsupported kind `{kind_str}`"),
            ));
        };
        match kind {
            PrimitiveKind::Write => {
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
            _ => {
                let id = pm
                    .get("id")
                    .and_then(|v| v.as_str())
                    .map_or_else(|| format!("p{idx}"), str::to_string);
                builder = builder.primitive(&id, kind);
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
