//! Phase-3 G17-B GREEN-PHASE pins for D26 .wasm-bytes shipping —
//! cargo-alias + workspace-member dimensions (phase-3-backlog §6.2 +
//! r1-wsa-5 + r4-r1-wsa-9).
//!
//! ## File-split rationale (LOAD-BEARING for reviewer orientation)
//!
//! The G17-B test-pin set covers FOUR D26 dimensions:
//!   1. **Per-fixture .wasm presence + loader prefer/fallback shape**
//!      — `tests/d26_wasm_present.rs` (un-ignored by G17-B).
//!   2. **Workspace `wat` exact-version pin (single-tool contract)**
//!      — `tests/wasm_tools_version.rs` (un-ignored by G17-B; test
//!      name preserved from the R3-D pin per the test-pin catalog).
//!   3. **Per-fixture BLAKE3 drift detector** —
//!      `tests/fixture_wasm_hashes_stable.rs` (already green at
//!      Phase-2b G7-B; G17-B updated PINNED_FIXTURES for the three
//!      fixtures whose canonical bytes shifted from `wabt` → `wat`).
//!   4. **Cargo-alias + workspace-member orchestration shape** —
//!      THIS file (the brief-named pin
//!      `sandbox_d26_wasm_bytes_shipping`).
//!
//! Splitting dimensions (2) and (4) keeps the version-pin assertion
//! grouped with the workspace-Cargo.toml read in
//! `wasm_tools_version.rs` while letting THIS file own the rebake-
//! protocol orchestration (alias + member list) — the two surfaces
//! evolve on different cadences (a `wat` minor bump touches (2) but
//! not (4); a tool-relocation refactor touches (4) but not (2)).
//!
//! ## Pins owned by this file
//!
//! - `d26_bench_wat_rebake_alias_present` — the cargo alias that ships
//!   the rebake protocol per r1-wsa-5 + r4-r1-wsa-9 single-tool.
//! - `d26_bench_wat_rebake_crate_member_listed` — workspace declares
//!   the regenerator crate as a member.

#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

#[test]
fn d26_bench_wat_rebake_alias_present() {
    // The `cargo bench-wat-rebake` alias is the canonical entry point
    // for regenerating committed `.wasm` fixture bytes. It MUST be
    // declared in `.cargo/config.toml` so devs can invoke it without
    // an absolute path. Defends pim-2 §3.6b: the rebake protocol cited
    // in r1-wsa-5 + r4-r1-wsa-9 must drive the production entry point.
    let cargo_config =
        std::fs::read_to_string(workspace_root().join(".cargo").join("config.toml")).unwrap();

    assert!(
        cargo_config.contains("bench-wat-rebake"),
        ".cargo/config.toml MUST declare a `bench-wat-rebake` alias per \
         r4-r1-wsa-9 single-tool rebake protocol (would otherwise force devs \
         to invoke `cargo run -p bench-wat-rebake -- ...` directly + drift \
         from the documented protocol shape)"
    );

    // Alias points at the `bench-wat-rebake` binary in
    // `tools/bench-wat-rebake/`:
    assert!(
        cargo_config.contains("-p bench-wat-rebake"),
        "the `bench-wat-rebake` alias MUST invoke the dedicated tool crate \
         (`-p bench-wat-rebake`) — pim-1 §3.5b doc-prose-vs-shipped-shape coupling"
    );
}

#[test]
fn d26_bench_wat_rebake_crate_member_listed() {
    // The workspace MUST declare `tools/bench-wat-rebake` as a member
    // so `cargo run -p bench-wat-rebake` resolves. A member-list omission
    // would make the alias point at a non-existent crate (silent CI
    // failure surface).
    let cargo_toml = std::fs::read_to_string(workspace_root().join("Cargo.toml")).unwrap();

    assert!(
        cargo_toml.contains("tools/bench-wat-rebake"),
        "workspace Cargo.toml [workspace.members] MUST list `tools/bench-wat-rebake` \
         per r4-r1-wsa-9 (the alias in `.cargo/config.toml` resolves through this \
         member entry)"
    );
}
