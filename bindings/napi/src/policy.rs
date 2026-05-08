//! Capability policy kinds surfaced to JS.
//!
//! Values map to the subset of `benten_caps` backends wired into
//! `EngineBuilder`:
//! - `NoAuth` → `NoAuthBackend` (default; trusted-single-process)
//! - `Ucan` → durable UCAN-grounded grant-backed policy (G14-B
//!   `UCANBackend` proof-chain validator + grant-store reads).
//!   Closes audit-6-1 / phase-3-backlog §2.3 — pre-G21-T2 this arm
//!   wired the Phase-1 `LegacyUcanStubBackend` stub
//!   (`E_CAP_NOT_IMPLEMENTED` on every check); post-G21-T2 it routes
//!   through the durable grant-backed surface so `grantCapability`
//!   carrying `issuer` + `hlc` reaches the durable backend's
//!   chain-walker.
//! - `GrantBacked` → `GrantBackedPolicy` backed by the engine's own
//!   `system:CapabilityGrant` / `system:CapabilityRevocation` Nodes.
//!   Same revocation-aware policy as `Ucan` but signals "no UCAN
//!   chain attribution on this grant" (`grantCapability` callers
//!   omit `issuer` / `hlc`).

/// Policy-kind identifiers accepted by `Engine.openWithPolicy`.
///
/// Strings rather than numeric discriminants because `#[napi(string_enum)]`
/// in napi-rs v3 projects to TypeScript as `'NoAuth' | 'Ucan' | 'GrantBacked'`
/// which gives the JS caller autocomplete without needing a TS wrapper
/// rebuild.
#[cfg(feature = "napi-export")]
#[napi_derive::napi(string_enum)]
pub enum PolicyKind {
    /// `NoAuthBackend` — no capability checking. The Phase-1 default;
    /// suitable for trusted single-process embedding only.
    NoAuth,
    /// Durable UCAN-grounded grant-backed policy (G14-B + G21-T2
    /// audit-6-1 closure). Composes `GrantBackedPolicy` (the durable
    /// revocation-aware policy hook) with the underlying
    /// `benten_caps::backends::UCANBackend` proof-chain validator
    /// (chain-walking + nbf/exp validation + per-token revocation).
    /// Grants minted under this kind carry `issuer` + `hlc` for
    /// chain-walker correlation.
    Ucan,
    /// `GrantBackedPolicy` — Phase-2b revocation-aware policy backed by
    /// the engine's own `system:CapabilityGrant` /
    /// `system:CapabilityRevocation` Nodes. Per-actor scope checks fire
    /// at WRITE / SUBSCRIBE / SANDBOX entry. Same durable surface as
    /// `Ucan`; this kind signals "no UCAN-chain attribution on grants".
    GrantBacked,
}

/// Parsed grant fields surfaced to the engine's
/// [`grant_capability_with_proof`] entry point.
///
/// Phase-1 callers populate `actor` + `scope` only; Phase-3 G21-T2
/// callers may also populate `issuer` (DID string of the UCAN-chain
/// root that minted the grant) + `hlc` (HLC stamp at issue time used
/// for replay-window narrowing during chain validation).
///
/// [`grant_capability_with_proof`]: benten_engine::Engine::grant_capability_with_proof
pub(crate) struct ParsedGrant {
    pub actor: String,
    pub scope: String,
    pub issuer: Option<String>,
    pub hlc: Option<i64>,
}

/// JSON-shape of a capability grant as accepted by `Engine.grantCapability`.
///
/// Phase-1 contract: `{ actor: "<cid-string>", scope: "store:post:write" }`.
/// Phase-3 G21-T2 widens to `{ actor, scope, issuer?: "did:key:...",
/// hlc?: <number> }` — the optional `issuer` + `hlc` fields close
/// phase-3-backlog §2.3 (b) (Pre-G21-T2 the parser silently dropped
/// them so even when callers passed them they never reached the
/// durable backend).
pub(crate) fn parse_grant_json(v: serde_json::Value) -> napi::Result<ParsedGrant> {
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
    // G21-T2 / phase-3-backlog §2.3 (b): widen to read `issuer` + `hlc`.
    // Both optional; absent fields produce `None` and the engine treats
    // the grant as Phase-1-style (no UCAN-chain attribution).
    let issuer = obj.get("issuer").and_then(|x| x.as_str()).map(String::from);
    let hlc = obj.get("hlc").and_then(|x| x.as_i64());
    Ok(ParsedGrant {
        actor,
        scope,
        issuer,
        hlc,
    })
}
