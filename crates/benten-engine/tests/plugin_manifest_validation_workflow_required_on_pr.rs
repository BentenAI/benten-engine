//! R4-FP-3 RED-PHASE pin: `.github/workflows/plugin-manifest-validation.yml`
//! exists AND is required-on-PR per branch-protection spec.
//!
//! ## Pin sources
//!
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.13 row 3.
//! - `.addl/phase-4-foundation/r4-triage.md` §5.3 R4-FP-3 charter.
//! - meth-r1-9 + D-4F-1: plugin manifest validation workflow MUST be
//!   required-on-PR so plugin manifests can't drift from the schema
//!   silently (Phase 4-Foundation G24-D ships the manifest schema +
//!   validator; this workflow gates manifest changes at PR time).

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
    Pin source: r2-test-landscape.md §2.13 row 3 + meth-r1-9 + D-4F-1. plugin-manifest- \
    validation.yml workflow exists + required-on-PR via branch-protection spec."]
fn plugin_manifest_validation_workflow_required_on_pr() {
    let root = workspace_root();
    let workflow = root.join(".github/workflows/plugin-manifest-validation.yml");

    assert!(
        workflow.is_file(),
        "plugin-manifest-validation workflow MUST exist at {} after G26-B wave-10 lands \
         the CI surface for plugin manifest schema validation (meth-r1-9 + D-4F-1). \
         Without it manifest changes can ship without schema-check gating.",
        workflow.display()
    );

    let spec_path = root.join(".github/branch-protection.yml");
    let spec = std::fs::read_to_string(&spec_path).unwrap_or_else(|e| {
        panic!(
            "branch-protection.yml MUST exist at {} ({})",
            spec_path.display(),
            e
        );
    });

    let mentions =
        spec.contains("plugin-manifest-validation") || spec.contains("plugin manifest validation");
    assert!(
        mentions,
        ".github/branch-protection.yml MUST list the plugin-manifest-validation workflow's \
         context string in required_status_checks.contexts per meth-r1-9 + D-4F-1. Without \
         the required-status registration a manifest schema-violation PR could merge silently."
    );
}
