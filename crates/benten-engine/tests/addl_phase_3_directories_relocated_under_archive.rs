//! Phase 4-Foundation R3 (Family A — G22-D orchestrator-direct archive
//! consolidation). RED-PHASE grep-assert: at R5 G22-D merge time, the
//! `.addl/phase-3/`, `.addl/phase-3-doc-review/`, and
//! `.addl/phase-3-test-review/` directories MUST have been relocated
//! under `.addl/_archive/phase-3/{,doc-review/,test-review/}` per the
//! G22-D `git mv` plan (matches Phase-2b precedent at
//! `.addl/_archive/phase-2b/`).
//!
//! # Charter
//!
//! Per `.addl/phase-4-foundation/r2-test-landscape.md` §2.1 G22-D row +
//! `.addl/phase-4-foundation/00-implementation-plan.md` §3 wave-1 G22-D
//! files-owned:
//!
//! ```text
//! git mv .addl/phase-3/                 .addl/_archive/phase-3/
//! git mv .addl/phase-3-doc-review/      .addl/_archive/phase-3/doc-review/
//! git mv .addl/phase-3-test-review/     .addl/_archive/phase-3/test-review/
//! ```
//!
//! # What this pin asserts (would-FAIL-if-no-op'd per §3.6b)
//!
//! Two halves of the consolidation contract:
//!
//! 1. The OLD top-level locations `.addl/phase-3/`,
//!    `.addl/phase-3-doc-review/`, `.addl/phase-3-test-review/` MUST
//!    NOT exist as directories anymore.
//! 2. The NEW archived locations `.addl/_archive/phase-3/`,
//!    `.addl/_archive/phase-3/doc-review/`,
//!    `.addl/_archive/phase-3/test-review/` MUST exist as directories
//!    AND must contain content (at minimum one file each — the empty
//!    move would not satisfy the consolidation goal of preserving
//!    Phase-3 forensic artifacts).
//!
//! Half-landing the consolidation (moving doc-review but leaving
//! phase-3/ in place; or moving phase-3/ but failing to nest doc-
//! review/test-review under it) trips one of the assertions.
//!
//! # Cross-references
//!
//! - HANDOFFs / brief / PHASE-3.md / CLAUDE.md cross-references MUST
//!   also be updated to point at the new path; that retense work is
//!   separately tracked by R5 G22-D commit body (no grep-assert here
//!   because path strings appear in narrative prose and bringing all
//!   of those under a strict grep gate would be brittle against
//!   legitimate retrospective mentions of the historical path). The
//!   directory-relocation half is the load-bearing piece this test
//!   pins.
//!
//! # RED-PHASE
//!
//! At write-time (R3 Family A; base SHA `f3930e1`) the directory move
//! has NOT been performed — `.addl/phase-3/` still lives at the top
//! level alongside `.addl/_archive/` (which already exists with
//! frozen Phase-1/2a/2b archives). Therefore both assertions in this
//! test fail against current HEAD — `#[ignore]`-marked with a RED-
//! PHASE tag; R5 G22-D un-ignores when the `git mv` lands.
//!
//! # Owned by
//!
//! Phase 4-Foundation R3 Family A test-writer. Closes at R5 G22-D
//! (orchestrator-direct).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};

/// Workspace root resolved from `CARGO_MANIFEST_DIR` of the
/// `benten-engine` crate (`crates/benten-engine`).
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// `true` iff `path` exists and is a directory.
fn is_dir(path: &Path) -> bool {
    std::fs::metadata(path).is_ok_and(|m| m.is_dir())
}

/// `true` iff `dir` is a directory AND contains at least one entry
/// (regular file or sub-directory).
fn dir_has_content(dir: &Path) -> bool {
    if !is_dir(dir) {
        return false;
    }
    match std::fs::read_dir(dir) {
        Ok(mut it) => it.next().is_some(),
        Err(_) => false,
    }
}

#[test]
#[ignore = "RED-PHASE: closes at R5 G22-D (.addl archive consolidation). Un-ignore after `git mv` of .addl/phase-3 + .addl/phase-3-doc-review + .addl/phase-3-test-review under .addl/_archive/phase-3/."]
fn old_phase_3_top_level_directories_no_longer_exist() {
    let root = workspace_root();
    let old_paths = [
        root.join(".addl/phase-3"),
        root.join(".addl/phase-3-doc-review"),
        root.join(".addl/phase-3-test-review"),
    ];

    let still_present: Vec<&Path> = old_paths
        .iter()
        .filter(|p| is_dir(p))
        .map(|p| p.as_path())
        .collect();

    assert!(
        still_present.is_empty(),
        "expected NO top-level .addl/phase-3* directories after G22-D \
         consolidation, but found {} still present: {:?} (un-ignore + \
         un-block this test once `git mv .addl/phase-3* .addl/_archive/phase-3/` \
         lands)",
        still_present.len(),
        still_present,
    );
}

#[test]
#[ignore = "RED-PHASE: closes at R5 G22-D (.addl archive consolidation). Un-ignore after `git mv` of .addl/phase-3 + .addl/phase-3-doc-review + .addl/phase-3-test-review under .addl/_archive/phase-3/."]
fn archived_phase_3_directories_exist_under_addl_archive_and_have_content() {
    let root = workspace_root();
    let archived_paths = [
        root.join(".addl/_archive/phase-3"),
        root.join(".addl/_archive/phase-3/doc-review"),
        root.join(".addl/_archive/phase-3/test-review"),
    ];

    let missing: Vec<&Path> = archived_paths
        .iter()
        .filter(|p| !dir_has_content(p))
        .map(|p| p.as_path())
        .collect();

    assert!(
        missing.is_empty(),
        "expected ALL archived Phase-3 directories to exist with content \
         under .addl/_archive/phase-3/, but {} are missing or empty: {:?} \
         (un-ignore + un-block this test once `git mv` of phase-3 + \
         phase-3-doc-review + phase-3-test-review lands and content is \
         preserved)",
        missing.len(),
        missing,
    );
}
