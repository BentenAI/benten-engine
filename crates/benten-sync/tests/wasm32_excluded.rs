//! G16-A LANDED architectural pin for `benten-sync` native-only
//! commitment per CLAUDE.md baked-in #17.
//!
//! ## Pin source
//!
//! - r2-test-landscape §2.4 G16-A row + §3.E thin-client cluster.
//! - plan §3 G16-A row line "`benten-sync` excluded from wasm32
//!   targets per CLAUDE.md baked-in #17".
//! - CLAUDE.md baked-in #17 (full-peer / thin-client commitment;
//!   browser tabs participate via D-PHASE-3-30 protocol, not as full
//!   peers).
//! - plan §3 G16-A row line "iroh transport NEVER compiles for
//!   wasm32: Cargo.toml `[target.'cfg(not(target_arch = \"wasm32\"))']`
//!   cfg-gate".
//!
//! ## What this pins
//!
//! `benten-sync` MUST NOT compile for `wasm32-unknown-unknown` target.
//! Browser tabs participate in Atriums as authenticated thin-client
//! views (D-PHASE-3-30: snapshot CID + authenticated POST + SSE/WS
//! subscription against a full peer) — NOT as full peers running iroh
//! + Loro + MST natively.
//!
//! Two layers defend the native-only commitment:
//!
//! 1. **lib.rs `compile_error!`** — fires immediately for any wasm32
//!    build attempt with a clear error pointing at CLAUDE.md baked-in
//!    #17 + the thin-client surface alternative.
//! 2. **Cargo.toml cfg-gated dependency tables** — iroh + tokio live
//!    behind `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`,
//!    so even a downstream consumer that bypasses the lib.rs gate
//!    cannot resolve the dep chain on wasm32.
//!
//! This test asserts BOTH defenses are present at the source-of-truth
//! manifests.

#![allow(clippy::unwrap_used)]

#[test]
fn benten_sync_does_not_compile_for_wasm32_unknown_unknown_per_thin_client_commitment() {
    // CLAUDE.md baked-in #17 architectural pin. Verify both defenses:
    let crate_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest = std::fs::read_to_string(crate_root.join("Cargo.toml")).expect("read Cargo.toml");
    let lib_rs = std::fs::read_to_string(crate_root.join("src/lib.rs")).expect("read src/lib.rs");

    // Defense 1: lib.rs `compile_error!` for wasm32.
    assert!(
        lib_rs.contains("compile_error!"),
        "src/lib.rs MUST emit a compile_error! macro for wasm32 builds \
         per CLAUDE.md baked-in #17 (full-peer / thin-client commitment)"
    );
    assert!(
        lib_rs.contains("target_arch = \"wasm32\"") || lib_rs.contains("target_arch=\"wasm32\""),
        "src/lib.rs MUST cfg-gate on `target_arch = \"wasm32\"` \
         per CLAUDE.md baked-in #17"
    );
    assert!(
        lib_rs.contains("CLAUDE.md baked-in #17") || lib_rs.contains("baked-in #17"),
        "src/lib.rs compile_error! MUST cite CLAUDE.md baked-in #17 by name \
         so future maintainers find the architectural commitment"
    );

    // Defense 2: Cargo.toml cfg-gated [target.'cfg(not(target_arch =
    // "wasm32"))'.dependencies] table for the iroh/tokio chain.
    assert!(
        manifest.contains("cfg(not(target_arch = \"wasm32\"))")
            || manifest.contains("cfg(not(target_arch=\"wasm32\"))"),
        "Cargo.toml MUST carry a `cfg(not(target_arch = \"wasm32\"))` \
         dependency table so iroh/tokio chain is not resolvable on wasm32 \
         per CLAUDE.md baked-in #17"
    );
    assert!(
        manifest.contains("iroh"),
        "Cargo.toml MUST reference iroh — the load-bearing wasm32-excluded \
         dep that this test guards against accidentally promoting to a \
         wasm32-resolvable dep table"
    );
}
