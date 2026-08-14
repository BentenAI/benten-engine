//! SANDBOX return-ABI: a result type the ABI cannot encode FAILS CLOSED.
//!
//! # The defect these pins close
//!
//! `encode_return_values` in `crates/benten-eval/src/primitives/sandbox.rs`
//! had a catch-all arm reading, in full:
//!
//! ```text
//! // V128, FuncRef, ExternRef — current corpus doesn't use them; encode as zero placeholder.
//! _ => out.extend_from_slice(&[0u8; 16]),
//! ```
//!
//! A guest returning a `v128` (wasm SIMD) therefore got SIXTEEN ZERO
//! BYTES and an `Ok` — a silently WRONG answer, not an error. Verified
//! against the shipped executor before the fix: a module returning
//! `v128.const i32x4 0x11111111 0x22222222 0x33333333 0x44444444`
//! returned `Ok(SandboxResult { output: [0; 16], .. })`.
//!
//! Two things made this reachable rather than theoretical:
//!   * wasmtime enables the SIMD proposal BY DEFAULT (`Config::wasm_simd`
//!     is `true`) and `sandbox::instance::shared_engine` does not disable
//!     it — so `v128` compiles and runs.
//!   * CLAUDE.md baked-in #16 designates SANDBOX as the escape hatch for
//!     "heavy math, ML inference, custom transformers" — i.e. exactly the
//!     workloads that reach for SIMD. The wrong answer was on the path we
//!     explicitly invited that workload onto.
//!
//! The justification "current corpus doesn't use them" was the actual
//! error in reasoning: the corpus is OUR trusted test input, whereas the
//! guest module is the UNTRUSTED party. What our fixtures happen to
//! return says nothing about what a guest may return.
//!
//! # What is pinned here
//!
//! End-to-end, through the production `sandbox::execute` surface. The
//! encoder is additionally pinned directly (including `externref` /
//! `anyref`, which no module can produce at this build's wasmtime
//! feature set because the `gc` cargo feature is off) by the unit tests
//! in `primitives::sandbox::tests`.
//!
//! # Falsification
//!
//! Restore the historical catch-all `_ => out.extend_from_slice(&[0u8; 16])`
//! in `encode_return_values`, drop the `?` at its call site, and delete
//! the pre-call ABI gate in `execute_with_live_cap_check`. Every test in
//! this file then fails.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use benten_core::Cid;
use benten_eval::AttributionFrame;
use benten_eval::sandbox::{ManifestRef, ManifestRegistry, SandboxConfig, SandboxError, execute};

fn attribution() -> AttributionFrame {
    let zero = Cid::from_blake3_digest([0u8; 32]);
    AttributionFrame {
        actor_cid: zero,
        handler_cid: zero,
        capability_grant_cid: zero,
        sandbox_depth: 0,
        ..Default::default()
    }
}

fn grant() -> Vec<String> {
    vec![
        "host:compute:log".to_string(),
        "host:compute:time".to_string(),
    ]
}

/// Drive a WAT module through the production SANDBOX executor.
fn run_wat(src: &str) -> Result<benten_eval::sandbox::SandboxResult, SandboxError> {
    let bytes = wat::parse_str(src).expect("WAT fixture must parse");
    execute(
        &bytes,
        ManifestRef::named("compute-basic"),
        &ManifestRegistry::new(),
        SandboxConfig::default(),
        &grant(),
        &attribution(),
    )
}

/// Assert the executor rejected the module, naming the offending type,
/// and routed to the stable catalog code.
fn assert_rejected_naming(
    res: Result<benten_eval::sandbox::SandboxResult, SandboxError>,
    ty: &str,
) {
    match res {
        Ok(ok) => panic!(
            "SANDBOX returned Ok for an unencodable `{ty}` result — the caller was handed \
             fabricated bytes {:?}",
            ok.output
        ),
        Err(SandboxError::ModuleInvalid { ref reason }) => {
            assert!(
                reason.contains(ty),
                "reason must name the offending type `{ty}`: {reason}"
            );
            assert_eq!(
                res.as_ref().unwrap_err().code(),
                benten_errors::ErrorCode::SandboxModuleInvalid,
            );
        }
        Err(other) => panic!("expected ModuleInvalid naming `{ty}`, got {other:?}"),
    }
}

/// THE flagship pin. A guest returning a non-zero SIMD vector must not
/// receive `Ok` with sixteen zero bytes.
///
/// FALSIFICATION: restore the `[0u8; 16]` catch-all + delete the ABI
/// gate. Observed failure then:
///   `SANDBOX returned Ok for an unencodable `v128` result — the caller
///    was handed fabricated bytes [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`
#[test]
fn sandbox_v128_return_is_rejected_not_silently_zeroed() {
    let res = run_wat(
        r#"(module (func (export "run") (result v128)
             v128.const i32x4 0x11111111 0x22222222 0x33333333 0x44444444))"#,
    );
    assert_rejected_naming(res, "v128");
}

/// `funcref` is a per-Store handle. Its bytes are meaningless in ANY
/// ABI — and doubly so here, because the per-call Store is dropped on
/// return (D3-RESOLVED), so any handle handed back is already dangling.
/// Empirically reachable at this build's feature set.
#[test]
fn sandbox_funcref_return_is_rejected() {
    let res = run_wat(r#"(module (func (export "run") (result funcref) ref.null func))"#);
    assert_rejected_naming(res, "funcref");
}

/// A non-null `funcref` — same rejection. Guards against a "null refs
/// are fine to zero" narrowing.
#[test]
fn sandbox_non_null_funcref_return_is_rejected() {
    let res = run_wat(
        r#"(module (func $f) (elem declare func $f)
             (func (export "run") (result funcref) ref.func $f))"#,
    );
    assert_rejected_naming(res, "funcref");
}

/// The nastiest historical shape: a multi-value return whose FIRST
/// result encodes correctly and whose tail does not. Pre-fix this
/// produced `[7,0,0,0] ++ [0u8; 16]` — a plausible-looking PARTIALLY
/// correct answer, with the correct `i32` lending false credibility to
/// sixteen invented bytes. A consumer reading the leading scalar has no
/// way to detect the tail was fabricated.
#[test]
fn sandbox_multi_value_v128_tail_is_rejected_not_partially_fabricated() {
    let res = run_wat(
        r#"(module (func (export "run") (result i32 v128)
             i32.const 7
             v128.const i32x4 0x01010101 0x02020202 0x03030303 0x04040404))"#,
    );
    assert_rejected_naming(res, "v128");
}

/// The rejection is a PRE-CALL gate, not merely a post-call backstop:
/// the guest body must never run.
///
/// This module's body is `unreachable`, so it traps immediately if it is
/// ever entered. If the ABI gate fires first we get `ModuleInvalid`
/// naming `v128`; if only the encoder backstop existed, the trap would
/// win and the error would be the trap mapping instead.
///
/// This matters beyond tidiness: by the time the encoder runs, the guest
/// has already had its observable side effects (host-fn `log` writes,
/// entropy draws, fuel burn). Rejecting up front makes the failure
/// side-effect-free.
///
/// FALSIFICATION: delete the ABI gate loop in
/// `execute_with_live_cap_check`, keeping only the encoder backstop —
/// this test then fails with the trap error instead of `ModuleInvalid`.
#[test]
fn sandbox_unencodable_result_rejected_before_guest_body_runs() {
    let res = run_wat(r#"(module (func (export "run") (result v128) unreachable))"#);
    assert_rejected_naming(res, "v128");
}

/// REGRESSION GUARD for the other half of the change: every result type
/// the ABI *can* encode still succeeds, byte-identically. The fix is
/// additive — it converts fabricated answers into typed errors and
/// changes nothing else. If this fails, a golden somewhere moved.
///
/// FALSIFICATION: swap `to_le_bytes()` for `to_be_bytes()` in
/// `encode_return_values`, or add `ValType::V128` to
/// `val_type_is_encodable` — the first breaks these assertions, the
/// second breaks the pins above.
#[test]
fn sandbox_scalar_returns_are_unchanged_and_byte_identical() {
    let i32_out = run_wat(r#"(module (func (export "run") (result i32) i32.const 42))"#)
        .expect("i32 return must still succeed");
    assert_eq!(i32_out.output, vec![42, 0, 0, 0]);

    let i64_out = run_wat(r#"(module (func (export "run") (result i64) i64.const 1))"#)
        .expect("i64 return must still succeed");
    assert_eq!(i64_out.output, vec![1, 0, 0, 0, 0, 0, 0, 0]);

    let f32_out = run_wat(r#"(module (func (export "run") (result f32) f32.const 1.0))"#)
        .expect("f32 return must still succeed");
    assert_eq!(f32_out.output, vec![0, 0, 128, 63]);

    let f64_out = run_wat(r#"(module (func (export "run") (result f64) f64.const 1.0))"#)
        .expect("f64 return must still succeed");
    assert_eq!(f64_out.output, vec![0, 0, 0, 0, 0, 0, 240, 63]);

    let void_out =
        run_wat(r#"(module (func (export "run")))"#).expect("no-result return must still succeed");
    assert!(void_out.output.is_empty());
}
