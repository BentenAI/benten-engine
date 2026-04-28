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

use benten_engine::{UserViewInputPattern, UserViewSpec};
use napi::bindgen_prelude::*;

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
