//! R3-A RED-PHASE pin: wasm32 bundle does not link redb (G13-C wave-3;
//! plan §4 seed; br-r1-1 BLOCKER companion).
//!
//! Pin source: r2-test-landscape §2.1 G13-C row
//! `wasm32_unknown_unknown_bundle_does_not_link_redb`; plan §4 seed.
//!
//! ## What this pins
//!
//! Companion to `crates/benten-graph/tests/browser_backend.rs`'s
//! `browser_backend_no_redb_dep_on_wasm32_unknown_unknown`. That test
//! pins the GRAPH-SIDE absence; this pins the ENGINE-SIDE absence.
//!
//! `cargo check --target wasm32-unknown-unknown -p benten-engine
//! --features browser-backend --no-default-features` MUST succeed
//! WITHOUT compiling redb anywhere in the dep tree.
//!
//! Verification at CI happens via `wasm-checks.yml` extension; this
//! Rust-side pin is the source-cite regression guard against the
//! engine accidentally pulling `RedbBackend` references into a code
//! path that's reachable on wasm32.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "phase-3-backlog §7.3.D — wasm32 engine bundle excludes redb. G13-C wave-3 shipped BrowserBackend thin-client cache impl (PR #106); test body pins wasm32-bundle-content audit (no redb symbol section); un-ignore at §4.4 Bundle-content audit pins landing (R4b architecture / wasm-bundle lens carry; G16-B-D sub-item D NAMED-NOW destination; CI workflow wasm-bundle-content-audit.yml) per Wave-E rationale-only sweep."]
fn wasm32_unknown_unknown_bundle_does_not_link_redb() {
    // G13-C implementer wires this:
    //   // Engine's lib.rs and engine.rs MUST gate any `RedbBackend`
    //   // reference behind `#[cfg(not(target_arch = "wasm32"))]` OR
    //   // make it specialization-only inside the default `Engine`
    //   // alias (which itself is gated to native target).
    //
    //   let engine_src = std::fs::read_to_string("crates/benten-engine/src/engine.rs").unwrap();
    //   let mut wasm_unsafe_redb_refs: Vec<usize> = Vec::new();
    //   // Walk the file, tracking cfg-context, flagging redb refs
    //   // that are NOT inside a not-wasm cfg.
    //   // ... (implementer fills the source-walker)
    //   assert!(wasm_unsafe_redb_refs.is_empty(),
    //       "engine.rs sites referencing RedbBackend without cfg(not(wasm32)) gating: {:?}",
    //       wasm_unsafe_redb_refs);
    //
    // OBSERVABLE consequence: a future PR introducing
    // `let backend = RedbBackend::create(...)` in a function reachable
    // from `EngineGeneric<B>::open()` (without the wasm-exclusion gate)
    // fails this test. The wasm-checks.yml CI run is the authoritative
    // wasm32-target compile verifier; this pin is the source-cite
    // regression guard so the failure surfaces in the same PR as the
    // mistake, not 30 minutes later in CI.
    unimplemented!("G13-C wires source-walker assertion that RedbBackend refs are wasm-excluded");
}
