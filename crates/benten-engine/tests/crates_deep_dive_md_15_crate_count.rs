//! `docs/CRATES-DEEP-DIVE.md` 15-crate count drift detector (R11 MC-7).
//!
//! Mirror of `architecture_md_12_crate_count_post_phase_4_foundation_canaries.rs`
//! for the CRATES-DEEP-DIVE workspace-synthesis doc. The R11 council found that
//! CRATES-DEEP-DIVE.md self-contradicted on the crate count (said "14 workspace
//! crates" in the intro while the workspace is at 15) AND omitted the 15th crate
//! `benten-membership-set` from the per-crate walk. Without a drift guard the
//! doc would silently diverge from the actual `crates/` layout the way Phase-1
//! R7 audits caught aspirational-prose-but-dead-code regressions repeatedly
//! (CLAUDE.md: "Verify, don't trust docs").
//!
//! This guards the DOC side; the cite-drift detector guards the workspace side.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// Asserts CRATES-DEEP-DIVE.md states the 15-crate count AND names every one of
/// the seven post-Phase-1 crates — including `benten-membership-set` (the 15th),
/// the crate the R11 audit found missing.
#[test]
fn crates_deep_dive_md_says_fifteen_crates_and_lists_membership_set() {
    let root = workspace_root();
    let doc_path = root.join("docs/CRATES-DEEP-DIVE.md");

    let doc_src = std::fs::read_to_string(&doc_path).unwrap_or_else(|e| {
        panic!(
            "docs/CRATES-DEEP-DIVE.md not found at {} ({}); this is a \
             load-bearing workspace-synthesis doc.",
            doc_path.display(),
            e
        );
    });

    let lower = doc_src.to_ascii_lowercase();

    // A stale "14 workspace crates" / "Fourteen crates" count means readers
    // miss `benten-membership-set`. The workspace is at 15.
    assert!(
        !lower.contains("14 workspace crates") && !lower.contains("fourteen crates."),
        "docs/CRATES-DEEP-DIVE.md still carries a pre-F-full crate count \
         ('14 workspace crates' / 'Fourteen crates'). After \
         benten-membership-set joined the workspace as the 15th crate, the \
         doc MUST say '15 workspace crates' + 'Fifteen crates'."
    );

    let says_fifteen = lower.contains("15 workspace crates") || lower.contains("fifteen crates");
    assert!(
        says_fifteen,
        "docs/CRATES-DEEP-DIVE.md MUST explicitly state the 15-crate count \
         ('15 workspace crates' / 'Fifteen crates')."
    );

    // Every post-Phase-1 crate must appear by name — the R11 miss was
    // `benten-membership-set` specifically.
    for name in [
        "benten-id",
        "benten-sync",
        "benten-platform-foundation",
        "benten-renderer-tauri",
        "benten-crypto-suite",
        "benten-drop",
        "benten-membership-set",
    ] {
        assert!(
            lower.contains(name),
            "docs/CRATES-DEEP-DIVE.md MUST mention `{name}` by name \
             (the R11 audit found `benten-membership-set` missing)."
        );
    }
}
