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

/// Map an `EngineError` into a napi error preserving the stable catalog code
/// in the message prefix so Vitest `.toThrow(/E_INPUT_LIMIT/)` matches fire.
pub(crate) fn engine_err(err: benten_engine::EngineError) -> napi::Error {
    let code = err.code();
    napi::Error::new(Status::GenericFailure, format!("{code}: {err}"))
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
