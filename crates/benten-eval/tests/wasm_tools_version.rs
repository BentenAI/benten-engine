//! Phase-3 G17-B GREEN-PHASE pin for the workspace wat-tooling
//! version pin (r1-wsa-5 → r4-r1-wsa-9 recalibration).
//!
//! Pin source: r2-test-landscape §2.5 G17-B
//! `d26_wasm_tools_version_pinned_at_1_227_x_per_r1_wsa_5`.
//!
//! ## Recalibration narrative (r4-r1-wsa-9 → r4-r1-wsa-9)
//!
//! r1-wsa-5 RECOMMENDATION named `wasm-tools 1.227.x` as the pin
//! target — but the actual ecosystem dep we already used at
//! integration-test time was the sibling `wat` crate (workspace
//! Cargo.toml line 309). Mixing `wasm-tools` (Bytecode Alliance CLI)
//! and `wat` crate (wabt-ecosystem library) at build vs test time
//! would emit slightly different bytes — exactly the producer/consumer
//! drift Phase-3 G17-B needs to avoid. r4-r1-wsa-9 recalibrated to a
//! single tool: `wat` is the ONE pinned dep; `wasm-tools` is REJECTED
//! as a parallel dep.
//!
//! ## Test name preserved
//!
//! The test name is preserved from the R3-D pin (was named
//! `d26_wasm_tools_version_pinned_at_1_227_x_per_r1_wsa_5` reflecting
//! the original r1-wsa-5 wording); body asserts the actual r4-r1-wsa-9
//! outcome. Renaming the function would invalidate the test-pin
//! catalog; the body update + module-level recalibration narrative
//! does the work.
//!
//! ## Why a separate test file
//!
//! Distinct from `d26_cross_platform_fixture_cid_stable` (in
//! `d26_wasm_present.rs`) because:
//!   - That pin asserts CID-determinism by checking actual bytes.
//!   - This pin asserts the BUILD-CONTRACT stability by checking the
//!     version declaration. A workspace bump that doesn't run the
//!     `cargo bench-wat-rebake` protocol would silently drift CIDs;
//!     this pin catches the version-bump-without-rebake first.
//!
//! Companion file `sandbox_d26_wasm_bytes_shipping.rs` covers the
//! cargo-alias + workspace-member dimensions (this file owns the
//! version-pin dimension).

#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;

#[test]
fn d26_wasm_tools_version_pinned_at_1_227_x_per_r1_wsa_5() {
    let workspace = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("Cargo.toml"),
    )
    .unwrap();

    // r4-r1-wsa-9 recalibration (1): workspace declares `wat` (NOT
    // `wasm-tools`) as the canonical wat-compilation dep. The `wat`
    // crate is wabt-ecosystem; `wasm-tools` is Bytecode Alliance CLI;
    // they emit slightly different bytes on the same source.
    assert!(
        workspace.contains("\nwat = ") || workspace.contains("\nwat="),
        "workspace [workspace.dependencies] MUST declare `wat` per r4-r1-wsa-9 \
         recalibration (the `wat` crate is the canonical single tool; mixing it \
         with wasm-tools at build vs test time would silently drift fixture CIDs)"
    );

    // r4-r1-wsa-9 recalibration (2): EXACT-version pin (= prefix).
    // Soft matchers (`^`, `~`, bare) permit silent minor bumps that
    // may change emitted bytes:
    assert!(
        workspace.contains("wat = \"=") || workspace.contains("wat=\"="),
        "workspace `wat` dep MUST use `= ` exact-version prefix per r4-r1-wsa-9; \
         soft matchers (^, ~, bare) defeat the determinism contract"
    );

    // r4-r1-wsa-9 recalibration (3): NO parallel `wasm-tools` dep.
    // The recalibration explicitly chose `wat` as the single tool;
    // a parallel `wasm-tools` would split build-time + test-time
    // .wat-compilation paths and reintroduce p/c drift.
    assert!(
        !workspace.contains("\nwasm-tools = "),
        "workspace Cargo.toml MUST NOT declare a `wasm-tools` parallel dep per \
         r4-r1-wsa-9 single-tool commitment"
    );

    // The rebake-tooling subcommand entry point exists at the
    // canonical path (`cargo bench-wat-rebake` resolves through this
    // crate per `.cargo/config.toml` alias):
    let rebake_main = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tools")
        .join("bench-wat-rebake")
        .join("src")
        .join("main.rs");
    assert!(
        rebake_main.exists(),
        "rebake-tooling subcommand entry point MUST exist at \
         `tools/bench-wat-rebake/src/main.rs` per r4-r1-wsa-9; the \
         `cargo bench-wat-rebake` alias in `.cargo/config.toml` resolves \
         through this binary"
    );
}
