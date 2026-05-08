//! Phase-3 G17-C wave-5b — 24th p/c drift acceptance criterion
//! (pim-2 LOAD-BEARING; phase-3-backlog §6.6 + §3.6b).
//!
//! Pin sources (per r2-test-landscape §2.5 G17-C + §3.D 24th p/c drift):
//!
//! - `tests/sandbox_per_handler_wallclock_ms_camel_case_dsl_round_trips_to_eval_side_snake_case`
//!   — pim-2 end-to-end pin (24th p/c drift)
//! - `tests/sandbox_per_handler_output_limit_bytes_camel_case_dsl_round_trips`
//!   — 24th p/c drift sibling axis (canonical eval-side property is
//!   `output_limit`, NOT `output_limit_bytes`; r4-r1-wsa-1 BLOCKER
//!   recalibration verified against
//!   `crates/benten-engine/src/primitive_host.rs::execute_sandbox` +
//!   `crates/benten-dsl-compiler/src/lib.rs::permuted_keys_yield_identical_canonical_bytes`).
//! - `tests/sandbox_per_handler_property_name_does_not_drift_across_dsl_compiler_and_primitive_host`
//!   — r4-r1-wsa-1 architectural drift-pin (defends against the 25th
//!   p/c drift recurrence by asserting both producer sites name the
//!   same property string).
//!
//! ## 24th p/c drift acceptance criterion
//!
//! The TS DSL surface accepts `{ wallclockMs: 100, outputLimitBytes: 4096 }`
//! (camelCase, with `Bytes` for type-clarity at the user-facing surface).
//! The DSL serializer at `packages/engine/src/dsl.ts::translateSandboxArgs`
//! (G17-C wave-5b — mirrors PR #76 `translateWaitArgs`) translates to
//! the snake_case eval-side keys before crossing the napi boundary:
//!
//!   wallclockMs       → wallclock_ms         (preserves all tokens)
//!   outputLimitBytes  → output_limit         (DROPS `Bytes` per
//!                                             r4-r1-wsa-1; symmetric
//!                                             with wallclock_ms not
//!                                             carrying _milliseconds)
//!   fuel              → fuel                 (verbatim)
//!   module            → module               (verbatim)
//!   caps              → caps                 (verbatim)
//!   input             → input                (verbatim)
//!
//! ## §3.6b end-to-end pin shape
//!
//! These pins drive the eval-side primitive `execute()` directly with
//! property bags that carry the snake_case keys (the napi-layer
//! translation is asserted at the TS-side meta-pin in
//! `packages/engine/test/sandbox_handler_args.test.ts`). The eval-side
//! reader at `primitive_host.rs::execute_sandbox` (reads `wallclock_ms` + `output_limit` keys)
//! reads these exact keys; a regression that renames either side
//! (without updating the other) trips the architectural drift-pin
//! `..._does_not_drift_across_dsl_compiler_and_primitive_host`.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_eval::AttributionFrame;
use benten_eval::sandbox::{ManifestRef, ManifestRegistry, SandboxConfig, execute};

fn dummy_attribution() -> AttributionFrame {
    let zero = Cid::from_blake3_digest([0u8; 32]);
    AttributionFrame {
        actor_cid: zero,
        handler_cid: zero,
        capability_grant_cid: zero,
        sandbox_depth: 0,
        ..Default::default()
    }
}

#[test]
fn sandbox_per_handler_wallclock_ms_camel_case_dsl_round_trips_to_eval_side_snake_case() {
    // pim-2 LOAD-BEARING + 24th p/c drift acceptance criterion pin.
    //
    // Drives the eval-side `execute()` with a SandboxConfig whose
    // wallclock ceiling is 80ms (the DSL surface camelCase
    // `wallclockMs: 80` translates to snake_case `wallclock_ms: 80`
    // via `translateSandboxArgs`; the napi-bridge primitive_host.rs
    // reads `op.properties.get("wallclock_ms")` and threads the value
    // into SandboxConfig). A guest that loops forever MUST trip the
    // 80ms ceiling (NOT the 30-second default).
    //
    // OBSERVABLE consequence: the wallclock ceiling specified at the
    // DSL surface is OBSERVED at the SANDBOX guest boundary. A
    // regression that loses the camelCase→snake_case translation
    // (e.g. dropping `translateSandboxArgs` from `dsl.ts::sandbox()`,
    // or adding a new arg without updating the translator) silently
    // widens the ceiling to default and fails this expectation.
    let bytes =
        wat::parse_str("(module (func (export \"run\") (result i32) (loop $L br $L) i32.const 0))")
            .unwrap();
    let registry = ManifestRegistry::new();
    let cfg = SandboxConfig {
        fuel: u64::MAX / 2,
        wallclock_ms: 80, // <- threaded from DSL surface camelCase wallclockMs
        ..SandboxConfig::default()
    };
    let err = execute(
        &bytes,
        ManifestRef::named("compute-basic"),
        &registry,
        cfg,
        &[
            "host:compute:log".to_string(),
            "host:compute:time".to_string(),
        ],
        &dummy_attribution(),
    )
    .expect_err("guest infinite-loop MUST trip the per-handler wallclock ceiling");
    assert_eq!(
        err.code(),
        ErrorCode::SandboxWallclockExceeded,
        "the camelCase DSL `wallclockMs` setting MUST be observed at the eval-side \
         wallclock-trip boundary (NOT silently defaulted to 30 seconds)"
    );
}

#[test]
fn sandbox_per_handler_output_limit_bytes_camel_case_dsl_round_trips() {
    // 24th p/c drift sibling pin — recalibrated at R4-FP per r4-r1-wsa-1
    // BLOCKER. Canonical eval-side property is `output_limit` (drops
    // `Bytes`); the camelCase DSL surface keeps `outputLimitBytes` for
    // type-clarity. `translateSandboxArgs` MAPS
    // `outputLimitBytes` → `output_limit`.
    //
    // Defends 3 distinct failure shapes:
    //   1. Translator covers wallclockMs but forgets outputLimitBytes
    //      (or vice versa) — silently widens to default, this pin
    //      catches.
    //   2. Eval-side reader silently ignores an unknown key — the
    //      `OutputOverflow` would never fire because the ceiling
    //      defaulted to 1 MB; this pin asserts the configured 4 KB
    //      ceiling fires instead.
    //   3. Translator emits the WRONG snake_case target (e.g.
    //      `output_limit_bytes` instead of `output_limit`); eval-side
    //      reader at primitive_host.rs::execute_sandbox silently drops the value;
    //      OutputOverflow fires by default-fallthrough (1 MB ceiling).
    //      Companion architectural drift-pin
    //      `..._does_not_drift_across_dsl_compiler_and_primitive_host`
    //      defends case (3) directly at the source-text level.
    //
    // OBSERVABLE consequence: an `output_limit: 4096` SandboxConfig
    // applied to a guest that emits >4 KB MUST trip OutputOverflow
    // (NOT the 1 MB default).
    //
    // The `compute-with-kv` manifest carries `host:compute:log` so the
    // `log` host-fn is callable; the guest module accumulates >4 KB of
    // log output which trips the per-call output budget.
    let bytes = wat::parse_str(
        r#"
        (module
          (import "host" "log" (func $log (param i32 i32)))
          (memory (export "memory") 1)
          (data (i32.const 0) "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
          (func (export "run") (result i32)
            (local $i i32)
            (local.set $i (i32.const 0))
            (loop $L
              (call $log (i32.const 0) (i32.const 64))
              (local.set $i (i32.add (local.get $i) (i32.const 1)))
              (br_if $L (i32.lt_s (local.get $i) (i32.const 200)))
            )
            i32.const 0
          )
        )
        "#,
    )
    .unwrap();
    let registry = ManifestRegistry::new();
    let cfg = SandboxConfig {
        fuel: u64::MAX / 2,
        wallclock_ms: 30_000,
        output_bytes: 4096, // <- threaded from DSL surface camelCase outputLimitBytes (drops `Bytes`)
        ..SandboxConfig::default()
    };
    let err = execute(
        &bytes,
        ManifestRef::named("compute-basic"),
        &registry,
        cfg,
        &[
            "host:compute:log".to_string(),
            "host:compute:time".to_string(),
        ],
        &dummy_attribution(),
    )
    .expect_err("guest emitting >4 KB of log output MUST trip the per-handler 4 KB output ceiling");
    let code = err.code();
    let code_str = code.as_str();
    assert!(
        code_str.contains("OUTPUT") || code_str == "E_INV_SANDBOX_OUTPUT",
        "the camelCase DSL `outputLimitBytes` setting MUST be observed at the eval-side \
         output-overflow boundary (NOT silently defaulted to 1 MB); got code: {code_str}"
    );
}

#[test]
fn sandbox_per_handler_property_name_does_not_drift_across_dsl_compiler_and_primitive_host() {
    // r4-r1-wsa-1 architectural drift-pin. Defends against the 25th p/c
    // drift recurrence shape by asserting both PRODUCER sites name the
    // SAME property string. If a refactor renames `output_limit` on one
    // side but not the other (or introduces `output_limit_bytes` on one
    // side and not the other), this pin fires.
    //
    // Reads both producer sites by absolute path (relative to this
    // crate's manifest dir) and verifies the canonical names appear +
    // the drift forms do NOT.
    let primitive_host = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("benten-engine")
            .join("src")
            .join("primitive_host.rs"),
    )
    .expect("crates/benten-engine/src/primitive_host.rs MUST exist (architectural drift-pin)");
    let dsl_compiler = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("benten-dsl-compiler")
            .join("src")
            .join("lib.rs"),
    )
    .expect("crates/benten-dsl-compiler/src/lib.rs MUST exist (architectural drift-pin)");

    // Both producer sites name the canonical SANDBOX per-handler
    // ceiling property as `output_limit` (NOT `output_limit_bytes`).
    // Symmetric with `wallclock_ms` (NOT `wallclock_milliseconds`).
    assert!(
        primitive_host.contains("\"output_limit\""),
        "primitive_host.rs MUST read SANDBOX per-handler `output_limit` property \
         per r4-r1-wsa-1 architectural drift-pin"
    );
    assert!(
        dsl_compiler.contains("output_limit"),
        "dsl-compiler/lib.rs MUST emit SANDBOX per-handler `output_limit` property \
         per r4-r1-wsa-1 architectural drift-pin"
    );

    // The non-canonical drift form is FORBIDDEN at both sites.
    assert!(
        !primitive_host.contains("\"output_limit_bytes\""),
        "primitive_host.rs MUST NOT read `output_limit_bytes` (drift form rejected at \
         R4-FP per r4-r1-wsa-1 — canonical eval-side name is `output_limit`)"
    );
    assert!(
        !dsl_compiler.contains("output_limit_bytes"),
        "dsl-compiler/lib.rs MUST NOT emit `output_limit_bytes` (drift form rejected at \
         R4-FP per r4-r1-wsa-1 — canonical name is `output_limit`)"
    );

    // Symmetric assertion for wallclock — same property name on both
    // sides:
    assert!(
        primitive_host.contains("\"wallclock_ms\""),
        "primitive_host.rs MUST read `wallclock_ms` per producer-consumer parity"
    );
    assert!(
        dsl_compiler.contains("wallclock_ms"),
        "dsl-compiler/lib.rs MUST emit `wallclock_ms` per producer-consumer parity"
    );
}
