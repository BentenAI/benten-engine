//! R3-D RED-PHASE pins for §7.3.A.7 testing-helper cfg-gating audit
//! (G17-A1 wave-5b; r1-wsa-6 LOAD-BEARING).
//!
//! Pin sources (per r2-test-landscape §2.5 G17-A1 + r1-wasmtime-sandbox r1-wsa-6):
//!
//! - `tests/sandbox_testing_helpers_cfg_gated_no_production_attack_surface_widening`
//!   — r1-wsa-6 LOAD-BEARING (the headline security pin)
//! - `tests/test_helpers_feature_flag_off_in_default_features` — r1-wsa-6
//! - `tests/cfg_test_path_only_visible_in_test_builds` — r1-wsa-6
//!
//! ## What this audits
//!
//! G17-A1 wave-5b ships the §7.3.A.7 testing-helper SURFACE in
//! `crates/benten-eval/src/sandbox/testing_helpers.rs`:
//!
//! - `testing_revoke_cap_mid_call`
//! - `testing_call_engine_dispatch`
//! - `testing_inject_forged_cap_claim_section`
//! - `testing_register_uncounted_host_fn`
//!
//! These helpers EXIST to drive ESC test bodies (G20-A1 un-ignores
//! the dependent test bodies). They MUST NOT widen production attack
//! surface — they are gated behind `cfg(any(test, feature = "test-helpers"))`
//! per r1-wsa-6.
//!
//! ## Three-pin shape (per pim-2 §3.6b end-to-end test pin requirement)
//!
//! - `..._no_production_attack_surface_widening` — composite pin:
//!   asserts BOTH cfg-gating mechanisms are present (cfg-attr at the
//!   helper module + feature flag default-off in Cargo.toml). Per
//!   Phase-2a sec-r6r2-02 precedent.
//! - `test_helpers_feature_flag_off_in_default_features` — Cargo.toml
//!   discipline pin: the `test-helpers` feature flag is NOT in the
//!   `default = [...]` array.
//! - `cfg_test_path_only_visible_in_test_builds` — cfg-attr discipline
//!   pin: the `testing_helpers` module/items are gated with
//!   `cfg(any(test, feature = "test-helpers"))`, NOT with `cfg(debug)`
//!   or unconditionally exposed.
//!
//! Mirrors Phase-2a sec-r6r2-02 cfg-gating audit precedent. Defends
//! against a regression where a refactor accidentally unconditionally
//! exposes a testing helper to production builds — which would let an
//! attacker forge cap-claim sections or bypass cap accounting.

#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

#[test]
#[ignore = "RED-PHASE: G17-A1 wave-5b ships testing_helpers.rs cfg-gated under cfg(any(test, feature = \"test-helpers\"))"]
fn sandbox_testing_helpers_cfg_gated_no_production_attack_surface_widening() {
    // r1-wsa-6 LOAD-BEARING composite pin. G17-A1 implementer wires:
    //
    //   let testing_src = std::fs::read_to_string(
    //       "crates/benten-eval/src/sandbox/testing_helpers.rs"
    //   ).unwrap();
    //
    //   // cfg-attr present at module level OR every fn level:
    //   assert!(testing_src.contains(
    //       "cfg(any(test, feature = \"test-helpers\"))"
    //   ),
    //       "testing_helpers.rs MUST gate every helper behind cfg(any(test, feature = \"test-helpers\")) per r1-wsa-6 LOAD-BEARING");
    //
    //   // Cargo.toml feature flag exists + is NOT in default:
    //   let manifest = std::fs::read_to_string("crates/benten-eval/Cargo.toml").unwrap();
    //   assert!(manifest.contains("test-helpers"),
    //       "benten-eval Cargo.toml MUST declare the test-helpers feature flag");
    //
    //   // The default features array excludes test-helpers:
    //   let default_section = extract_features_default_section(&manifest);
    //   assert!(!default_section.contains("test-helpers"),
    //       "test-helpers MUST NOT be in default features per r1-wsa-6 + Phase-2a sec-r6r2-02 precedent");
    //
    //   // Each helper fn name is referenced ONLY from cfg-gated paths:
    //   for helper in &[
    //       "testing_revoke_cap_mid_call",
    //       "testing_call_engine_dispatch",
    //       "testing_inject_forged_cap_claim_section",
    //       "testing_register_uncounted_host_fn",
    //   ] {
    //       // Either the module is wholly gated, or each fn is gated.
    //       assert!(testing_src.contains(helper),
    //           "Expected helper {} present in testing_helpers.rs", helper);
    //   }
    //
    // OBSERVABLE consequence: a future refactor that drops the cfg-
    // gating (e.g. for "debugging convenience") fails this pin AND
    // the implementer's PR review (security-auditor brief explicitly
    // includes cfg-gating audit per Phase-2a sec-r6r2-02 precedent).
    //
    // This is the LOAD-BEARING headline pin for §7.3.A.7.
    unimplemented!(
        "G17-A1 wires composite cfg-gating audit assertion across testing_helpers.rs + Cargo.toml"
    );
}

#[test]
#[ignore = "RED-PHASE: G17-A1 wave-5b — Cargo.toml test-helpers feature off by default"]
fn test_helpers_feature_flag_off_in_default_features() {
    // r1-wsa-6 Cargo-discipline pin. G17-A1 implementer:
    //
    //   let manifest = std::fs::read_to_string("crates/benten-eval/Cargo.toml").unwrap();
    //
    //   // [features] section parsed:
    //   let features_section = manifest.find("[features]").expect("[features] section present");
    //   let after = &manifest[features_section..];
    //   let default_line_idx = after.find("default ").or_else(|| after.find("default=")).expect("default = ... line");
    //   let default_line_end = after[default_line_idx..].find('\n').unwrap();
    //   let default_line = &after[default_line_idx..default_line_idx + default_line_end];
    //
    //   assert!(!default_line.contains("test-helpers"),
    //       "Cargo.toml [features] default array MUST NOT include test-helpers per r1-wsa-6");
    //
    // OBSERVABLE consequence: a `cargo build -p benten-eval` (default
    // features only, as runs in production CI + downstream consumer
    // builds) does NOT compile testing_helpers.rs items. Defends r1-wsa-6.
    unimplemented!("G17-A1 wires Cargo.toml [features] default-off discipline assertion");
}

#[test]
#[ignore = "RED-PHASE: G17-A1 wave-5b — napi cdylib production-build symbol-table assertion (r4-r1-wsa-3 LOAD-BEARING half of pim-2 §3.6b shape)"]
fn napi_cdylib_production_build_does_not_export_testing_helper_symbols() {
    // r4-r1-wsa-3 LOAD-BEARING pin (the production-flow half of the
    // pim-2 §3.6b end-to-end shape). The 3 source-cite pins above
    // catch the easy regression (cfg-attr deletion); THIS pin catches
    // the harder regression (Cargo feature-graph composition that
    // accidentally activates benten-eval's `test-helpers` feature
    // transitively through bindings/napi's production feature set).
    //
    // Phase-2a sec-r6r2-02 precedent named this exact regression vector:
    // a refactor that adds (e.g.) `bindings/napi.test-helpers = ["benten-eval/test-helpers"]`
    // to enable broader testing, but accidentally lists this in the
    // [features] default array of bindings/napi/Cargo.toml. The 3
    // source-cite pins still pass (cfg-attrs unchanged in the .rs files);
    // the napi cdylib silently exposes the testing helpers in production.
    //
    // G17-A1 implementer wires this:
    //
    //   // Step 1: invoke the production cdylib build:
    //   //
    //   //   cargo build -p benten-napi --release \
    //   //     --no-default-features \
    //   //     --features <production-feature-set>   // implementer pins exact set
    //   //
    //   // Step 2: locate the cdylib artifact:
    //   //
    //   //   let artifact = std::path::PathBuf::from("target/release/libbenten_napi.dylib")
    //   //       .or("target/release/libbenten_napi.so")
    //   //       .or("target/release/benten_napi.dll");
    //   //   if !artifact.exists() {
    //   //       eprintln!("skip: napi cdylib production build artifact absent");
    //   //       return;
    //   //   }
    //   //
    //   // Step 3: scan the symbol table for testing-helper symbols. On
    //   // macOS/Linux: `nm -gU <artifact>`; on Windows: `dumpbin /exports`.
    //   // The 4 testing-helper functions:
    //   const TESTING_HELPER_SYMBOLS: &[&str] = &[
    //       "testing_revoke_cap_mid_call",
    //       "testing_call_engine_dispatch",
    //       "testing_inject_forged_cap_claim_section",
    //       "testing_register_uncounted_host_fn",
    //   ];
    //
    //   //   let nm_output = std::process::Command::new("nm")
    //   //       .arg("-gU").arg(&artifact).output().unwrap();
    //   //   let symbols = String::from_utf8_lossy(&nm_output.stdout);
    //   //   for helper in TESTING_HELPER_SYMBOLS {
    //   //       assert!(!symbols.contains(helper),
    //   //           "napi cdylib production build MUST NOT export testing helper {} \
    //   //            per r4-r1-wsa-3 + Phase-2a sec-r6r2-02; check Cargo feature-graph \
    //   //            composition — bindings/napi production feature set must NOT \
    //   //            transitively activate benten-eval's test-helpers feature",
    //   //           helper);
    //   //   }
    //
    //   // Step 4 (sibling — Cargo feature-graph drift assertion):
    //   //
    //   //   let napi_cargo = std::fs::read_to_string("bindings/napi/Cargo.toml").unwrap();
    //   //   // The dependencies/features tables MUST NOT transitively
    //   //   // enable benten-eval/test-helpers from any default-active path:
    //   //   //   - benten-eval = { ..., features = ["test-helpers"] }       FORBIDDEN at default
    //   //   //   - default = ["...", "..."] with paths reaching test-helpers
    //   //   //
    //   //   // Cleanest shape: parse the manifest with toml-rs + walk the
    //   //   // feature graph reachable from the default array; assert the
    //   //   // closure does NOT include "benten-eval/test-helpers".
    //   //
    //   // OBSERVABLE consequence: a Cargo refactor that accidentally
    //   // pulls test-helpers into the production cdylib symbol table
    //   // fails this pin. Defends against the harder regression vector
    //   // (feature-graph composition) the source-cite pins miss.
    //
    // Pairs with the source-cite trio above to compose the pim-2
    // end-to-end shape: source-cite (cfg-attr presence) + Cargo
    // feature-graph + production-build symbol-table assertion. This
    // pin is the LOAD-BEARING half per r4-r1-wsa-3 — the source-cite
    // alone is insufficient.
    unimplemented!(
        "G17-A1 wires napi cdylib production-build symbol-table scan + Cargo feature-graph closure assertion per r4-r1-wsa-3 LOAD-BEARING (pim-2 production-flow half)"
    );
}

#[test]
#[ignore = "RED-PHASE: G17-A1 wave-5b — cfg attribute discipline (not cfg(debug), not unconditional)"]
fn cfg_test_path_only_visible_in_test_builds() {
    // r1-wsa-6 cfg-attr discipline pin. G17-A1 implementer:
    //
    //   let testing_src = std::fs::read_to_string(
    //       "crates/benten-eval/src/sandbox/testing_helpers.rs"
    //   ).unwrap();
    //
    //   // ANTI-PATTERNS (none of these are acceptable substitutes):
    //   //
    //   //   #[cfg(debug_assertions)] — wrong: leaks in release+debug profile
    //   //   #[cfg(not(production))]  — non-existent; cfg-key is meaningless
    //   //   pub fn testing_*() {...} — wrong: unconditional public surface
    //
    //   // Specifically rule out cfg(debug_assertions):
    //   //
    //   // (Allow it ONLY if it's also conjunctively gated with test/test-helpers.)
    //   let bad_pattern_count = testing_src
    //       .matches("#[cfg(debug_assertions)]")
    //       .count();
    //   assert_eq!(bad_pattern_count, 0,
    //       "testing_helpers.rs MUST NOT use cfg(debug_assertions) as the gate; \
    //        use cfg(any(test, feature = \"test-helpers\")) per r1-wsa-6");
    //
    //   // Also rule out unconditional public surface — every helper
    //   // must be either inside an outer cfg-block or each fn cfg-gated.
    //   //   (heuristic: scan for `pub fn testing_` not preceded by a
    //   //    cfg-attr within the prior 3 lines)
    //
    // OBSERVABLE consequence: a refactor that converts
    // `cfg(any(test, feature = "test-helpers"))` to
    // `cfg(debug_assertions)` looks plausibly correct ("only debug
    // builds") but accidentally exposes the helpers to release-with-
    // debug-assertions builds (which production sometimes uses for
    // tracing). This pin fails on that regression.
    unimplemented!(
        "G17-A1 wires cfg-attr discipline assertion (rules out debug_assertions / unconditional)"
    );
}
