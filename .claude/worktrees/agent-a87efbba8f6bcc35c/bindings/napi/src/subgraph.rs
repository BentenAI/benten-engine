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
use benten_engine::{PrimitiveKind, PrimitiveSpec, SubgraphSpec, WriteSpec};
use napi::bindgen_prelude::*;

use crate::node::json_to_props;

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
        // Non-WRITE kinds: preserve `args` as the PrimitiveSpec
        // properties bag so STREAM/SUBSCRIBE/SANDBOX/etc. carry their
        // declared properties (`source`, `chunkSize`, `pattern`,
        // `manifest`, `module`, etc.) through to the registered handler.
        // Wave-8c-stream-infra: the engine's STREAM dispatch reads
        // `properties.source` + `properties.chunkSize` from the registered
        // PrimitiveSpec; without this, every DSL-built STREAM handler
        // would surface `missing required source property`.
        let effective_id = if id.is_empty() {
            format!("n{idx}")
        } else {
            id.as_str().to_string()
        };
        let args = nm
            .get("args")
            .cloned()
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
        let props = json_to_props(args)?;
        let mut spec = PrimitiveSpec::new(&effective_id, kind);
        for (k, v) in props {
            spec = spec.with_property(&k, v);
        }
        builder = builder.primitive_with_props(spec);
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
                // Wave-8c-stream-infra: preserve `properties` so STREAM /
                // SUBSCRIBE / SANDBOX dispatch can read their declared
                // configuration (`source`, `chunkSize`, `pattern`,
                // `manifest`, `module`, etc.).
                let properties = pm
                    .get("properties")
                    .cloned()
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
                let prop_map = json_to_props(properties)?;
                let mut spec = PrimitiveSpec::new(&id, kind);
                for (k, v) in prop_map {
                    spec = spec.with_property(&k, v);
                }
                builder = builder.primitive_with_props(spec);
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
    use crate::json_build::ObjBuilder;

    // Spec shape: `{ ok, edge?, errorCode?, errorMessage?, createdCid?,
    // cid?, list?, completedIterations?, successfulWriteCount }` — 9
    // possible fields (refinement-audit #1052: prime for the max).
    let created = outcome.created_cid().map(|c| c.to_base32());
    ObjBuilder::with_capacity(9)
        .bool("ok", outcome.is_ok_edge())
        .opt_str("edge", outcome.edge_taken())
        .opt_str("errorCode", outcome.error_code().map(|c| c.to_string()))
        .opt_str("errorMessage", outcome.error_message())
        // Both `createdCid` (spec) and the shorter `cid` alias so the R3
        // test `expect(typeof outcome.cid).toBe("string")` is satisfied
        // without forcing a rename on the TS side.
        .opt_str("createdCid", created.clone())
        .opt_str("cid", created)
        .opt_raw(
            "list",
            outcome.as_list().map(|list| {
                // #812 missed-extract-helper: this list-row was a
                // verbatim copy of `node::node_to_json`'s
                // `{ labels, properties }` shape — reuse it.
                serde_json::Value::Array(
                    list.iter().map(crate::node::node_to_json).collect(),
                )
            }),
        )
        .opt_raw(
            "completedIterations",
            outcome
                .completed_iterations()
                .map(|iter| serde_json::Value::Number(u64::from(iter).into())),
        )
        .u64(
            "successfulWriteCount",
            u64::from(outcome.successful_write_count()),
        )
        .build()
}

/// R6FP-tail (Round-2 Instance 10) — project a
/// [`benten_engine::RegisterReplaceOutcome`] into JSON for the JS side.
///
/// Shape: `{ handlerId, cid, previousCid, chainDepth, versionTag, replaced }`.
/// Pre-Instance-10 the napi/devserver path returned only the new CID
/// String; this helper widens the surface so JS callers can correlate
/// hot-replace observability without subscribing to reload events.
pub(crate) fn register_replace_outcome_to_json(
    outcome: &benten_engine::RegisterReplaceOutcome,
) -> serde_json::Value {
    crate::json_build::ObjBuilder::with_capacity(6)
        .str("handlerId", outcome.handler_id.clone())
        .cid("cid", &outcome.cid)
        .opt_cid("previousCid", outcome.previous_cid.as_ref())
        .u64("chainDepth", outcome.chain_depth as u64)
        .str("versionTag", outcome.version_tag())
        .bool("replaced", outcome.replaced())
        .build()
}
