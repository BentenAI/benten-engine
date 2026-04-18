//! View definition + read-result projection.
//!
//! `ViewDefJson` shape: `{ viewId: string, inputLabel?: string }`. The
//! Phase-1 engine derives the view type from `viewId` (see
//! `Engine::create_view` — content-listing views recognize the
//! `content_listing[_<label>]` id family).

use napi::bindgen_prelude::*;

/// Extract the view id from the `{ viewId, ... }` JSON shape.
pub(crate) fn extract_view_id(v: &serde_json::Value) -> napi::Result<String> {
    match v {
        serde_json::Value::Object(obj) => obj
            .get("viewId")
            .and_then(|x| x.as_str())
            .map(str::to_string)
            .ok_or_else(|| napi::Error::new(Status::InvalidArg, "viewDef.viewId: required string")),
        _ => Err(napi::Error::new(
            Status::InvalidArg,
            "viewDef: must be an object",
        )),
    }
}
