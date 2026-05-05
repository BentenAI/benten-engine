//! R3-A RED-PHASE pin: wasm browser bundle size ≤ 600KB gzipped
//! (G13-C wave-3; br-r1-1 BLOCKER + spike-bundle-cap-empirical.md).
//!
//! Pin source: r2-test-landscape §2.1 G13-C row
//! `wasm_r1_7_browser_bundle_size_at_or_below_600kb_gzipped`; br-r1-1
//! BLOCKER (recalibrated from 350KB per `spike-bundle-cap-empirical.md`
//! 2026-05-04 — Loro is OUT per CLAUDE.md baked-in #17).
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

#[test]
#[ignore = "RED-PHASE: G13-C wave-3 produces the Phase-3 browser bundle artifact"]
fn wasm_r1_7_browser_bundle_size_at_or_below_600kb_gzipped() {
    // G13-C implementer wires this against the post-build artifact
    // path. Skip-when-absent shape (matching Phase-2b sibling):
    //
    //   let bundle = std::path::PathBuf::from("bindings/napi/dist/browser/benten_engine_bg.wasm");
    //   if !bundle.exists() {
    //       eprintln!("skip: browser bundle artifact absent at {:?}", bundle);
    //       return;
    //   }
    //   let raw = std::fs::read(&bundle).unwrap();
    //   let gz = gzip_compress(&raw); // implementer wires zstd or flate2
    //   assert!(gz.len() <= 600 * 1024,
    //       "Phase-3 browser bundle gzipped size {} bytes exceeds 600 KB cap \
    //        (recalibrated per spike-bundle-cap-empirical.md per br-r1-1)",
    //       gz.len());
    //
    // The 600KB cap is RECALIBRATED from the Phase-2b 350KB figure
    // because the Phase-3 bundle adds:
    //   - benten-id (Ed25519 + did:key + UCAN; ~80-120KB net of compression)
    //   - benten-caps durable UCAN backend dispatch glue (~20-40KB)
    //   - BrowserBackend (~5-10KB; thin BTreeMap wrapper)
    //   - capability cap_recheck dispatch dispatch glue (~5-10KB)
    // Loro + iroh + benten-sync are EXCLUDED per CLAUDE.md baked-in #17.
    //
    // OBSERVABLE consequence: bundle bloat above 600KB fails CI per-PR.
    unimplemented!("G13-C wires browser-bundle gzip-size assertion against 600KB cap");
}
