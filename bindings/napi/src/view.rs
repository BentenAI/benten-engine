//! View definition + read-result projection.
//!
//! `ViewDefJson` shape: `{ viewId: string, inputLabel?, propertyFilter?,
//! updateStrategy?, budget? }`. The Phase-1 engine derives the view
//! type from `viewId` (see `Engine::create_view` — content-listing
//! views recognize the `content_listing[_<label>]` id family); fields
//! beyond `viewId` are reserved for Phase-2 `ViewCreateOptions`
//! extensions. Unknown fields are rejected up front so a caller
//! building forward-compatible view definitions doesn't get silently
//! ignored.

use benten_engine::{Engine as InnerEngine, UserViewInputPattern, UserViewSpec};
use napi::bindgen_prelude::*;

use crate::error::engine_err;
use crate::node::node_to_json;

/// Known Phase-1 view-definition fields. Anything outside this set
/// surfaces `E_INPUT_LIMIT` so a typo or a forward-compatible field
/// doesn't silently disappear at the napi boundary.
const KNOWN_VIEW_FIELDS: &[&str] = &[
    "viewId",
    "inputLabel",
    "propertyFilter",
    "updateStrategy",
    "budget",
];

/// Extract the view id from the `{ viewId, ... }` JSON shape.
pub(crate) fn extract_view_id(v: &serde_json::Value) -> napi::Result<String> {
    let obj = match v {
        serde_json::Value::Object(obj) => obj,
        _ => {
            return Err(napi::Error::new(
                Status::InvalidArg,
                "viewDef: must be an object",
            ));
        }
    };
    // Reject unknown fields so a forward-compatible caller writing
    // `{ viewId, groupBy: ... }` learns immediately that `groupBy`
    // wasn't honored, rather than at Phase-2 upgrade time.
    for key in obj.keys() {
        if !KNOWN_VIEW_FIELDS.contains(&key.as_str()) {
            return Err(napi::Error::new(
                Status::GenericFailure,
                format!(
                    "E_INPUT_LIMIT: viewDef contains unknown field `{key}`; Phase-1 supports {KNOWN_VIEW_FIELDS:?}",
                ),
            ));
        }
    }
    obj.get("viewId")
        .and_then(|x| x.as_str())
        .map(str::to_string)
        .ok_or_else(|| napi::Error::new(Status::InvalidArg, "viewDef.viewId: required string"))
}

/// Phase-2b G8-B: known fields on the user-view spec JSON shape. Mirrors
/// the TS `UserViewSpec` interface in `packages/engine/src/views.ts`.
const KNOWN_USER_VIEW_FIELDS: &[&str] = &["id", "inputPattern", "strategy"];

/// Parse the `{ id, inputPattern, strategy? }` JSON shape coming from
/// `engine.createView(spec)` into a [`UserViewSpec`].
///
/// Strategy parsing is permissive on case (`'B'` / `'b'` both accepted) so
/// JS callers can hand-write either form. Unknown strategies surface as
/// `E_INPUT_LIMIT` at the napi boundary; the typed
/// `E_VIEW_STRATEGY_A_REFUSED` / `E_VIEW_STRATEGY_C_RESERVED` errors fire
/// at the engine boundary inside `Engine::create_user_view`.
pub(crate) fn parse_user_view_spec(v: &serde_json::Value) -> napi::Result<UserViewSpec> {
    let obj = match v {
        serde_json::Value::Object(obj) => obj,
        _ => {
            return Err(napi::Error::new(
                Status::InvalidArg,
                "createView spec: must be an object",
            ));
        }
    };
    for key in obj.keys() {
        if !KNOWN_USER_VIEW_FIELDS.contains(&key.as_str()) {
            return Err(napi::Error::new(
                Status::GenericFailure,
                format!(
                    "E_INPUT_LIMIT: createView spec contains unknown field `{key}`; supported fields are {KNOWN_USER_VIEW_FIELDS:?}",
                ),
            ));
        }
    }

    let id = obj
        .get("id")
        .and_then(|x| x.as_str())
        .map(str::to_string)
        .ok_or_else(|| {
            napi::Error::new(Status::InvalidArg, "createView spec.id: required string")
        })?;

    let input_pattern_obj = obj.get("inputPattern").ok_or_else(|| {
        napi::Error::new(
            Status::InvalidArg,
            "createView spec.inputPattern: required object",
        )
    })?;
    let input_pattern_obj = input_pattern_obj.as_object().ok_or_else(|| {
        napi::Error::new(
            Status::InvalidArg,
            "createView spec.inputPattern: must be an object",
        )
    })?;
    let input_pattern = if let Some(label) = input_pattern_obj.get("label").and_then(|x| x.as_str())
    {
        UserViewInputPattern::Label(label.to_string())
    } else if let Some(prefix) = input_pattern_obj
        .get("anchorPrefix")
        .and_then(|x| x.as_str())
    {
        UserViewInputPattern::AnchorPrefix(prefix.to_string())
    } else {
        return Err(napi::Error::new(
            Status::InvalidArg,
            "createView spec.inputPattern: must carry either `label` or `anchorPrefix`",
        ));
    };

    let mut builder = UserViewSpec::builder().id(id).input_pattern(input_pattern);

    if let Some(strategy_value) = obj.get("strategy") {
        let strategy_str = strategy_value.as_str().ok_or_else(|| {
            napi::Error::new(
                Status::InvalidArg,
                "createView spec.strategy: must be a string ('A' / 'B' / 'C')",
            )
        })?;
        let strategy = match strategy_str.to_ascii_uppercase().as_str() {
            "A" => benten_ivm::Strategy::A,
            "B" => benten_ivm::Strategy::B,
            "C" => benten_ivm::Strategy::C,
            other => {
                return Err(napi::Error::new(
                    Status::GenericFailure,
                    format!(
                        "E_INPUT_LIMIT: createView spec.strategy `{other}` is not one of 'A' / 'B' / 'C'"
                    ),
                ));
            }
        };
        builder = builder.strategy(strategy);
    }

    builder
        .build()
        .map_err(|msg| napi::Error::new(Status::InvalidArg, msg))
}

// ---------------------------------------------------------------------------
// Phase-3 G19-C1 — UserView.snapshot() + onUpdate() runtime materialization
// (per `docs/future/phase-3-backlog.md` §7.1.3)
// ---------------------------------------------------------------------------

/// Internal: drive [`InnerEngine::user_view_snapshot`] and project the
/// returned `Vec<Node>` to a JSON array. Each row is the same shape the
/// existing `read_view` napi entry point returns inside `Outcome.list`.
///
/// Returns:
/// - `Ok(Some(json_array))` — view registered; rows materialized.
/// - `Ok(None)` — no view with this id is registered.
/// - `Err(...)` — IVM-disabled / view-stale (typed engine errors round-tripped
///   through `engine_err`).
///
/// Cfg-gated `cfg(not(feature = "browser-backend"))` because the
/// underlying `Engine::user_view_snapshot` lives in `engine_views.rs`
/// which is itself gated out of the browser thin-client bundle (per
/// CLAUDE.md baked-in #17 — views are read-only projections of the
/// full peer's state in the wasm32 target). The lib.rs call site is
/// inside `napi_surface` which is also `cfg(not(target_arch = "wasm32"))`-gated,
/// so this gating is consistent.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn user_view_snapshot_adapter(
    engine: &InnerEngine,
    view_id: &str,
) -> napi::Result<Option<serde_json::Value>> {
    match engine.user_view_snapshot(view_id).map_err(engine_err)? {
        None => Ok(None),
        Some(rows) => {
            let arr: Vec<serde_json::Value> = rows.iter().map(node_to_json).collect();
            Ok(Some(serde_json::Value::Array(arr)))
        }
    }
}

/// Internal: drive [`InnerEngine::user_view_drain_updates_since`] and
/// project the returned ChangeEvents to a JSON object the JS-side
/// `view.onUpdate()` async iterator consumes:
///
/// ```text
/// {
///   "registered": true | false,
///   "events": [<ChangeEvent-JSON>],
///   "next_offset": <u64>
/// }
/// ```
///
/// `registered: false` signals "no view with this id" so the JS side
/// can surface a typed error to the caller; `events` is empty until at
/// least one ChangeEvent matching the view's input label is recorded
/// after `since_offset`. The TS wrapper records `next_offset` after
/// each drain so the next async-iterator step replays only events
/// strictly newer than the prior cursor.
///
/// Cfg-gated `cfg(not(feature = "browser-backend"))` for the same
/// reason as `user_view_snapshot_adapter` above — `engine_views.rs`
/// is gated out under browser-backend per CLAUDE.md baked-in #17.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn user_view_drain_updates_adapter(
    engine: &InnerEngine,
    view_id: &str,
    since_offset: u64,
) -> napi::Result<serde_json::Value> {
    let mut map = serde_json::Map::new();
    let drained = engine
        .user_view_drain_updates_since(view_id, since_offset)
        .map_err(engine_err)?;
    let Some(events) = drained else {
        map.insert("registered".into(), serde_json::Value::Bool(false));
        map.insert("events".into(), serde_json::Value::Array(Vec::new()));
        map.insert(
            "next_offset".into(),
            serde_json::Value::Number(serde_json::Number::from(since_offset)),
        );
        return Ok(serde_json::Value::Object(map));
    };
    let next_offset = engine.user_view_change_offset();
    let mut events_json = Vec::with_capacity(events.len());
    for ev in events {
        let mut evmap = serde_json::Map::new();
        evmap.insert(
            "kind".into(),
            serde_json::Value::String(format!("{:?}", ev.kind)),
        );
        evmap.insert(
            "labels".into(),
            serde_json::Value::Array(
                ev.labels
                    .iter()
                    .map(|l| serde_json::Value::String(l.clone()))
                    .collect(),
            ),
        );
        evmap.insert("cid".into(), serde_json::Value::String(ev.cid.to_base32()));
        evmap.insert(
            "tx_id".into(),
            serde_json::Value::Number(serde_json::Number::from(ev.tx_id)),
        );
        events_json.push(serde_json::Value::Object(evmap));
    }
    map.insert("registered".into(), serde_json::Value::Bool(true));
    map.insert("events".into(), serde_json::Value::Array(events_json));
    map.insert(
        "next_offset".into(),
        serde_json::Value::Number(serde_json::Number::from(next_offset)),
    );
    Ok(serde_json::Value::Object(map))
}
