//! R3-C RED-PHASE pin: Loro + iroh excluded from browser bundle
//! (G16-A + G16-B; per CLAUDE.md baked-in #17 thin-client commitment).
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.4 G16-A rows
//!   `loro_not_in_browser_bundle_per_baked_in_17` +
//!   `iroh_not_in_browser_bundle_per_baked_in_17`.
//! - r2-test-landscape §3.E thin-client cluster + §3.G bundle-size
//!   regression-detection.
//! - CLAUDE.md baked-in #17 (full-peer / thin-client commitment).
//! - plan §3 G16-B row line "Loro DOES NOT ship to browser bundle;
//!   closes br-r1-5 MAJOR via thin-client commitment".
//! - `br-r1-5` MAJOR (Loro in browser bundle bloats wasm size beyond
//!   600KB cap; resolved by thin-client commitment — Loro stays in
//!   benten-sync which is native-only).
//!
//! ## What this pins
//!
//! After G16-A + G16-B land, the wasm32-unknown-unknown browser
//! bundle (built via `cargo build --target wasm32-unknown-unknown
//! -p benten-napi`) MUST NOT include Loro or iroh symbols. Both
//! libraries live in `benten-sync` which is native-only per
//! CLAUDE.md baked-in #17; the wasm32 build path explicitly
//! excludes that crate from its dependency tree.
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale `"RED-PHASE: G16-A + G16-B + benten-sync wasm32-exclusion at landing"`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G16-A + G16-B + baked-in #17 — Loro absent from browser bundle"]
fn loro_not_in_browser_bundle_per_baked_in_17() {
    // baked-in #17 + br-r1-5 MAJOR closure pin. G16-B implementer
    // wires this against the wasm32-unknown-unknown build artifact:
    //
    //   let wasm_bytes = std::fs::read(WASM_BUNDLE_PATH).unwrap();
    //   let symbols = parse_wasm_symbols(&wasm_bytes);
    //   for sym in &symbols {
    //       assert!(
    //           !sym.contains("loro"),
    //           "wasm32-unknown-unknown browser bundle contains Loro symbol {sym}; \
    //            Loro must stay in benten-sync (native-only) per CLAUDE.md baked-in #17"
    //       );
    //   }
    //
    // OBSERVABLE consequence: a future refactor that accidentally
    // pulls Loro into the wasm32 dep tree fails this test. Defends
    // against the bundle-size regression that br-r1-5 named MAJOR.
    unimplemented!("G16-B wires wasm32 bundle symbol scan to confirm Loro absence");
}

#[test]
#[ignore = "RED-PHASE: G16-A + baked-in #17 — iroh absent from browser bundle"]
fn iroh_not_in_browser_bundle_per_baked_in_17() {
    // baked-in #17 + br-r1-5 sibling closure pin. G16-A implementer
    // wires this against the wasm32-unknown-unknown build artifact:
    //
    //   let wasm_bytes = std::fs::read(WASM_BUNDLE_PATH).unwrap();
    //   let symbols = parse_wasm_symbols(&wasm_bytes);
    //   for sym in &symbols {
    //       assert!(
    //           !sym.contains("iroh"),
    //           "wasm32-unknown-unknown browser bundle contains iroh symbol {sym}; \
    //            iroh must stay in benten-sync (native-only) per CLAUDE.md baked-in #17"
    //       );
    //   }
    //
    // OBSERVABLE consequence: defends against any future change that
    // pulls iroh symbols into the wasm32 build tree (which would
    // both blow the bundle size cap + break the thin-client
    // commitment that browsers use HTTP/SSE/WS to reach a full peer
    // rather than running iroh directly).
    unimplemented!("G16-A wires wasm32 bundle symbol scan to confirm iroh absence");
}
