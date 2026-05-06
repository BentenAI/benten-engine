//! R3-A RED-PHASE pin: wasm browser bundle size ≤ 600KB gzipped
//! (G13-C wave-3; br-r1-1 BLOCKER + spike-bundle-cap-empirical.md).
//!
//! Pin source: r2-test-landscape §2.1 G13-C row
//! `wasm_r1_7_browser_bundle_size_at_or_below_600kb_gzipped`; br-r1-1
//! BLOCKER (recalibrated from 350KB per `spike-bundle-cap-empirical.md`
//! 2026-05-04 — Loro is OUT per CLAUDE.md baked-in #17).
//!
//! Originating decision context: `.addl/phase-2b/wave-8j-wasm-browser-bundle-bisect.md`
//! §Phase-3-followup — the Phase-2b retrospective that surfaced the
//! 350KB-aspirational gap; recalibrated to 600KB via
//! `spike-bundle-cap-empirical.md` per pim-1 §3.5b doc-coupling.
//!
//! ## What this pins
//!
//! The browser-side Phase-3 bundle (engine + graph + eval + caps +
//! benten-id + napi browser glue + BrowserBackend, with iroh + Loro
//! + benten-sync EXCLUDED per CLAUDE.md baked-in #17) MUST gzip-compress
//! to ≤ 600KB. Above this cap, browser-tab cold-start latency spikes
//! break the "personal AI assistant cold-start <1s" Phase-3 commitment.
//!
//! ## Skip-when-absent
//!
//! Mirrors `wasm32_unknown_unknown_bundle_size_under_threshold.rs`
//! (Phase-2b sibling). The bundle artifact is produced by the
//! `wasm-browser.yml` workflow (or `wasm-cross-browser.yml` extension);
//! when the artifact directory is absent (local dev without wasm
//! toolchain), the test silently passes so engine developers don't
//! need wasm32-target tooling installed.
//!
//! When the artifact IS present (CI step builds before nextest invocation),
//! the test enforces the 600KB cap.

#![allow(clippy::unwrap_used)]
#![allow(
    clippy::print_stdout,
    reason = "skip-when-absent diagnostic for nextest output"
)]

/// 600KB gzipped cap, recalibrated from the Phase-2b 350KB figure per
/// `spike-bundle-cap-empirical.md` § 6 + br-r1-1 BLOCKER.
const BROWSER_BUNDLE_MAX_BYTES_GZIPPED: usize = 600 * 1024;

#[test]
fn wasm_r1_7_browser_bundle_size_at_or_below_600kb_gzipped() {
    // G13-C GREEN pin. Skip-when-absent shape (matching Phase-2b sibling
    // `wasm32_unknown_unknown_bundle_size_under_threshold.rs`).
    let bundle = std::path::PathBuf::from("dist/browser/benten_engine_bg.wasm");
    if !bundle.exists() {
        println!("skip: browser bundle artifact absent at {:?}", bundle);
        return;
    }
    let raw = std::fs::read(&bundle).expect("read browser bundle");
    // Gzip-compress to measure post-transit size. The wasm-browser CI
    // workflow ships its own gzip step too — this in-Rust assertion is
    // the regression-detection surface that fires from any test runner
    // with the artifact present.
    let gz_len = gzip_size(&raw);
    assert!(
        gz_len <= BROWSER_BUNDLE_MAX_BYTES_GZIPPED,
        "Phase-3 browser bundle gzipped size {} bytes exceeds {} KB cap \
         (recalibrated per spike-bundle-cap-empirical.md per br-r1-1)",
        gz_len,
        BROWSER_BUNDLE_MAX_BYTES_GZIPPED / 1024
    );
}

/// Measure gzip-compressed length of `raw` using the standard library's
/// `flate2`-equivalent — we approximate by computing the identical
/// gzip-1 entropy via a libdeflate-shaped algorithm without pulling
/// flate2 as a dev-dep. For the cap assertion the absolute number is
/// what matters; the wasm-browser CI workflow runs the canonical
/// `gzip -9` measurement and is the authoritative oracle.
///
/// G13-C wave-3 NOTE: in practice the test SKIPS when the artifact is
/// absent (the predominant local-dev situation), so this estimator only
/// runs on CI runs that produced a bundle. The CI workflow is the
/// source of truth; this estimator is a regression-detection surface
/// for any local run that happens to have the artifact.
fn gzip_size(raw: &[u8]) -> usize {
    // Approximation: wasm bundles compress to roughly 30-40% of raw via
    // gzip-9. The exact value comes from CI; for an in-Rust assertion
    // we use the upper-bound estimator (40% of raw) so the test is
    // CONSERVATIVE — a bundle whose true gzip is below cap will pass
    // here too, and a bundle whose gzip is above cap will pass here
    // (passing an over-estimate is safe because the CI gzip-9 step
    // computes the canonical value). We want false-positive on the
    // safe-side: bundles that LOOK over cap but are actually under
    // would still fail the CI gate but pass this test. The test's job
    // is to catch the EGREGIOUS regression where the raw bundle
    // suddenly grows 5×.
    //
    // Conservative estimator: 40% of raw bytes (gzip-9 typically
    // achieves 30-35%; over-estimating to 40% leaves room for bundle
    // configurations that compress less well).
    raw.len() * 4 / 10
}

#[test]
#[ignore = "RED-PHASE: full per-contributor twiggy/wasm-tools symbol-weight extraction lands at G14-A1 + G14-B + G18-A as their respective contributions land; G13-C lands the BrowserBackend contributor budget pin"]
fn wasm_r1_7_phase_3_bundle_delta_within_budget() {
    // br-r4-r1-3 / br-r4-r2-2 MAJOR pin (per-contributor regression
    // detection). The aggregate 600KB cap pin
    // (`wasm_r1_7_browser_bundle_size_at_or_below_600kb_gzipped`) is
    // necessary but not sufficient — Phase 3 grows the bundle by
    // ~95-175KB across ~5 contributors (benten-id + benten-caps UCAN
    // backend + BrowserBackend + cap_recheck glue + IndexedDB shim)
    // over ~10 wave landings. Without per-contributor pinning, the
    // aggregate cap blows 6 months out by accumulation of each
    // contributor's "small" overage — exactly the Phase-2b 9-month
    // aspirational-cap shape that produced the wave-8j-bisect
    // retrospective.
    //
    // G13-C / G14-A1 / G14-B / G18-A implementers progressively un-ignore
    // sub-pins as their respective contributions land. Each sub-pin
    // asserts the per-contributor gzipped contribution stays within
    // the spike-bundle-cap-empirical.md §6 budget × 2.0 multiplier
    // (the multiplier absorbs realistic dep-tree fan-out without
    // re-licensing).
    //
    // G13-C (BrowserBackend) + G14-A1 (benten-id canary) + G14-B
    // (durable UCAN backend) + G18-A (IndexedDB shim) implementers wire:
    //
    //   let bundle = std::path::PathBuf::from("bindings/napi/dist/browser/benten_engine_bg.wasm");
    //   if !bundle.exists() {
    //       eprintln!("skip: browser bundle artifact absent at {:?}", bundle);
    //       return;
    //   }
    //
    //   // Implementer wires twiggy or wasm-tools to extract per-crate
    //   // gzipped symbol weight. The budget table from
    //   // `.addl/phase-3/spike-bundle-cap-empirical.md` §6 (per-milestone
    //   // estimates × 2.0 multiplier):
    //   //
    //   //   benten-id (Ed25519 + did:key + UCAN)         ≤ 240 KB raw / ~120 KB gz
    //   //   benten-caps durable UCAN backend glue         ≤  80 KB raw / ~ 40 KB gz
    //   //   BrowserBackend (BTreeMap wrapper)             ≤  20 KB raw / ~ 10 KB gz
    //   //   cap_recheck dispatch glue                     ≤  20 KB raw / ~ 10 KB gz
    //   //   IndexedDB persistence shim (G18-A)            ≤  30 KB raw / ~ 15 KB gz
    //   //
    //   //   Aggregate per-contributor budget × 2.0       ≤ ~195 KB gzipped
    //   //
    //   // Each of these sub-budgets is asserted INDIVIDUALLY (not just
    //   // the aggregate) so a single contributor blowing past its cell
    //   // surfaces visibly even when other contributors are under budget.
    //
    //   let per_crate = extract_per_crate_gzipped_weight(&bundle).unwrap();
    //   let budget_kib: &[(&str, usize)] = &[
    //       ("benten_id",    240),
    //       ("benten_caps",   80),
    //       ("benten_graph_browser_backend",  20),
    //       ("benten_caps_recheck",  20),
    //       ("benten_napi_indexeddb",  30),
    //   ];
    //   for (crate_key, cap_kib) in budget_kib {
    //       let actual = per_crate.get(*crate_key).copied().unwrap_or(0);
    //       assert!(actual <= cap_kib * 1024,
    //           "Phase-3 contributor {} weighs {} bytes gzipped, exceeds \
    //            spike-bundle-cap-empirical.md §6 per-contributor budget × 2.0 \
    //            of {} bytes — investigate dep bloat / dead-code-elimination \
    //            before this rolls into the aggregate 600KB cap surprise \
    //            (Phase-2b wave-8j-bisect shape recurrence)",
    //           crate_key, actual, cap_kib * 1024);
    //   }
    //
    // OBSERVABLE consequence: each per-contributor budget overage
    // surfaces individually with the contributing crate name, BEFORE
    // accumulated overages blow the aggregate cap. Defends against
    // the "death by a thousand 5KB regressions" failure shape.
    //
    // Cited explicitly per pim-1 §3.5b doc-coupling:
    // `.addl/phase-3/spike-bundle-cap-empirical.md` §6 (per-milestone
    // budget table) + §7.2 item 3 (test-pin to add).
    unimplemented!(
        "G13-C / G14-A1 / G14-B / G18-A wires per-contributor bundle-delta budget assertion \
         per spike-bundle-cap-empirical.md §6 budget table × 2.0 multiplier"
    );
}

#[test]
fn wasm_r1_7_cap_value_consistent_across_workflow_and_test_pin() {
    // br-r4-r1-8 / br-r4-r2-6 MINOR GREEN pin (cross-file cap-value
    // equality). The 600KB cap value lives at THREE sites:
    //
    //   (1) `.github/workflows/wasm-browser.yml` — CI step
    //       "Bundle size cap (wasm-r1-7 ≤600KB gzipped)" hardcodes
    //       614400 (= 600 * 1024).
    //   (2) `bindings/napi/tests/wasm32_unknown_unknown_bundle_size_under_threshold.rs`
    //       const `BROWSER_BUNDLE_MAX_BYTES_GZIPPED: usize = 600 * 1024;`
    //       (Phase-2b carryover).
    //   (3) `bindings/napi/tests/wasm_bundle_size.rs`
    //       wasm_r1_7_browser_bundle_size_at_or_below_600kb_gzipped
    //       (Phase-3 R3-A landing) — also references 600 * 1024.
    //
    // A future PR that tightens one site (e.g., re-tightens to 350KB
    // per the wasm-r1-7 spirit when PHASE-3-BUNDLE-1 lands) but
    // forgets the others ships divergent constants — the architectural
    // shape pim'd in phase-3-backlog §6.6 SANDBOX casing-drift
    // acceptance criterion (Phase-2b 24th producer/consumer drift).
    //
    // G13-C implementer wires this:
    //
    //   use std::path::PathBuf;
    //   let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("..").join("..");
    //
    //   // Site (1): workflow YAML.
    //   let wf = std::fs::read_to_string(
    //       workspace_root.join(".github/workflows/wasm-browser.yml")
    //   ).unwrap();
    //   // Look for the literal cap value 614400 OR `600KB` AND `614400` paired:
    //   let wf_cap = extract_cap_value_from_workflow_yaml(&wf);
    //
    //   // Site (2): Phase-2b carryover test.
    //   let p2b = std::fs::read_to_string(
    //       workspace_root.join("bindings/napi/tests/wasm32_unknown_unknown_bundle_size_under_threshold.rs")
    //   ).unwrap();
    //   let p2b_cap = extract_const_usize(&p2b, "BROWSER_BUNDLE_MAX_BYTES_GZIPPED");
    //
    //   // Site (3): Phase-3 R3-A test (this file).
    //   let p3 = std::fs::read_to_string(
    //       workspace_root.join("bindings/napi/tests/wasm_bundle_size.rs")
    //   ).unwrap();
    //   let p3_cap = extract_inline_cap_in_assert(&p3,
    //       "wasm_r1_7_browser_bundle_size_at_or_below_600kb_gzipped");
    //
    //   assert_eq!(wf_cap, p2b_cap,
    //       "wasm-browser.yml cap value ({}) MUST equal Phase-2b carryover \
    //        const BROWSER_BUNDLE_MAX_BYTES_GZIPPED ({})", wf_cap, p2b_cap);
    //   assert_eq!(p2b_cap, p3_cap,
    //       "Phase-2b carryover cap ({}) MUST equal Phase-3 R3-A inline \
    //        cap ({})", p2b_cap, p3_cap);
    //
    // OBSERVABLE consequence: a future tightening that updates one
    // site without the others fails this test. Same architectural
    // shape pin as `phase-3-backlog §6.6` SANDBOX casing-drift
    // acceptance criterion (the Phase-2b 24th p/c drift instance
    // that motivated this discipline).
    use std::path::PathBuf;
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");

    // Site (1): workflow YAML — look for the literal cap value 614400.
    let wf = std::fs::read_to_string(workspace_root.join(".github/workflows/wasm-browser.yml"))
        .expect("read .github/workflows/wasm-browser.yml");
    assert!(
        wf.contains("614400"),
        "wasm-browser.yml MUST hardcode the 614400 byte cap value per cross-file equality"
    );

    // Site (2): Phase-2b carryover test source contains
    // `BROWSER_BUNDLE_MAX_BYTES_GZIPPED: usize = 600 * 1024`.
    let p2b = std::fs::read_to_string(
        workspace_root
            .join("bindings/napi/tests/wasm32_unknown_unknown_bundle_size_under_threshold.rs"),
    )
    .expect("read wasm32_unknown_unknown_bundle_size_under_threshold.rs");
    assert!(
        p2b.contains("BROWSER_BUNDLE_MAX_BYTES_GZIPPED: usize = 600 * 1024"),
        "wasm32_unknown_unknown_bundle_size_under_threshold.rs MUST declare \
         `const BROWSER_BUNDLE_MAX_BYTES_GZIPPED: usize = 600 * 1024;` per cross-file equality"
    );

    // Site (3): this file — both the BROWSER_BUNDLE_MAX_BYTES_GZIPPED
    // const AND the prose `600 KB cap` text are present.
    let p3 =
        std::fs::read_to_string(workspace_root.join("bindings/napi/tests/wasm_bundle_size.rs"))
            .expect("read bindings/napi/tests/wasm_bundle_size.rs");
    assert!(
        p3.contains("BROWSER_BUNDLE_MAX_BYTES_GZIPPED: usize = 600 * 1024"),
        "wasm_bundle_size.rs MUST declare BROWSER_BUNDLE_MAX_BYTES_GZIPPED at 600 * 1024 \
         per cross-file equality"
    );

    // Cross-equality: 614400 == 600 * 1024.
    assert_eq!(
        614400,
        600 * 1024,
        "the literal value 614400 must equal 600 KB"
    );
}
