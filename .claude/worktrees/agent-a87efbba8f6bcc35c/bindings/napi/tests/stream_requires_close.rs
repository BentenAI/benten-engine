//! G19-C2 wave-7 (§7.1.2) — napi `requiresExplicitClose` accessor
//! source-cite + presence diagnostic.
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.7 G19-C2 +
//! `.addl/phase-3/00-implementation-plan.md` §3 G19-C2 must-pass column):
//!
//! - `tests/stream_requires_explicit_close_napi_accessor_present` — §7.1.2
//!
//! ## Pin shape (source-cite diagnostic per pim-2 §3.6b)
//!
//! `bindings/napi/src/lib.rs::StreamHandleJs::requires_explicit_close`
//! is the napi-side anchor for the TS-side FinalizationRegistry leak
//! detector: handles produced by `engine.openStream(...)` return `true`
//! (explicit-close lifecycle); handles produced by
//! `engine.callStream(...)` return `false` (AsyncIterable auto-close).
//!
//! The TS-side wrapper at `packages/engine/src/stream.ts::wrapStreamHandle`
//! consults this accessor at handle-construction time to decide whether
//! to arm the FinalizationRegistry leak detector. Without the accessor
//! the wrapper would have no observable signal and would either skip
//! arming for all handles (false negatives — leaks would never fire) or
//! arm for all handles (false positives — `for await` AsyncIterable
//! auto-close would erroneously fire `E_STREAM_HANDLE_LEAKED`).
//!
//! This test is a SOURCE-CITE DIAGNOSTIC. The LOAD-BEARING end-to-end
//! pins live at `packages/engine/test/stream_leak.test.ts` (Vitest
//! drives `engine.openStream(...)` + GC pressure + asserts the leak
//! detector fires through the FinalizationRegistry callback).

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[test]
fn stream_requires_explicit_close_napi_accessor_present() {
    let napi_lib_rs = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("lib.rs"),
    )
    .expect("napi lib.rs readable");

    // §7.1.2 source-cite: the accessor is wired with the
    // `js_name = "requiresExplicitClose"` annotation so JS callers see
    // the camelCase name.
    assert!(
        napi_lib_rs.contains("requires_explicit_close")
            && napi_lib_rs.contains("requiresExplicitClose"),
        "lib.rs must expose the requires_explicit_close napi accessor \
         with js_name = \"requiresExplicitClose\" — without it the \
         TS-side FinalizationRegistry leak detector at \
         packages/engine/src/stream.ts has no observable signal to \
         decide whether to arm leak detection per handle (G19-C2 §7.1.2)."
    );

    // Verify the `#[napi(js_name = ...)]` shape so a JS caller can
    // reach the accessor as `streamHandle.requiresExplicitClose()`.
    assert!(
        napi_lib_rs.contains("js_name = \"requiresExplicitClose\""),
        "the napi accessor must carry js_name = \"requiresExplicitClose\" \
         so JS-side callers see the camelCase symbol that
         packages/engine/src/stream.ts::wrapStreamHandle calls."
    );
}
