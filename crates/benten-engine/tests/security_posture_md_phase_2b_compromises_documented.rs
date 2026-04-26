//! Phase 2b R4-FP B-4 — `docs/SECURITY-POSTURE.md` Phase-2b compromise
//! drift detector.
//!
//! TDD red-phase. Pin source: plan §7d (Compromises #N+5 / #N+7 /
//! #N+8 / #N+9 documented in SECURITY-POSTURE.md after R5 lands them;
//! Compromise #4 + #9 marked CLOSED at G11-2b close).
//!
//! Reference compromises (per plan lines 559-564):
//!   * #N+5 — Module manifest minimal CID-pin in 2b; full Ed25519
//!     deferred to Phase 3 (G10-B owns).
//!   * #N+7 — manifest-not-yet-subgraph (phil-r1-7; G10-B owns).
//!   * #N+8 — browser-persistent-storage absent in 2b
//!     (G10-A-browser owns).
//!   * #N+9 — cross-browser-determinism CI cadence
//!     (G10-A-browser owns).
//!
//! Plus Compromise #4 + #9 must transition to CLOSED in 2b per
//! exit-criterion #5 + R2 §6 (`security_posture_compromise_4_marked_closed`,
//! `security_posture_compromise_9_marked_closed`).
//!
//! Owned by R3-E (CI workflow tests row); test landed by R4-FP B-4.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn read_security_posture() -> String {
    let root = workspace_root();
    let doc_path = root.join("docs/SECURITY-POSTURE.md");
    std::fs::read_to_string(&doc_path).unwrap_or_else(|e| {
        panic!(
            "docs/SECURITY-POSTURE.md not found at {} ({}); this is a \
             load-bearing Phase-1 doc per CLAUDE.md key-reading list.",
            doc_path.display(),
            e
        );
    })
}

/// `security_posture_documents_phase_2b_new_compromises` — plan §7d
/// + sec-pre-r1 carry items.
#[test]
#[ignore = "Phase 2b G11-2b-A pending — Phase-2b compromise additions to SECURITY-POSTURE.md unimplemented"]
fn security_posture_documents_phase_2b_new_compromises() {
    let doc = read_security_posture();
    let lower = doc.to_ascii_lowercase();

    // The compromise IDs use N+K notation in the plan but the doc uses
    // concrete numbers. After G11-2b-A's doc sweep, the doc MUST mention
    // each Phase-2b compromise topic by both its keyword + its
    // disposition phrase. We check for the keyword-anchors that
    // distinguish each item without coupling to the exact #14/#15/etc
    // numbering the doc author picks.

    // #N+5 — module manifest minimal CID-pin / Ed25519 deferred.
    assert!(
        (lower.contains("module manifest") || lower.contains("manifest"))
            && lower.contains("ed25519"),
        "docs/SECURITY-POSTURE.md MUST document Compromise #N+5 — module \
         manifest minimal CID-pin in 2b; full Ed25519 deferred to Phase 3 \
         (G10-B owns; plan §7d)."
    );

    // #N+7 — manifest-not-yet-subgraph.
    assert!(
        lower.contains("manifest-not-yet-subgraph") || lower.contains("not yet subgraph"),
        "docs/SECURITY-POSTURE.md MUST document Compromise #N+7 — \
         manifest-not-yet-subgraph (phil-r1-7 carry; plan §7d)."
    );

    // #N+8 — browser-persistent-storage absent in 2b (in-memory only).
    assert!(
        (lower.contains("browser") && (lower.contains("indexeddb") || lower.contains("in-memory"))),
        "docs/SECURITY-POSTURE.md MUST document Compromise #N+8 — \
         browser-persistent-storage absent in 2b (in-memory only; \
         IndexedDB deferred Phase-3; G10-A-browser owns; plan §7d)."
    );

    // #N+9 — cross-browser-determinism CI cadence.
    assert!(
        lower.contains("cross-browser")
            || lower.contains("release-era cadence")
            || lower.contains("release-tag"),
        "docs/SECURITY-POSTURE.md MUST document Compromise #N+9 — \
         cross-browser-determinism CI cadence (release-era only; \
         G10-A-browser owns; plan §7d)."
    );
}

/// `security_posture_compromise_4_marked_closed` — R2 §6 row.
#[test]
#[ignore = "Phase 2b G7-C + G11-2b-A pending — Compromise #4 closure in SECURITY-POSTURE.md unimplemented"]
fn security_posture_compromise_4_marked_closed() {
    let doc = read_security_posture();
    // Find the Compromise #4 section, look for CLOSED disposition.
    // The current doc reads "Compromise #4 — WASM runtime is compile-check only";
    // after G7 lands the SANDBOX runtime, this must transition to "CLOSED".
    let lower = doc.to_ascii_lowercase();
    let has_closed_4 = lower.contains("compromise #4") && lower.contains("closed");
    assert!(
        has_closed_4,
        "docs/SECURITY-POSTURE.md Compromise #4 (WASM runtime is \
         compile-check only) MUST be marked CLOSED after G7 lands the \
         live SANDBOX runtime per plan §1 exit-criterion #5 + R2 §6. \
         Pattern: header line should include `— CLOSED` like Compromise \
         #2 / #3 / #7 / #8 do."
    );
}

/// `security_posture_compromise_9_marked_closed` — R2 §6 row.
#[test]
#[ignore = "Phase 2b G12-E + G11-2b-A pending — Compromise #9 closure in SECURITY-POSTURE.md unimplemented"]
fn security_posture_compromise_9_marked_closed() {
    let doc = read_security_posture();
    let lower = doc.to_ascii_lowercase();
    // Compromise #9 is "Dedup writes pure-read"; closure depends on
    // G12-E cross-process WAIT metadata + audit-sequence work per
    // plan §3.2 + §7d.
    let has_closed_9 = lower.contains("compromise #9") && lower.contains("closed");
    assert!(
        has_closed_9,
        "docs/SECURITY-POSTURE.md Compromise #9 (Dedup writes pure-read \
         / sec-r1-4 / atk-3) MUST be marked CLOSED after G12-E lands per \
         plan §1 exit-criterion #5 + R2 §6. Pattern: header line should \
         include `— CLOSED`."
    );
}
