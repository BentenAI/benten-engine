//! R3-E RED-PHASE pins for G20-A3 cargo-vet policy + cargo-public-api
//! existing-crate baselines (wave-8a; §7.3.A.9 + sec-r1-5).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.8 G20-A3 +
//! `.addl/phase-3/00-implementation-plan.md` §3 G20-A3 must-pass column):
//!
//! - `tests/cargo_vet_policy_self_test_un_ignored` — §7.3.A.9
//! - `tests/cargo_vet_exemption_budget_at_or_below_5_at_phase_3_close` — sec-r1-5
//! - `tests/cargo_public_api_drift_un_ignored` — §7.3.A.9
//! - `tests/browser_target_bundle_size_un_ignored` — §7.3.A.9
//! - `tests/suspension_store_round_trip_subscription_cursor_un_ignored` — §7.3.A.9
//! - `tests/inspect_state_pretty_prints_un_ignored` — §6.9
//!
//! ## What G20-A3 establishes (§7.3.A.9 + sec-r1-5)
//!
//! cargo-vet onboarding policy lands; existing-crate cargo-public-api
//! baselines committed; per-NEW-crate baselines (G14-A1 benten-id;
//! G16-A benten-sync) committed at the wave that creates each crate
//! per seq-minor-5.
//!
//! Per sec-r1-5: cargo-vet exemption-budget = 5 entries max at
//! Phase-3-close; criteria-set = `safe-to-deploy` (default) +
//! `crypto-reviewed` (manual for benten-id deps); periodic-policy-review
//! cadence quarterly.
//!
//! ## RED-PHASE discipline
//!
//! Per r3-a-precedent: this lives in `tests/phase_3_workspace/` because
//! it audits cross-cutting CI / supply-chain state. R3-A owns Compromise
//! #12 fn; R3-E owns the cargo-vet policy + cargo-public-api fns
//! (disjoint test-fn ownership within shared file).

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G20-A3 wave-8a — cargo-vet policy self-test un-ignored (§7.3.A.9)"]
fn cargo_vet_policy_self_test_un_ignored() {
    // §7.3.A.9 G20-A3 pin. Implementer wires this:
    //
    //   // Read supply-chain/audits.toml + supply-chain/config.toml:
    //   let audits_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("..").join("..").join("supply-chain").join("audits.toml");
    //   assert!(audits_path.exists(),
    //       "G20-A3 must commit supply-chain/audits.toml at Phase-3 close");
    //
    //   let config_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("..").join("..").join("supply-chain").join("config.toml");
    //   assert!(config_path.exists(),
    //       "G20-A3 must commit supply-chain/config.toml at Phase-3 close");
    //
    //   let config = std::fs::read_to_string(&config_path).unwrap();
    //   // Criteria-set per sec-r1-5: safe-to-deploy + crypto-reviewed
    //   assert!(config.contains("safe-to-deploy"),
    //       "cargo-vet config must declare safe-to-deploy criteria");
    //
    // OBSERVABLE consequence: cargo-vet onboarding lands at Phase-3
    // close per §7.3.A.9 + sec-r1-5 policy.
    unimplemented!("G20-A3 wires cargo-vet policy self-test");
}

#[test]
#[ignore = "RED-PHASE: G20-A3 wave-8a — cargo-vet exemption budget ≤ 5 at Phase-3 close (sec-r1-5)"]
fn cargo_vet_exemption_budget_at_or_below_5_at_phase_3_close() {
    // sec-r1-5 budget pin. G20-A3 implementer wires this:
    //
    //   let exemptions_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("..").join("..").join("supply-chain").join("exemptions.toml");
    //   if exemptions_path.exists() {
    //       let exemptions = std::fs::read_to_string(&exemptions_path).unwrap();
    //       let count = exemptions.matches("[[exemptions").count();
    //       assert!(count <= 5,
    //           "cargo-vet exemption budget exceeded: {} > 5 (sec-r1-5)",
    //           count);
    //   }
    //
    // OBSERVABLE consequence: supply-chain hygiene budget enforced at
    // Phase-3 close.
    unimplemented!("G20-A3 wires cargo-vet exemption budget pin");
}

#[test]
#[ignore = "RED-PHASE: G20-A3 wave-8a — cargo-public-api drift workflow un-ignored (§7.3.A.9)"]
fn cargo_public_api_drift_un_ignored() {
    // §7.3.A.9 G20-A3 pin. Implementer wires this:
    //
    //   // Verify EXISTING-crate baselines are committed:
    //   let baseline_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("..").join("..").join("docs").join("public-api");
    //   assert!(baseline_dir.exists(),
    //       "docs/public-api/ must exist for cargo-public-api baselines");
    //
    //   // Each existing crate's baseline must be present (per the
    //   // workspace member list at Cargo.toml):
    //   for crate_name in &[
    //       "benten-errors", "benten-core", "benten-graph", "benten-ivm",
    //       "benten-caps", "benten-eval", "benten-engine", "benten-dsl-compiler",
    //       "benten-id", "benten-sync",
    //   ] {
    //       let baseline = baseline_dir.join(format!("{}.json", crate_name));
    //       assert!(baseline.exists(),
    //           "missing cargo-public-api baseline for {}", crate_name);
    //   }
    //
    // OBSERVABLE consequence: every workspace crate has a public-API
    // baseline; future drift fires CI workflow.
    unimplemented!("G20-A3 wires cargo-public-api drift pin");
}

#[test]
#[ignore = "RED-PHASE: G20-A3 wave-8a — browser-target bundle size un-ignored (§7.3.A.9)"]
fn browser_target_bundle_size_un_ignored() {
    // §7.3.A.9 G20-A3 closure pin. Implementer wires this:
    //
    //   // The bundle-size gate test at bindings/napi/tests/wasm_bundle_size.rs
    //   // currently has #[ignore] OR is informational-only. G20-A3
    //   // un-ignores it AND promotes to required.
    //   let bundle_test = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("..").join("..").join("bindings").join("napi")
    //       .join("tests").join("wasm_bundle_size.rs");
    //   let src = std::fs::read_to_string(&bundle_test).unwrap();
    //
    //   // Per HARD RULE rule-12: no Phase-3-destination #[ignore]
    //   // rationales remain after G20-A3:
    //   for line in src.lines() {
    //       if line.contains("#[ignore") && line.contains("Phase 3") {
    //           panic!("wasm_bundle_size.rs still has Phase-3-destination ignore: {}",
    //               line.trim());
    //       }
    //   }
    //
    // OBSERVABLE consequence: bundle-size gate is required + green.
    unimplemented!("G20-A3 wires browser-target bundle size un-ignore pin");
}

#[test]
#[ignore = "RED-PHASE: G20-A3 wave-8a — suspension store subscription cursor un-ignored (§7.3.A.9)"]
fn suspension_store_round_trip_subscription_cursor_un_ignored() {
    // §7.3.A.9 G20-A3 closure pin. Implementer wires the missing
    // subscribe-persistent-cursor helpers + un-ignores existing skips.
    //
    //   // Drive the production code path: open a SUBSCRIBE with a
    //   // persistent cursor; round-trip through suspension_store
    //   // across a process restart.
    //   //
    //   // Pre-G20-A3: the cursor helpers are #[ignore]'d
    //   // Phase-3-residuals.
    //   // Post-G20-A3: helpers wired, tests green.
    //
    // OBSERVABLE consequence: the §7.3.A.9 cursor sub-cluster fully
    // clears.
    unimplemented!("G20-A3 wires subscription cursor un-ignore + helper wiring");
}

#[test]
#[ignore = "RED-PHASE: G20-A3 wave-8a — benten-dev inspect-state CLI un-ignored (§6.9)"]
fn inspect_state_pretty_prints_un_ignored() {
    // §6.9 pin. G20-A3 implementer wires `tools/benten-dev/bin/benten-dev.mjs`
    // thin-CLI front-door + un-skips the 4 inspect_state_pretty_prints
    // tests.
    //
    //   let cli_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("..").join("..").join("tools").join("benten-dev")
    //       .join("bin").join("benten-dev.mjs");
    //   assert!(cli_path.exists(),
    //       "G20-A3 must commit benten-dev thin-CLI front-door");
    //
    //   // Drive the CLI to inspect-state command + verify pretty-printed
    //   // output (sentinel-presence + observable-consequence per §3.6b):
    //   let output = std::process::Command::new("node")
    //       .arg(&cli_path)
    //       .args(&["inspect-state", "--in-memory"])
    //       .output()
    //       .unwrap();
    //   assert!(output.status.success());
    //   assert!(!output.stdout.is_empty(),
    //       "inspect-state must produce pretty-printed output");
    //
    // OBSERVABLE consequence: §6.9 thin-CLI front-door lands; 4
    // inspect-state tests un-skipped.
    unimplemented!("G20-A3 wires benten-dev inspect-state CLI un-ignore");
}
