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
//! - Phase-3 R6 fix-pass Wave B (ds-r6-3 closure): at-build-time CI
//!   cell named `benten-sync-refuses-wasm32` in
//!   `.github/workflows/wasm-checks.yml` asserts the `compile_error!`
//!   macro fires on a real
//!   `cargo check --target wasm32-unknown-unknown -p benten-sync`.
//!
//! ## What this pins
//!
//! `benten-sync` MUST NOT compile for `wasm32-unknown-unknown` target.
//! Browser tabs participate in Atriums as authenticated thin-client
//! views (D-PHASE-3-30: snapshot CID + authenticated POST + SSE/WS
//! subscription against a full peer) — NOT as full peers running iroh
//! + Loro + MST natively.
//!
//! Three layers defend the native-only commitment (defense-in-depth):
//!
//! 1. **lib.rs `compile_error!`** — fires immediately for any wasm32
//!    build attempt with a clear error pointing at CLAUDE.md baked-in
//!    #17 + the thin-client surface alternative. (Source-side gate.)
//! 2. **Cargo.toml cfg-gated dependency tables** — iroh + tokio live
//!    behind `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`.
//!    NOTE (corrected 2026-07-29, measured): this is NOT a defense. It
//!    removes iroh/tokio/loro on wasm32, which is exactly what leaves a
//!    crate that COMPILES. With the lib.rs gate deleted and getrandom's
//!    js/wasm_js features on, `benten-sync` builds for wasm32 at exit 0.
//!    Rung 1 is the only thing that actually refuses. (Manifest hygiene.)
//! 3. **CI at-build-time assertion** — `.github/workflows/wasm-checks.yml`
//!    job `benten-sync-refuses-wasm32`, in two arms. Arm (a) runs `cargo
//!    check --target wasm32-unknown-unknown -p benten-sync` and asserts it
//!    fails; that is structural only, since the failure comes from
//!    `getrandom` before rustc reaches our source, and NO mutation of our
//!    gate changes it. Arm (b) rebuilds the same check in a throwaway probe
//!    with getrandom's js/wasm_js features enabled — the thing a
//!    thin-client bundler would do — and asserts the build still fails
//!    carrying `baked-in #17` attributed to `crates/benten-sync/src/lib.rs`.
//!    Arm (b) is the falsifiable one: delete the `compile_error!` and the
//!    probe compiles, which reds the job. (At-build-time gate.)
//!
//! This test asserts ALL THREE defenses are present at the source-of-
//! truth manifests + the CI workflow.

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

#[test]
fn benten_sync_wasm32_refusal_pinned_in_ci_workflow_per_ds_r6_3_closure() {
    // Defense-in-depth rung 3 (CI at-build-time assertion). Closes
    // ds-r6-3 MAJOR per `.addl/phase-3/r6-r1-distributed-systems.json`.
    //
    // Workflow-pin shape: assert `.github/workflows/wasm-checks.yml`
    // declares the `benten-sync-refuses-wasm32` job that runs
    // `cargo check --target wasm32-unknown-unknown -p benten-sync` and
    // asserts the build fails with the `compile_error!` macro firing.
    //
    // Without this CI cell, defense-in-depth has only the two
    // declarative rungs (lib.rs + Cargo.toml grep). A regression that
    // (a) removed `compile_error!` while keeping the Cargo.toml
    // cfg-gate, or (b) removed the Cargo.toml cfg-gate while keeping
    // `compile_error!`, would silently leave the OTHER rung in place
    // and pass the source-of-truth grep audits — but at-build-time
    // could surface either as a SUCCESS (wasm32 build that should NOT
    // succeed) or as a different unrelated failure.
    let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    let workflow =
        std::fs::read_to_string(workspace_root.join(".github/workflows/wasm-checks.yml"))
            .expect("read .github/workflows/wasm-checks.yml");

    assert!(
        workflow.contains("benten-sync-refuses-wasm32") || workflow.contains("benten-sync refuses"),
        ".github/workflows/wasm-checks.yml MUST declare a \
         `benten-sync-refuses-wasm32` job per ds-r6-3 closure (CLAUDE.md \
         baked-in #17 defense-in-depth rung 3)"
    );

    assert!(
        workflow.contains("cargo check --target wasm32-unknown-unknown -p benten-sync"),
        ".github/workflows/wasm-checks.yml `benten-sync-refuses-wasm32` job \
         MUST invoke `cargo check --target wasm32-unknown-unknown -p \
         benten-sync` to drive the at-build-time assertion"
    );

    // The CI cell must invert the exit code (build MUST fail). Look
    // for the canonical pattern that asserts non-zero exit + the
    // failure-narrative-classifier.
    assert!(
        workflow.contains("compile_error!") || workflow.contains("baked-in #17"),
        ".github/workflows/wasm-checks.yml `benten-sync-refuses-wasm32` job \
         MUST verify the build failure stderr cites `compile_error!` or \
         `baked-in #17` so the failure-classifier guards against unrelated \
         dep-graph breaks"
    );
}
