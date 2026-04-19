//! Capability policy kinds surfaced to JS.
//!
//! Values map to the subset of `benten_caps` backends G7 wired into
//! `EngineBuilder`:
//! - `NoAuth` → `NoAuthBackend` (default)
//! - `Ucan` → `UcanBackend` stub (Phase-3 will return real errors; today the
//!   backend returns `E_CAP_NOT_IMPLEMENTED` on any check)
//! - `GrantBacked` → `GrantBackedPolicy` backed by the engine's own
//!   `system:CapabilityGrant` / `system:CapabilityRevocation` Nodes. The
//!   Rust-side builder wires a `GrantReader` against the engine's own
//!   backend — we just flip a flag here and the `EngineBuilder` handles
//!   the reader construction.

/// Policy-kind identifiers accepted by `Engine.openWithPolicy`.
///
/// Strings rather than numeric discriminants because `#[napi(string_enum)]`
/// in napi-rs v3 projects to TypeScript as `'NoAuth' | 'Ucan' | 'GrantBacked'`
/// which gives the JS caller autocomplete without needing a TS wrapper
/// rebuild.
#[cfg(feature = "napi-export")]
#[napi_derive::napi(string_enum)]
pub enum PolicyKind {
    NoAuth,
    Ucan,
    GrantBacked,
}

/// JSON-shape of a capability grant as accepted by `Engine.grantCapability`.
///
/// `{ actor: "<cid-string>", scope: "store:post:write" }`.
pub(crate) fn parse_grant_json(v: serde_json::Value) -> napi::Result<(String, String)> {
    use napi::bindgen_prelude::*;
    let obj = match v {
        serde_json::Value::Object(m) => m,
        _ => {
            return Err(napi::Error::new(
                Status::InvalidArg,
                "grant: must be an object",
            ));
        }
    };
    let actor = obj
        .get("actor")
        .and_then(|x| x.as_str())
        .ok_or_else(|| napi::Error::new(Status::InvalidArg, "grant.actor: required string"))?
        .to_string();
    let scope = obj
        .get("scope")
        .and_then(|x| x.as_str())
        .ok_or_else(|| napi::Error::new(Status::InvalidArg, "grant.scope: required string"))?
        .to_string();
    Ok((actor, scope))
}
