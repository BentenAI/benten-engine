//! Error translation from `benten_engine::EngineError` / `benten_core::CoreError`
//! into `napi::Error`.
//!
//! Rust errors are mapped to `napi::Status::GenericFailure` with the debug
//! representation as the message so stable error codes survive the JS bridge.
//! Per-primitive typed error edges (`ON_NOT_FOUND`, `ON_DENIED`, …) are NOT
//! thrown — they surface on `Outcome.edge` so TS callers can route without a
//! try/catch.

use napi::bindgen_prelude::*;

/// Map any `Display`able error into a napi error carrying the debug repr.
pub(crate) fn to_napi<E: core::fmt::Display>(err: E) -> napi::Error {
    napi::Error::new(Status::GenericFailure, format!("{err}"))
}

/// Sentinel marker the napi error message uses to delimit the
/// JSON-encoded structured-field context. The TS `mapNativeError`
/// (Group 2 surface) splits on this sentinel to recover the structured
/// bag and attach it to `BentenError.context`. The double-`$` is
/// chosen because it is unlikely to appear naturally in any error
/// Display rendering — keeps the suffix unambiguous.
///
/// R6FP-Group-1 (Round-2 Instance 8) public-shape note: the sentinel
/// string is part of the cross-layer contract between the Rust napi
/// adapter and the TS `mapNativeError`. Changing it requires a
/// coordinated update on both sides.
const CONTEXT_SENTINEL: &str = " :: $$benten-context$$";

/// Map an `EngineError` into a napi error preserving the stable catalog code
/// in the message prefix so Vitest `.toThrow(/E_INPUT_LIMIT/)` matches fire.
///
/// R6FP-Group-1 (Round-2 Instance 8): when the EngineError variant
/// carries structured per-variant fields (e.g.
/// `ModuleManifestCidMismatch { expected, computed, summary }`,
/// `Invariant(RegistrationError { ...14 fields })`), append the
/// JSON-encoded bag as a `$$benten-context$$` suffix on the napi
/// error message. The TS `mapNativeError` (Group 2) splits on the
/// sentinel and populates `BentenError.context`. Pre-fix the napi
/// adapter formatted as `format!("{code}: {err}")` Display-only,
/// reducing all structured fields to a flat string and breaking the
/// TS `BentenError(code, fixHint, message, context?)` constructor's
/// fourth-arg surface.
pub(crate) fn engine_err(err: benten_engine::EngineError) -> napi::Error {
    let code = err.code();
    let display = format!("{err}");
    match err.context_json() {
        Some(ctx) => {
            let ctx_json = serde_json::to_string(&ctx).unwrap_or_else(|_| "{}".into());
            napi::Error::new(
                Status::GenericFailure,
                format!("{code}: {display}{CONTEXT_SENTINEL}{ctx_json}"),
            )
        }
        None => napi::Error::new(Status::GenericFailure, format!("{code}: {display}")),
    }
}

/// Map a `CoreError` into a napi error, preserving the stable catalog code
/// via `CoreError::code().as_static_str()` as the message prefix so the TS
/// `mapNativeError` regex reconstructs the typed subclass (r6-err-8 — was
/// previously fabricating `E_CORE:` which is not a catalog code).
pub(crate) fn core_err(err: benten_core::CoreError) -> napi::Error {
    let code = err.code();
    napi::Error::new(
        Status::InvalidArg,
        format!("{}: {err}", code.as_static_str()),
    )
}
