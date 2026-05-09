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
#[ignore = "phase-3-backlog §7.3.D — wasm bundle Loro absence audit per CLAUDE.md baked-in #17 (full peer vs thin compute surface). G16-A + G16-B + G18-A wave-5 ALL shipped; test body pins specific Loro-absence symbol-section audit; un-ignore at §4.4 Bundle-content audit pins landing (CI workflow wasm-bundle-content-audit.yml) per Wave-E rationale-only sweep."]
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
    //   // Loro symbol presence check (r4-r2-napi-2 PICK: wasmparser walk,
    //   // NOT bytewise heuristic — wasm-opt -Oz strip would defeat the
    //   // bytewise check by removing the name-section debug info).
    //   //
    //   // The wasmparser pick walks imports / exports / custom-name-section
    //   // (when present) + collects function names. It survives wasm-opt
    //   // -Oz IF the audit runs on the PRE-STRIP artifact; if the audit
    //   // runs post-strip the walk's symbol coverage is degraded but the
    //   // Cargo.lock defense-in-depth complement (below) still fires.
    //   //
    //   // R5 G18-A implementer wires:
    //   //
    //   //   use wasmparser::Parser;
    //   //   for payload in Parser::new(0).parse_all(&raw) {
    //   //       match payload.unwrap() {
    //   //           wasmparser::Payload::ImportSection(imports) => {
    //   //               for import in imports {
    //   //                   let import = import.unwrap();
    //   //                   for marker in &["loro_", "Loro"] {
    //   //                       assert!(!import.module.contains(marker)
    //   //                              && !import.name.contains(marker),
    //   //                           "browser bundle import '{}::{}' contains \
    //   //                            Loro marker {} per CLAUDE.md baked-in #17",
    //   //                           import.module, import.name, marker);
    //   //                   }
    //   //               }
    //   //           }
    //   //           wasmparser::Payload::ExportSection(exports) => {
    //   //               for export in exports {
    //   //                   let export = export.unwrap();
    //   //                   for marker in &["loro_", "Loro"] {
    //   //                       assert!(!export.name.contains(marker),
    //   //                           "browser bundle export '{}' contains \
    //   //                            Loro marker {} per CLAUDE.md baked-in #17",
    //   //                           export.name, marker);
    //   //                   }
    //   //               }
    //   //           }
    //   //           wasmparser::Payload::CustomSection(reader)
    //   //               if reader.name() == "name" => {
    //   //               // Walk the name-section for fn names; reject any
    //   //               // containing `loro_` / `Loro`. This section is
    //   //               // STRIPPED by wasm-opt -Oz; presence is best-effort.
    //   //           }
    //   //           _ => {}
    //   //       }
    //   //   }
    //   //
    //   //   // Pre-strip artifact preferred; if running post-strip, the
    //   //   // wasmparser walk degrades + Cargo.lock defense-in-depth fires.
    //
    //   // r4-r2-napi-2 DEFENSE-IN-DEPTH: Cargo.lock walk — survives
    //   // wasm-opt -Oz strip because it asserts at compile-time which
    //   // crates resolve into the wasm32 build target. The browser
    //   // crate's resolved deps must NOT include `loro` or `loro-internal`:
    //   let cargo_lock = std::fs::read_to_string(
    //       std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //           .join("..").join("..").join("Cargo.lock")
    //   ).unwrap();
    //   //   Parse the Cargo.lock; resolve which packages are pulled in
    //   //   by the wasm32 dep-graph rooted at the browser cdylib crate.
    //   //   Assert NO transitive resolution to loro / loro-internal /
    //   //   iroh / iroh-net / iroh-blobs from the wasm32 root.
    //   //   (Implementation: cargo metadata --filter-platform wasm32-unknown-unknown
    //   //    + walk resolve.nodes; or parse Cargo.lock by hand under
    //   //    --locked CI invariant.)
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
#[ignore = "phase-3-backlog §7.3.D — wasm bundle iroh absence audit per CLAUDE.md baked-in #17. G16-A + G18-A wave-5 ALL shipped; test body pins specific iroh-absence symbol-section audit; un-ignore at §4.4 Bundle-content audit pins landing per Wave-E rationale-only sweep."]
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
