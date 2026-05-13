//! R4-FP-3 RED-PHASE pin: `.github/workflows/admin-ui-v0-build.yml`
//! exists AND is required-on-PR per branch-protection spec.
//!
//! ## Pin sources
//!
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.13 row 1.
//! - `.addl/phase-4-foundation/r4-triage.md` §5.3 R4-FP-3 charter.
//! - meth-r1-9 + D-4F-4: admin UI v0 build workflow (vitest browser
//!   mode + Playwright) MUST be required-on-PR so the admin UI plugin
//!   surface can't regress silently.
//!
//! ## What this pin asserts
//!
//! Two arms (defense in depth):
//! 1. The workflow YAML file exists at `.github/workflows/admin-ui-v0-build.yml`.
//! 2. The branch-protection spec at `.github/branch-protection.yml`
//!    lists the workflow's primary context string in
//!    `required_status_checks.contexts`.

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
    Pin source: r2-test-landscape.md §2.13 row 1 + meth-r1-9 + D-4F-4. admin-ui-v0-build.yml \
    workflow exists + required-on-PR via branch-protection spec."]
fn admin_ui_v0_build_workflow_required_on_pr() {
    let root = workspace_root();
    let workflow = root.join(".github/workflows/admin-ui-v0-build.yml");

    assert!(
        workflow.is_file(),
        "admin-ui-v0 build workflow MUST exist at {} after G26-B wave-10 lands the \
         CI surface for the admin UI v0 build path (meth-r1-9 + D-4F-4)",
        workflow.display()
    );

    let spec_path = root.join(".github/branch-protection.yml");
    let spec = std::fs::read_to_string(&spec_path).unwrap_or_else(|e| {
        panic!(
            "branch-protection.yml MUST exist at {} ({}) — load-bearing repo \
             security config landed Phase-3 era",
            spec_path.display(),
            e
        );
    });

    // The spec MUST list admin-ui-v0-build as a required context string.
    // The exact context string depends on the workflow job's `name:`
    // field, so we match on a substring that the implementer chooses
    // at G26-B time. Per existing branch-protection.yml shape, context
    // strings are quoted YAML strings — we look for any string mentioning
    // admin-ui-v0 / admin UI v0.
    let mentions = spec.contains("admin-ui-v0") || spec.contains("admin UI v0");
    assert!(
        mentions,
        ".github/branch-protection.yml MUST list the admin-ui-v0 build workflow's \
         context string in required_status_checks.contexts per meth-r1-9 + D-4F-4. \
         Without the required-status registration the workflow can silently fail without \
         blocking merge."
    );
}
