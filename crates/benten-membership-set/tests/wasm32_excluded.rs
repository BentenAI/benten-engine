//! Defense-in-depth pin for `benten-membership-set` native-only commitment
//! per CLAUDE.md baked-in #17 (mirrors the `benten-sync` precedent at
//! `crates/benten-sync/tests/wasm32_excluded.rs`).
//!
//! ## What this pins
//!
//! `benten-membership-set` MUST NOT compile for `wasm32-unknown-unknown`.
//! Its B-1 dep set includes the native-only `benten-sync` (iroh transport +
//! Loro CRDT + MST), so the keying primitive is itself a full-peer native
//! mechanism. Browser tabs / thin-client surfaces do NOT carry the
//! MembershipSet keying glue — they receive the materialized membership-set
//! snapshot via the D-PHASE-3-30 thin-client protocol, NOT the in-bundle crate.
//!
//! ## Defense-in-depth (mirrors benten-sync's three rungs)
//!
//! 1. **lib.rs `compile_error!`** — present at `src/lib.rs` L109-115. NOTE
//!    (measured 2026-07-29): it never fires from a whole-crate wasm32 build,
//!    because the un-cfg-gated `benten-sync` dependency must compile first
//!    and its own gate fires there. Kept as the defense that survives
//!    benten-sync one day dropping its gate. (Source-side gate.)
//! 2. **Cargo.toml native-only dep chain** — the B-1 set includes
//!    `benten-sync`. This is a REAL blocker for this crate (unlike the
//!    cfg-gated-table story in benten-sync's own pin, which is not one):
//!    benten-sync's `compile_error!` fires while building it as a
//!    dependency, so this crate can never be reached on wasm32.
//!    (Transitive gate — and today the operative one.)
//! 3. **CI at-build-time assertion** — `.github/workflows/wasm-checks.yml`
//!    job `membership-set-refuses-wasm32`, in three arms. Arm (a) runs
//!    `cargo build --target wasm32-unknown-unknown -p benten-membership-set`
//!    and asserts it fails; structural only — it dies in `getrandom` and no
//!    mutation of either gate changes it. Arm (b) re-runs that with
//!    getrandom's js/wasm_js features enabled and asserts the failure still
//!    carries `baked-in #17` from `crates/benten-sync/src/lib.rs` (rung 2,
//!    falsified by deleting BENTEN-SYNC's gate). Arm (c) compiles
//!    `src/lib.rs` ALONE with rustc and no `--extern` and asserts this
//!    crate's own marker appears — the only arm that can observe rung 1,
//!    falsified by deleting it. (At-build-time gate.)
//!
//! This test asserts ALL THREE defenses are present at the source-of-truth
//! manifests + the CI workflow.

#![allow(clippy::unwrap_used)]

#[test]
fn benten_membership_set_does_not_compile_for_wasm32_unknown_unknown_per_baked_in_17() {
    // CLAUDE.md baked-in #17 architectural pin. Verify the source + manifest
    // rungs (rungs 1 + 2):
    let crate_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest = std::fs::read_to_string(crate_root.join("Cargo.toml")).expect("read Cargo.toml");
    let lib_rs = std::fs::read_to_string(crate_root.join("src/lib.rs")).expect("read src/lib.rs");

    // Defense rung 1: lib.rs `compile_error!` for wasm32.
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

    // Defense rung 2: the manifest declares the native-only `benten-sync`
    // dependency (whose iroh/tokio transport is itself behind a
    // `cfg(not(target_arch = "wasm32"))` table), so the transitive dep chain
    // is unresolvable on wasm32 even if the lib.rs gate were bypassed.
    assert!(
        manifest.contains("benten-sync"),
        "Cargo.toml MUST reference benten-sync — the load-bearing native-only \
         B-1 dependency whose wasm32-excluded transport chain backstops the \
         lib.rs compile_error! gate per CLAUDE.md baked-in #17"
    );
}

#[test]
fn benten_membership_set_wasm32_refusal_pinned_in_ci_workflow() {
    // Defense-in-depth rung 3 (CI at-build-time assertion). Mirrors the
    // `benten-sync-refuses-wasm32` cell.
    //
    // Workflow-pin shape: assert `.github/workflows/wasm-checks.yml`
    // declares the `membership-set-refuses-wasm32` job that runs
    // `cargo build --target wasm32-unknown-unknown -p benten-membership-set`
    // and asserts the build fails with the `compile_error!` macro firing.
    //
    // Without this CI cell, defense-in-depth has only the declarative rungs
    // (lib.rs + Cargo.toml grep). A regression that removed `compile_error!`
    // while keeping the dep chain — or vice versa — would silently leave the
    // OTHER rung in place and pass the source-of-truth grep audits, but
    // at-build-time could surface either as a SUCCESS (wasm32 build that
    // should NOT succeed) or a different unrelated failure.
    let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    let workflow =
        std::fs::read_to_string(workspace_root.join(".github/workflows/wasm-checks.yml"))
            .expect("read .github/workflows/wasm-checks.yml");

    assert!(
        workflow.contains("membership-set-refuses-wasm32")
            || workflow.contains("membership-set refuses"),
        ".github/workflows/wasm-checks.yml MUST declare a \
         `membership-set-refuses-wasm32` job (CLAUDE.md baked-in #17 \
         defense-in-depth rung 3)"
    );

    assert!(
        workflow.contains("cargo build --target wasm32-unknown-unknown -p benten-membership-set"),
        ".github/workflows/wasm-checks.yml `membership-set-refuses-wasm32` job \
         MUST invoke `cargo build --target wasm32-unknown-unknown -p \
         benten-membership-set` to drive the at-build-time assertion"
    );

    // The CI cell must invert the exit code (build MUST fail) AND verify the
    // failure stderr cites the architectural gate (not an unrelated break).
    assert!(
        workflow.contains("compile_error!") || workflow.contains("baked-in #17"),
        ".github/workflows/wasm-checks.yml `membership-set-refuses-wasm32` job \
         MUST verify the build failure stderr cites `compile_error!` or \
         `baked-in #17` so the failure-classifier guards against unrelated \
         dep-graph breaks"
    );
}
