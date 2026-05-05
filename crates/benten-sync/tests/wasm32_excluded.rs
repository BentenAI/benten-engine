//! R3-C RED-PHASE architectural pin for `benten-sync` native-only
//! commitment per CLAUDE.md baked-in #17 (G16-A wave-6 canary).
//!
//! ## Pin source
//!
//! - r2-test-landscape §2.4 G16-A row + §3.E thin-client cluster +
//!   §4 baked-in #17 architectural pins.
//! - plan §3 G16-A row line "`benten-sync` excluded from wasm32
//!   targets per CLAUDE.md baked-in #17".
//! - CLAUDE.md baked-in #17 (full-peer / thin-client commitment;
//!   browser tabs participate via D-PHASE-3-N protocol, not as full
//!   peers).
//! - plan §3 G16-A row line "iroh transport NEVER compiles for
//!   wasm32: Cargo.toml `[target.'cfg(not(target_arch = \"wasm32\"))']`
//!   cfg-gate".
//!
//! ## What this pins
//!
//! `benten-sync` MUST NOT compile for `wasm32-unknown-unknown` target.
//! Browser tabs participate in Atriums as authenticated thin-client
//! views (D-PHASE-3-N: snapshot CID + authenticated POST + SSE/WS
//! subscription against a full peer) — NOT as full peers running
//! iroh + Loro + MST natively.
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale `"RED-PHASE: G16-A wave-6 wires Cargo.toml cfg-gate; wasm32 build asserted to fail post-G16-A"`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G16-A wave-6 — CLAUDE.md baked-in #17 — benten-sync wasm32-excluded"]
fn benten_sync_does_not_compile_for_wasm32_unknown_unknown_per_thin_client_commitment() {
    // CLAUDE.md baked-in #17 architectural pin. G16-A implementer
    // wires this against a build-time assertion + a CI cell that
    // attempts `cargo check -p benten-sync --target wasm32-unknown-unknown`
    // and asserts the build FAILS (with a clear error).
    //
    // Concrete shape (preferred — at the source level):
    //   #[cfg(target_arch = "wasm32")]
    //   compile_error!(
    //       "benten-sync is native-only per CLAUDE.md baked-in #17. \
    //        Browser tabs participate via D-PHASE-3-N thin-client protocol, \
    //        not as full Atrium peers. Use `benten-engine`'s thin-client \
    //        surfaces from wasm32 builds."
    //   );
    //
    // Companion CI cell (`.github/workflows/cross-target-build.yml`
    // or similar) attempts a wasm32 build of benten-sync and
    // asserts FAILURE — i.e. the absence of the cfg-gate would be
    // a regression. The test in this file is the in-tree assertion
    // shape that documents the expected behavior.
    //
    // The Rust-side test asserts the cfg-gate is in place by
    // checking Cargo.toml's target-specific dependency tables
    // (sync-only deps must NOT appear under
    // `[target.'cfg(target_arch = "wasm32")'.dependencies]`):
    //   let manifest = std::fs::read_to_string(
    //       std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //           .join("Cargo.toml")
    //   ).unwrap();
    //   // benten-sync's iroh + loro + mst deps must live behind a
    //   // not-wasm32 cfg-gate. The simplest assertion: the manifest
    //   // includes a `cfg(not(target_arch = "wasm32"))` table OR the
    //   // top-level dependencies table excludes iroh/loro entirely
    //   // (with a build-time compile_error! for wasm32 builds in lib.rs).
    //   assert!(
    //       manifest.contains("cfg(not(target_arch = \"wasm32\"))")
    //           || std::fs::read_to_string("src/lib.rs").unwrap().contains("compile_error"),
    //       "benten-sync must cfg-gate iroh/loro deps OR emit compile_error! on wasm32 per CLAUDE.md baked-in #17"
    //   );
    //
    // OBSERVABLE consequence: any future refactor that accidentally
    // ships iroh/loro into a wasm32 build of benten-sync fails this
    // assertion + the companion CI cell.
    unimplemented!(
        "G16-A wires Cargo.toml cfg-gate + lib.rs compile_error! for wasm32-unknown-unknown"
    );
}
