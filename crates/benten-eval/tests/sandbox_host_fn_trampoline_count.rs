//! Phase 2b R3-B — D25 trampoline-counts unit tests (G7-A).
//!
//! D25-RESOLVED — host-fn output bytes counted at the codegen-emitted
//! TRAMPOLINE (centralized accounting; one place to audit), NOT in the
//! host-fn body. Body never touches the counter directly.
//!
//! This is the implementation default; host-fns that need to bypass the
//! output budget (NONE in 2b's D1 surface) declare
//! `bypass_output_budget = true` in host-functions.toml.
//!
//! **G20-A1 wave-8a** (Phase 3): bodies un-ignored. Drive
//! `benten_eval::sandbox::execute` with a 3-call log loop and assert
//! the SandboxResult.output_consumed reflects accumulated trampoline-
//! counted bytes (the centralised accounting claim). The bypass-field
//! default-false test pins host-functions.toml schema invariant.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use benten_core::Cid;
use benten_eval::AttributionFrame;
use benten_eval::sandbox::{
    ManifestRef, ManifestRegistry, SandboxConfig, default_host_fns, execute,
};

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
fn sandbox_host_fn_output_bytes_counted_at_trampoline_not_body() {
    // D25-RESOLVED — emitter is the trampoline. Drive 3 successive
    // log calls @ 60 KiB each + assert the SandboxResult's
    // `output_consumed` reflects ≥ 180 KiB. The trampoline-side
    // CountedSink is what increments the counter (not the host-fn
    // body). A regression where the host-fn body pre-counted bytes
    // (or the trampoline silently no-op'd accounting) would surface
    // as `output_consumed == 0` even after successful calls.
    let bytes = wat::parse_str(
        "(module
           (import \"host\" \"log\" (func $log (param i32 i32)))
           (memory (export \"memory\") 4)
           (func (export \"run\") (result i32)
             (local $i i32)
             (loop $L
               i32.const 0
               i32.const 60000
               call $log
               local.get $i
               i32.const 1
               i32.add
               local.tee $i
               i32.const 3
               i32.lt_s
               br_if $L
             )
             local.get $i
           )
         )",
    )
    .unwrap();
    let registry = ManifestRegistry::new();
    let cfg = SandboxConfig {
        output_bytes: 1024 * 1024,
        fuel: 100_000_000,
        ..SandboxConfig::default()
    };
    let attribution = dummy_attribution();
    let res = execute(
        &bytes,
        ManifestRef::named("compute-basic"),
        &registry,
        cfg,
        &[
            "host:compute:log".to_string(),
            "host:compute:time".to_string(),
        ],
        &attribution,
    )
    .expect("3 log calls @ 60K under 1MB budget MUST succeed");
    assert!(
        res.output_consumed >= 180_000,
        "trampoline-counted bytes MUST reflect 3 log calls × 60K = \
         180_000 bytes accumulated; got output_consumed={} (a \
         regression where the body pre-counted or the trampoline \
         silently no-op'd would surface as a smaller / zero number)",
        res.output_consumed
    );

    // STRUCTURAL pin: the trampoline is the centralised accounting
    // site. Source-grep at primitives/sandbox.rs to confirm the
    // trampoline writes to a CountedSink (rather than each host-fn
    // body holding its own counter).
    let exec_src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("primitives")
            .join("sandbox.rs"),
    )
    .expect("benten-eval/src/primitives/sandbox.rs must be readable");
    assert!(
        exec_src.contains("CountedSink"),
        "the SANDBOX trampoline MUST consult the CountedSink primary \
         path (D25 centralised accounting); the symbol must appear in \
         primitives/sandbox.rs"
    );
}

#[test]
fn sandbox_host_fn_bypass_output_budget_field_default_false() {
    // D25 — host-functions.toml field `bypass_output_budget: bool`
    // defaults to `false`. NONE of the D1 initial surface
    // (time / log / kv:read / random) sets it to `true`.
    //
    // Pin via codegen-emitted table (the canonical D1 surface).
    let table = default_host_fns();
    for (name, spec) in table.iter() {
        assert!(
            !spec.bypass_output_budget,
            "D25 — D1 surface host-fn {name:?} MUST have \
             bypass_output_budget = false; a future PR setting this \
             true requires explicit security review"
        );
    }

    // Schema-pin via host-functions.toml: confirm at least one
    // explicit `bypass_output_budget = false` declaration is present
    // (parseability) AND no `bypass_output_budget = true` line.
    let toml_src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("host-functions.toml"),
    )
    .expect("workspace host-functions.toml readable");
    assert!(
        toml_src.contains("bypass_output_budget = false"),
        "host-functions.toml MUST carry at least one explicit \
         `bypass_output_budget = false` declaration (parseability pin)"
    );
    assert!(
        !toml_src.contains("bypass_output_budget = true"),
        "Phase-2b host-functions.toml MUST NOT carry any \
         `bypass_output_budget = true` declaration (D25 default-false)"
    );
}
