//! Error translation from `benten_engine::EngineError` / `benten_core::CoreError`
//! into `napi::Error`.
//!
//! Phase-3 G19-B (§7.2 BentenError.context full structured-field
//! coverage): the napi error message is a JSON-serialised object with
//! shape `{ "code": "E_*", "message": "<display>", "fields": {...} }`.
//! This replaces the pre-G19-B "message-prefix-`E_*` carrier with
//! `$$benten-context$$` sentinel suffix" shape; the JSON-shape gives
//! every typed-error variant uniform structured-field coverage at the
//! napi → TS boundary without per-variant carrier work. The TS
//! `mapNativeError` (`packages/engine/src/errors.ts`) JSON-parses the
//! message body and populates the typed `BentenError` subclass +
//! `.context` accessor.
//!
//! ## Backward-compat for code-extraction regex tests
//!
//! Existing Vitest patterns like `.toThrow(/E_INPUT_LIMIT/)` continue
//! to match because the JSON encoding always includes a `"code":"E_*"`
//! field as a literal substring. The TS-side `extractCode` regex
//! falls back to message-string scanning when JSON-parse fails, so
//! errors thrown directly from non-engine code paths (e.g. a hand-built
//! `napi::Error` with a plain string message) still round-trip cleanly.

use napi::bindgen_prelude::*;

/// Map any `Display`able error into a napi error carrying the debug repr.
pub(crate) fn to_napi<E: core::fmt::Display>(err: E) -> napi::Error {
    napi::Error::new(Status::GenericFailure, format!("{err}"))
}

/// Phase-3 G19-B (§7.2): map an `EngineError` into a napi error whose
/// `.message` is a JSON object `{ code, message, fields? }`. The TS
/// `mapNativeError` JSON-parses the body and populates the typed
/// `BentenError` subclass with the `code` + per-variant structured
/// `fields` bag (surfacing as `error.context`).
///
/// Variants whose `EngineError::context_json` returns `Some(_)` carry
/// their structured fields inline under `"fields"`; variants that
/// return `None` (Display-lossless) omit the `"fields"` key entirely
/// so the JS side sees `error.context === undefined` rather than an
/// empty `{}`.
pub(crate) fn engine_err(err: benten_engine::EngineError) -> napi::Error {
    let message = crate::error_envelope::engine_err_envelope_json(&err);
    napi::Error::new(Status::GenericFailure, message)
}

/// Phase-3 G19-B (§7.2): map a `CoreError` into a napi error using the
/// same JSON shape as `engine_err` so both surfaces parse identically
/// on the TS side. CoreError variants don't currently carry structured
/// fields beyond Display, so `"fields"` is omitted.
pub(crate) fn core_err(err: benten_core::CoreError) -> napi::Error {
    let message = crate::error_envelope::core_err_envelope_json(&err);
    napi::Error::new(Status::InvalidArg, message)
}
