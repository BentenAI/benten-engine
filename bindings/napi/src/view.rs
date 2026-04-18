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
