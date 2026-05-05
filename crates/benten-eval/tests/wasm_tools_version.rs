//! R3-D RED-PHASE pin for wasm-tools version pin (G17-B wave-5b;
//! r1-wsa-5 MAJOR).
//!
//! Pin source: r2-test-landscape §2.5 G17-B
//! `d26_wasm_tools_version_pinned_at_1_227_x_per_r1_wsa_5`.
//!
//! ## Version-pin shape (r1-wsa-5)
//!
//! Reproducibility of D26 .wasm bytes (per `d26_wasm_present.rs`)
//! depends on a stable wasm-tools version. r1-wsa-5 pinned the
//! workspace dev-dep at 1.227.x; this test asserts the workspace
//! Cargo.toml carries the pin.
//!
//! Distinct from `d26_cross_platform_fixture_cid_stable` because:
//!
//! - That pin asserts CID-determinism by checking actual bytes.
//! - This pin asserts the BUILD-CONTRACT stability by checking the
//!   version declaration. A workspace bump that doesn't run the
//!   `cargo bench-wat-rebake` protocol would silently drift CIDs;
//!   this pin catches the version-bump-without-rebake first.

#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

#[test]
#[ignore = "RED-PHASE: G17-B wave-5b locks workspace wasm-tools dev-dep at 1.227.x per r1-wsa-5"]
fn d26_wasm_tools_version_pinned_at_1_227_x_per_r1_wsa_5() {
    // r1-wsa-5 pin. G17-B implementer wires this:
    //
    //   // Workspace Cargo.toml carries the pin in [workspace.dependencies]
    //   // or [workspace.dev-dependencies]:
    //   let workspace = std::fs::read_to_string(
    //       std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //           .join("..").join("..").join("Cargo.toml")
    //   ).unwrap();
    //
    //   // Either form is acceptable:
    //   //   wasm-tools = "1.227"
    //   //   wasm-tools = { version = "1.227", ... }
    //   //   wasm-tools = "=1.227.X" (exact pin)
    //
    //   // Heuristic regex-style assertion (implementer pins exact form):
    //   let has_pin = workspace.contains("wasm-tools = \"1.227")
    //       || workspace.contains("wasm-tools = { version = \"1.227")
    //       || workspace.contains("wasm-tools=\"=1.227");
    //   assert!(has_pin,
    //       "workspace Cargo.toml must pin wasm-tools at 1.227.x per r1-wsa-5; \
    //        a version drift (without running `cargo bench-wat-rebake`) silently \
    //        breaks fixture-CID stability per phase-3-backlog §6.2");
    //
    //   // Additionally, the rebake-tooling subcommand exists:
    //   let bench_tool = std::fs::read_to_string("tools/bench-wat-rebake/src/main.rs")
    //       .or_else(|_| std::fs::read_to_string("tools/wat-rebake/src/main.rs"));
    //   //   (implementer pins the canonical path)
    //
    // OBSERVABLE consequence: a workspace bump like
    //   wasm-tools = "1.230"
    // fails this pin AND requires the rebaker tooling subcommand to
    // be run + new fixture-CIDs to be committed in the same PR.
    // Defends r1-wsa-5 determinism shape.
    unimplemented!("G17-B wires workspace Cargo.toml wasm-tools version-pin assertion");
}
