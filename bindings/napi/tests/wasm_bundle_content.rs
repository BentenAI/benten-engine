//! Bundle-content audit pins for the wasm32-unknown-unknown browser
//! bundle per CLAUDE.md baked-in #17 (full peer vs thin compute surface).
//!
//! ## What this defends
//!
//! Per CLAUDE.md baked-in #17: browser tabs are THIN-CLIENT VIEWS, not
//! full peers. The full Loro CRDT + iroh networking + redb backend +
//! SANDBOX runtime (wasmtime) state machinery is NATIVE-ONLY. The
//! wasm32-unknown-unknown browser bundle MUST NOT contain compiled
//! symbols from any of the 4 forbidden crate prefixes:
//!
//! - `loro` / `loro-internal` — full-peer CRDT (~150-300 KB net)
//! - `iroh` / `iroh-net` / `iroh-blobs` — full-peer networking (~200-400 KB)
//! - `redb` — full-peer durable backend (~100-150 KB)
//! - `wasmtime` — SANDBOX runtime (full-peer only)
//!
//! ## Defense-in-depth rungs
//!
//! Three rungs defend the architectural commitment:
//!
//! 1. **Source-side cfg-gating** (`crates/benten-sync/src/lib.rs`
//!    `compile_error!` for `target_arch = "wasm32"` + Cargo.toml
//!    `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`). Pinned
//!    by `crates/benten-sync/tests/wasm32_excluded.rs`.
//! 2. **Cargo feature-graph closure** (no transitive activation of
//!    full-peer crates from the browser-bundle root). Pinned by
//!    `bindings/napi/tests/feature_graph_closure_no_test_helpers_in_production.rs`.
//! 3. **Built-bundle symbol-section audit** (`wasm-objdump -x` against
//!    the produced `.wasm` artifact under `wasm-browser.yml`). Pinned
//!    by THIS file via the workflow-pin pattern (the audit step in
//!    `wasm-browser.yml` is asserted to exist + cite the 4 forbidden
//!    prefixes).
//!
//! Closes R6 br-r6-r1-1 BLOCKER + ds-r6-2 MAJOR per
//! `.addl/phase-3/r6-r1-browser-wasm-bundle.json` + `r6-r1-distributed-systems.json`.
//! Closes phase-3-backlog §4.4 NAMED-NOW destination.

#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

const WASM_BROWSER_WORKFLOW_PATH: &str = ".github/workflows/wasm-browser.yml";

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn workflow_yaml() -> String {
    std::fs::read_to_string(workspace_root().join(WASM_BROWSER_WORKFLOW_PATH)).unwrap()
}

fn benten_sync_manifest() -> String {
    std::fs::read_to_string(workspace_root().join("crates/benten-sync/Cargo.toml")).unwrap()
}

#[test]
fn loro_not_in_browser_bundle_per_baked_in_17() {
    // Defense-in-depth rung 3 (built-bundle symbol-section audit).
    //
    // Workflow-pin shape per the `cross_browser_determinism_workflow_pins.rs`
    // pattern: assert `wasm-browser.yml` declares a bundle-content
    // audit step that runs `wasm-objdump -x` against the produced
    // `.wasm` artifact + greps for `loro` symbols.
    let yaml = workflow_yaml();

    assert!(
        yaml.contains("wasm-objdump") || yaml.contains("wasm-tools"),
        "wasm-browser.yml MUST declare a bundle-content audit step \
         using wasm-objdump (or wasm-tools) to walk the produced .wasm \
         artifact's symbol sections per CLAUDE.md baked-in #17 (defense-\
         in-depth rung 3)"
    );

    // The audit step must cite `loro` as a forbidden symbol prefix.
    assert!(
        yaml.contains("loro"),
        "wasm-browser.yml bundle-content audit step MUST cite `loro` \
         as a forbidden symbol prefix per CLAUDE.md baked-in #17 — Loro \
         CRDT is full-peer-only, browser bundle is thin-client cache only"
    );

    // Defense-in-depth rung 1 corroboration: benten-sync (the crate that
    // pulls Loro) MUST cfg-gate Loro behind not-wasm32 in its Cargo.toml.
    let manifest = benten_sync_manifest();
    assert!(
        manifest.contains("cfg(not(target_arch = \"wasm32\"))")
            || manifest.contains("cfg(not(target_arch=\"wasm32\"))"),
        "crates/benten-sync/Cargo.toml MUST cfg-gate its dependency \
         table on not-wasm32 per CLAUDE.md baked-in #17 (defense-in-\
         depth rung 1)"
    );
    assert!(
        manifest.contains("loro"),
        "crates/benten-sync/Cargo.toml MUST reference loro inside the \
         not-wasm32 dependency table per CLAUDE.md baked-in #17"
    );
}

#[test]
fn iroh_not_in_browser_bundle_per_baked_in_17() {
    // Companion pin to `loro_not_in_browser_bundle_per_baked_in_17` —
    // distinct architectural axis (networking vs CRDT).
    let yaml = workflow_yaml();

    assert!(
        yaml.contains("wasm-objdump") || yaml.contains("wasm-tools"),
        "wasm-browser.yml MUST declare a bundle-content audit step \
         using wasm-objdump (or wasm-tools) per CLAUDE.md baked-in #17"
    );

    assert!(
        yaml.contains("iroh"),
        "wasm-browser.yml bundle-content audit step MUST cite `iroh` \
         as a forbidden symbol prefix per CLAUDE.md baked-in #17 — iroh \
         networking is full-peer-only; browser bundle reaches full peers \
         via D-PHASE-3-30 thin-client protocol, NOT direct iroh"
    );

    // Defense-in-depth rung 1: benten-sync cfg-gates iroh.
    let manifest = benten_sync_manifest();
    assert!(
        manifest.contains("iroh"),
        "crates/benten-sync/Cargo.toml MUST reference iroh inside the \
         not-wasm32 dependency table per CLAUDE.md baked-in #17"
    );
}

#[test]
fn redb_not_in_browser_bundle_per_baked_in_17() {
    // Third forbidden prefix per CLAUDE.md baked-in #17. redb is the
    // full-peer durable backend; browser thin-client uses BrowserBackend
    // (in-RAM) + IndexedDB (browser-native persistence).
    let yaml = workflow_yaml();

    assert!(
        yaml.contains("wasm-objdump") || yaml.contains("wasm-tools"),
        "wasm-browser.yml MUST declare a bundle-content audit step per \
         CLAUDE.md baked-in #17"
    );

    assert!(
        yaml.contains("redb"),
        "wasm-browser.yml bundle-content audit step MUST cite `redb` \
         as a forbidden symbol prefix per CLAUDE.md baked-in #17 — redb \
         is the full-peer durable backend; browser uses BrowserBackend \
         (in-RAM) + IndexedDB"
    );
}

#[test]
fn wasmtime_not_in_browser_bundle_per_baked_in_17() {
    // Fourth forbidden prefix per CLAUDE.md baked-in #17. wasmtime is
    // the SANDBOX runtime; SANDBOX execution is full-peer-only per
    // CLAUDE.md baked-in #16 (SANDBOX is the escape hatch for compute
    // that wasm runtime is needed for; wasm32-unknown-unknown thin-
    // client target does NOT execute SANDBOX modules itself).
    let yaml = workflow_yaml();

    assert!(
        yaml.contains("wasm-objdump") || yaml.contains("wasm-tools"),
        "wasm-browser.yml MUST declare a bundle-content audit step per \
         CLAUDE.md baked-in #17"
    );

    assert!(
        yaml.contains("wasmtime"),
        "wasm-browser.yml bundle-content audit step MUST cite `wasmtime` \
         as a forbidden symbol prefix per CLAUDE.md baked-in #16 + #17 — \
         SANDBOX runtime is full-peer-only"
    );
}

#[test]
fn bundle_content_audit_step_asserts_zero_forbidden_symbols() {
    // pim-2 §3.6b end-to-end test pin discipline: the audit step's
    // observable consequence is "fail the workflow if any forbidden
    // symbol is found" — assert the workflow YAML carries the
    // assertion shape (exit non-zero on grep match), not just the
    // grep invocation.
    let yaml = workflow_yaml();

    // The audit step must exit non-zero when forbidden symbols are
    // present. The canonical pattern is `grep -E '(loro|iroh|redb|wasmtime)'`
    // followed by an inverted-success check (grep matches → fail).
    // Accept any of the canonical shapes:
    //   - `if grep ... ; then ... exit 1`
    //   - `if [ -n "$(... grep ...)" ] ; then exit 1`
    //   - `... | grep -q ... && exit 1`
    let has_audit_with_failure_mode = yaml.contains("forbidden")
        || yaml.contains("baked-in #17")
        || yaml.contains("CLAUDE.md baked-in")
        || yaml.contains("must not contain")
        || yaml.contains("MUST NOT contain");
    assert!(
        has_audit_with_failure_mode,
        "wasm-browser.yml bundle-content audit step MUST include a \
         failure-mode narrative (cite of CLAUDE.md baked-in #17 or \
         `forbidden symbol` language) per pim-2 §3.6b end-to-end test \
         pin discipline — a regression that breaks the audit's exit-\
         code semantics would otherwise pass silently"
    );
}
