//! `docs/ARCHITECTURE.md` 12-crate count drift detector.
//!
//! The workspace landed at 12 crates after `benten-platform-foundation`
//! (11th, Phase 4-Foundation — schema-rendering + materializer + plugin
//! manifest + admin UI v0 + `Renderer` trait abstraction) and
//! `benten-renderer-tauri` (12th, Phase 4-Foundation — Tauri 2.x
//! renderer engine extension per CLAUDE.md baked-in #19).
//!
//! ARCHITECTURE.md enumerates all 12 crates by name with `benten-sync`
//! flagged native-only per CLAUDE.md baked-in #17. The cite-drift
//! detector source-of-truth derives the count dynamically from
//! `Cargo.toml` per `tools/cite-drift-detector/src/lib.rs::derive_crate_count_from_workspace`,
//! so this test guards the DOC side while the detector guards the
//! workspace side.
//!
//! Drift discipline: doc-as-source-of-truth on the workspace shape
//! must agree with the actual `crates/` directory layout. Without
//! this test, ARCHITECTURE.md would silently drift from the workspace
//! manifest the way Phase-1 R7 audits caught aspirational-prose-but-
//! dead-code regressions repeatedly (CLAUDE.md: "Verify, don't trust
//! docs"). G26-A pre-tag retense (Phase 4-Foundation R6-FP-G) renamed
//! this file from `architecture_md_10_crate_count_post_phase_3_canaries.rs`
//! and retensed every assertion to the 12-crate shape.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// `architecture_md_says_twelve_crates_after_phase_4_foundation_canaries`.
///
/// Asserts ARCHITECTURE.md says "Twelve crates" (or "12 crates") in the
/// section header AND in the prose body, with `benten-platform-foundation`
/// + `benten-renderer-tauri` listed as workspace members alongside the
/// Phase-3 `benten-id` + `benten-sync` rows. The pre-Phase-4-Foundation
/// phrasing was "Ten crates"; if the doc still says "Ten", operators
/// reading the doc miss the new platform-shippable surfaces.
#[test]
fn architecture_md_says_twelve_crates_after_phase_4_foundation_canaries() {
    let root = workspace_root();
    let doc_path = root.join("docs/ARCHITECTURE.md");

    let doc_src = std::fs::read_to_string(&doc_path).unwrap_or_else(|e| {
        panic!(
            "docs/ARCHITECTURE.md not found at {} ({}); this is a \
             load-bearing doc — pre-existing in Phase 1.",
            doc_path.display(),
            e
        );
    });

    let lower = doc_src.to_ascii_lowercase();

    // The "Eight crates" / "Ten crates" pre-Phase-4-Foundation canonical
    // headings MUST be gone. Historical narrative ("8 → 10 → 12
    // transition") is allowed because it describes past states
    // accurately.
    assert!(
        !lower.contains("## ten crates")
            && !lower.contains("## eight crates")
            && !lower.contains("# ten crates")
            && !lower.contains("# eight crates"),
        "docs/ARCHITECTURE.md still carries a pre-Phase-4-Foundation \
         heading 'Ten crates' / 'Eight crates'. After \
         benten-platform-foundation + benten-renderer-tauri join the \
         workspace, the canonical heading MUST say 'Twelve crates' \
         (paired with the cite-drift detector source-of-truth)."
    );

    // Should explicitly assert the new count.
    let says_twelve = lower.contains("twelve crates")
        || lower.contains("## 12 crates")
        || lower.contains("# twelve")
        || lower.contains("twelve rust crates");
    assert!(
        says_twelve,
        "docs/ARCHITECTURE.md MUST explicitly state 'Twelve crates' / \
         '## 12 crates' / similar with benten-platform-foundation + \
         benten-renderer-tauri as workspace members."
    );

    // The four post-Phase-1 crates must all be listed by name.
    for name in [
        "benten-id",
        "benten-sync",
        "benten-platform-foundation",
        "benten-renderer-tauri",
    ] {
        assert!(
            lower.contains(name),
            "docs/ARCHITECTURE.md MUST mention `{name}` by name."
        );
    }

    // benten-sync must be marked native-only per CLAUDE.md baked-in #17.
    assert!(
        lower.contains("native-only") || lower.contains("native only"),
        "docs/ARCHITECTURE.md MUST declare benten-sync as native-only \
         per CLAUDE.md baked-in #17 (excluded from wasm32 targets)."
    );

    // The pre-existing dsl-compiler row must remain.
    assert!(
        lower.contains("benten-dsl-compiler"),
        "docs/ARCHITECTURE.md MUST mention `benten-dsl-compiler` by name."
    );
}

/// Workspace-shape sanity check — verifies the actual `crates/` layout
/// matches the doc. Asserts the four post-Phase-1 crate directories
/// are present so the 12-crate doc claim is not aspirational. Also
/// guards against silent removal of a Phase-3 / Phase-4-Foundation
/// crate.
#[test]
fn workspace_has_phase_4_foundation_canary_crate_dirs() {
    let root = workspace_root();
    for name in [
        "benten-dsl-compiler",
        "benten-id",
        "benten-sync",
        "benten-platform-foundation",
        "benten-renderer-tauri",
    ] {
        let dir = root.join("crates").join(name);
        assert!(
            dir.is_dir(),
            "crates/{name}/ MUST exist. Without the directory, the \
             12-crates phrasing in ARCHITECTURE.md would be aspirational \
             (the regression Phase-1 R7 audit caught repeatedly)."
        );
    }
}
