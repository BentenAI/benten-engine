//! R4-FP-3 RED-PHASE pin: CI-side materializer determinism gate
//! (canonical bytes stable across runs).
//!
//! ## Pin sources
//!
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.13 row 2.
//! - `.addl/phase-4-foundation/r4-triage.md` §5.3 R4-FP-3 charter.
//! - mat-r1-3: materializer output MUST be deterministic across runs;
//!   the per-crate test
//!   `crates/benten-platform-foundation/tests/materializer_canonical_bytes_determinism_across_runs.rs`
//!   shipped at R3 Family E; the CI-side cross-run gate composes with
//!   it (different test runner / different worker / repeated invocation
//!   produces identical canonical bytes).
//!
//! ## What this pin asserts
//!
//! Two arms (defense in depth):
//!
//! 1. The companion per-crate test
//!    `materializer_canonical_bytes_determinism_across_runs.rs` exists
//!    at `crates/benten-platform-foundation/tests/`.
//! 2. A CI workflow OR a gating composition is registered that runs the
//!    materializer determinism test in the always-required CI surface
//!    (either via an existing `determinism.yml` extension OR a new
//!    workflow file). The composition prevents the materializer
//!    determinism gate from silently regressing.

#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    PathBuf::from(&manifest_dir)
        .parent()
        .and_then(std::path::Path::parent)
        .map(std::path::Path::to_path_buf)
        .expect("workspace root")
}

#[test]
#[ignore = "phase-4-foundation R4-FP-3 RED-PHASE — G26-B wave-10 un-ignores. \
    Pin source: r2-test-landscape.md §2.13 row 2 + mat-r1-3. CI-side materializer determinism \
    gate composes with the per-crate test shipped at R3 Family E."]
fn materializer_determinism_canonical_bytes_stable_across_runs() {
    let root = workspace_root();

    // ARM 1: per-crate determinism test exists.
    let per_crate_test =
        root.join("crates/benten-platform-foundation/tests/materializer_canonical_bytes_determinism_across_runs.rs");
    assert!(
        per_crate_test.is_file(),
        "Per-crate materializer determinism test MUST exist at {} (shipped at R3 Family E \
         per r2-test-landscape §2.5 G23-B). Without it the CI gate has nothing to gate.",
        per_crate_test.display()
    );

    // ARM 2: CI workflow registers the materializer determinism test.
    // Look for either an existing determinism workflow extension OR a
    // new dedicated workflow. The implementer at G26-B picks the
    // composition shape; this test asserts SOMETHING references the
    // materializer determinism path in `.github/workflows/`.
    let workflows = root.join(".github/workflows");
    assert!(workflows.is_dir(), ".github/workflows/ MUST exist");

    let mut found_reference = false;
    for entry in std::fs::read_dir(&workflows).unwrap().flatten() {
        let p = entry.path();
        if !p.extension().is_some_and(|e| e == "yml" || e == "yaml") {
            continue;
        }
        let body = std::fs::read_to_string(&p).unwrap_or_default();
        if body.contains("materializer_canonical_bytes_determinism_across_runs")
            || body.contains("materializer-determinism")
            || body.contains("materializer determinism")
            || (body.contains("materializer") && body.contains("determinism"))
        {
            found_reference = true;
            break;
        }
    }

    assert!(
        found_reference,
        "At least one workflow in .github/workflows/ MUST reference the materializer \
         determinism gate (either by test name or by descriptive label). Without the CI \
         registration, the per-crate test could silently regress without surfacing on PR. \
         G26-B wave-10 wires this composition per meth-r1-9 + mat-r1-3."
    );
}
