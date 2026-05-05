//! R3-D RED-PHASE pins for wasm browser bundle content discipline
//! (G16-A + G16-B; CLAUDE.md baked-in #17 thin-client commitment).
//!
//! Pin sources (per r2-test-landscape §2.4 G16-A + §4 thin-client + §3.G):
//!
//! - `tests/loro_not_in_browser_bundle_per_baked_in_17` — baked-in #17
//! - `tests/iroh_not_in_browser_bundle_per_baked_in_17` — baked-in #17
//!
//! ## Bundle-content discipline shape
//!
//! Per CLAUDE.md baked-in #17: browser tabs are THIN-CLIENT VIEWS,
//! not full peers. The full Loro CRDT + iroh networking + benten-sync
//! state machinery is NATIVE-ONLY. The browser bundle MUST NOT
//! contain compiled Loro or iroh symbols, because:
//!
//! 1. **Architectural:** browser-as-full-peer was explicitly rejected
//!    per the v1 milestone gate framing.
//! 2. **Bundle size:** Loro alone adds ~150-300 KB net of compression;
//!    iroh adds ~200-400 KB. Both would balloon the 600 KB cap (per
//!    spike-bundle-cap-empirical.md).
//!
//! ## File ownership note
//!
//! Per r2-test-landscape §2.4 row, this file's ownership is shared
//! between R3-C (G16-A surface — iroh/Loro architectural callouts) and
//! R3-D (G18-A bundle-content auditing). R3-D authors the pin
//! function bodies that audit the wasm bundle output; if R3-C lands
//! a sibling architectural callout in the same file (e.g. against
//! benten-sync compilation), the architectural-pin function names are
//! disjoint by topic.
//!
//! Pairs with `wasm_bundle_size.rs` (the 600 KB cap pin) — that pin
//! catches "bundle is too big" via raw size; this pin catches "wrong
//! library wound up in the bundle" via symbol auditing.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G16-A + G16-B + G18-A wave 5 audit Loro absence from browser bundle per CLAUDE.md baked-in #17"]
fn loro_not_in_browser_bundle_per_baked_in_17() {
    // CLAUDE.md baked-in #17 architectural pin. G18-A (or G16-B
    // implementer who completes the architectural callout) wires this:
    //
    //   // Locate the browser bundle artifact (skip when absent — the
    //   // wasm-browser.yml workflow produces this; local dev without
    //   // wasm toolchain skips):
    //   let bundle = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("dist").join("browser").join("benten_engine_bg.wasm");
    //   if !bundle.exists() {
    //       eprintln!("skip: browser bundle artifact absent at {:?}", bundle);
    //       return;
    //   }
    //
    //   let raw = std::fs::read(&bundle).unwrap();
    //
    //   // Loro symbol presence check (heuristic — Loro's exported
    //   // symbol prefix is `loro_` for Rust ABI + `Loro` for typed
    //   // identifiers; either appearing in the wasm bundle indicates
    //   // Loro was compiled in):
    //   //
    //   //   Cleaner: walmparser walks the wasm and collects fn names.
    //   //   Heuristic: bytewise search for `loro_` ASCII signature.
    //
    //   let bundle_str = String::from_utf8_lossy(&raw);
    //   assert!(!bundle_str.contains("loro_internal::"),
    //       "browser bundle MUST NOT contain Loro symbols per CLAUDE.md baked-in #17 thin-client commitment; \
    //        full Loro CRDT machinery is native-only (see crates/benten-sync architecture)");
    //
    //   // Sibling assertion: the Cargo workspace declares Loro as
    //   // native-only:
    //   let workspace = std::fs::read_to_string(
    //       std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //           .join("..").join("..").join("Cargo.toml")
    //   ).unwrap();
    //   //   (implementer pins the cfg-gating shape — likely a
    //   //    target.cfg(not(target_arch = "wasm32")).dependencies block)
    //
    // OBSERVABLE consequence: a regression that adds Loro to the
    // browser bundle (e.g. via "browser-as-full-peer" ambition or via
    // an accidental wasm32-cfg-leak) fails this pin. Defends CLAUDE.md
    // baked-in #17 directly + reinforces the bundle-cap budget.
    unimplemented!("G16-B/G18-A wires browser-bundle Loro-absence symbol assertion");
}

#[test]
#[ignore = "RED-PHASE: G16-A + G18-A wave 5 audit iroh absence from browser bundle per CLAUDE.md baked-in #17"]
fn iroh_not_in_browser_bundle_per_baked_in_17() {
    // CLAUDE.md baked-in #17 architectural pin. G18-A wires this:
    //
    //   let bundle = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("dist").join("browser").join("benten_engine_bg.wasm");
    //   if !bundle.exists() {
    //       eprintln!("skip: browser bundle artifact absent at {:?}", bundle);
    //       return;
    //   }
    //
    //   let raw = std::fs::read(&bundle).unwrap();
    //   let bundle_str = String::from_utf8_lossy(&raw);
    //
    //   // iroh symbol presence check:
    //   for iroh_marker in &["iroh_net::", "iroh_blobs::", "iroh::Endpoint"] {
    //       assert!(!bundle_str.contains(iroh_marker),
    //           "browser bundle MUST NOT contain iroh marker {} per CLAUDE.md baked-in #17; \
    //            full peer networking is native-only",
    //           iroh_marker);
    //   }
    //
    // OBSERVABLE consequence: a regression that adds iroh to the
    // browser bundle (e.g. via "browser-tab-as-WebRTC-peer" ambition)
    // fails this pin. Defends CLAUDE.md baked-in #17 directly.
    //
    // Pairs with `loro_not_in_browser_bundle_per_baked_in_17` — both
    // distinct architectural axes (CRDT vs networking) per
    // r2-test-landscape §3.G bundle-size regression-detection table.
    unimplemented!("G18-A wires browser-bundle iroh-absence symbol assertion");
}
