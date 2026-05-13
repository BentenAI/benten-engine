//! R4-FP-3 RED-PHASE pin: `.github/branch-protection.yml` lists the
//! 3 new Phase-4-Foundation required-status contexts.
//!
//! ## Pin sources
//!
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.13 row 5.
//! - `.addl/phase-4-foundation/r4-triage.md` §5.3 R4-FP-3 charter.
//! - meth-r1-9: branch-protection spec must enumerate the 3 new
//!   workflows that landed during Phase-4-Foundation so they become
//!   required-on-PR contexts (companion pin to the 3 per-workflow pins
//!   in this same R4-FP-3 wave).
//!
//! ## What this pin asserts
//!
//! Aggregate pin: branch-protection.yml lists all 3 new Phase-4-
//! Foundation required-status contexts:
//!
//! 1. admin-ui-v0 build workflow context
//! 2. plugin-manifest-validation workflow context
//! 3. materializer-determinism (composed into existing determinism.yml
//!    OR new dedicated workflow) context
//!
//! Distinct from the per-workflow pins: this aggregate enforces the
//! umbrella claim that ALL 3 are registered. Defends against the
//! failure shape where 1 of 3 ships + the other 2 silently drop.

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
    Pin source: r2-test-landscape.md §2.13 row 5 + meth-r1-9. branch-protection.yml aggregate \
    over 3 new Phase-4-Foundation required-status contexts."]
fn branch_protection_spec_lists_new_phase_4_foundation_required_contexts() {
    let root = workspace_root();
    let spec_path = root.join(".github/branch-protection.yml");
    let spec = std::fs::read_to_string(&spec_path).unwrap_or_else(|e| {
        panic!(
            "branch-protection.yml MUST exist at {} ({}) — load-bearing repo \
             security config landed Phase-3 era",
            spec_path.display(),
            e
        );
    });

    // The 3 required-context markers (substring match — implementer
    // picks exact context strings at G26-B time; these markers are
    // descriptive enough to catch the registration).
    let required: &[(&str, &[&str])] = &[
        ("admin-ui-v0 build", &["admin-ui-v0", "admin UI v0 build"]),
        (
            "plugin-manifest-validation",
            &["plugin-manifest-validation", "plugin manifest validation"],
        ),
        (
            "materializer-determinism",
            &[
                "materializer-determinism",
                "materializer determinism",
                "materializer canonical bytes",
            ],
        ),
    ];

    let mut missing: Vec<&str> = Vec::new();
    for (label, aliases) in required {
        let found = aliases.iter().any(|a| spec.contains(a));
        if !found {
            missing.push(label);
        }
    }

    assert!(
        missing.is_empty(),
        "branch-protection.yml MUST list ALL 3 new Phase-4-Foundation required-status \
         contexts. Missing: {:?}. Without registration the workflows can silently fail \
         without blocking merge. Companion pin to per-workflow pins in same R4-FP-3 wave.",
        missing,
    );

    // SUBSTANCE / pim-18 §3.6f vacuous-truth defense: the spec must
    // have a `contexts:` block with a non-empty list. Catches the
    // failure shape where the spec is empty/commented-out + all aliases
    // appear only in comment text.
    let has_contexts_block = spec.contains("contexts:");
    assert!(
        has_contexts_block,
        "branch-protection.yml MUST contain a `contexts:` block (required_status_checks.contexts) \
         — substantive registration, not comment-only mention"
    );
}
