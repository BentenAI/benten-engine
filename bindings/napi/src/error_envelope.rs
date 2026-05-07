//! Phase-3 G19-B (§7.2): JSON-shape envelope formatter for the napi
//! error carrier.
//!
//! Lives outside the napi-export-gated `error` module so the
//! in-process-test rlib build (which doesn't pull napi-rs) can still
//! exercise the envelope shape via [`testing::engine_err_message`]
//! end-to-end. Both the production `engine_err` (which wraps the
//! envelope in a `napi::Error`) and the test helper consume this
//! formatter — single source of truth for the wire shape.
//!
//! The wire shape:
//!
//! ```json
//! {
//!   "code": "E_*",            // stable catalog code, ALWAYS present
//!   "message": "<display>",   // EngineError Display rendering
//!   "fields": { ... }         // OPTIONAL — present when
//!                             //   EngineError::context_json() returns Some
//! }
//! ```
//!
//! The TS-side `mapNativeError` (`packages/engine/src/errors.ts`) JSON-
//! parses the message body and populates the typed `BentenError`
//! subclass via `CODE_TO_CTOR_GENERATED` + `.context` from `fields`.

use serde_json::json;

/// Format an [`benten_engine::EngineError`] into the G19-B JSON envelope
/// (`{ "code", "message", "fields"? }`) and return it as a JSON string
/// suitable for stuffing into `napi::Error::new(_, message)`.
///
/// Variants whose `context_json()` returns `Some(...)` ride under
/// `"fields"`; variants that return `None` omit the key entirely so the
/// JS side sees `error.context === undefined` rather than an empty `{}`.
#[must_use]
pub(crate) fn engine_err_envelope_json(err: &benten_engine::EngineError) -> String {
    let code = err.code();
    let display = format!("{err}");
    let body = match err.context_json() {
        Some(fields) => json!({
            "code": code.as_static_str(),
            "message": display,
            "fields": fields,
        }),
        None => json!({
            "code": code.as_static_str(),
            "message": display,
        }),
    };
    serde_json::to_string(&body).unwrap_or_else(|_| {
        // Should be infeasible — `serde_json::Value` always
        // serialises — but keep the path closed under the unlikely
        // failure with the Display rendering as a degraded carrier.
        format!("{}: {display}", code.as_static_str())
    })
}

/// Format a [`benten_core::CoreError`] into the same G19-B JSON
/// envelope. CoreError variants don't currently carry structured
/// fields beyond Display, so `"fields"` is always omitted.
#[must_use]
pub(crate) fn core_err_envelope_json(err: &benten_core::CoreError) -> String {
    let code = err.code();
    let display = format!("{err}");
    let body = json!({
        "code": code.as_static_str(),
        "message": display,
    });
    serde_json::to_string(&body).unwrap_or_else(|_| format!("{}: {display}", code.as_static_str()))
}
