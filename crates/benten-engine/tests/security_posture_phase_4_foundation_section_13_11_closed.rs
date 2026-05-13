//! R4-FP-3 RED-PHASE pin: `docs/SECURITY-POSTURE.md` §13.11 closure
//! narrative present after Phase-4-Foundation close.
//!
//! ## Pin sources
//!
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.12 row 3.
//! - `.addl/phase-4-foundation/r4-triage.md` §5.3 R4-FP-3 charter.
//! - sec-3.5-r1-13: SECURITY-POSTURE.md §13.11 historical-fix narrative
//!   landed at Phase-4-Foundation close (UCAN revocation observance
//!   gap closure via PR #199 namespace-mismatch root-cause fix +
//!   `Engine::revoke_capability_by_grant_cid` shipped).
//!
//! ## What this pin asserts
//!
//! Phase-4-Foundation Track B (pre-v1 cleanup window) shipped a closure
//! narrative for the UCAN revocation observance gap. The narrative
//! must be documented at §13.11 in SECURITY-POSTURE.md as a HISTORICAL
//! fix with phase-precise destination (PR #199 reference + namespace-
//! mismatch root cause + `revoke_capability_by_grant_cid` API name).
//!
//! Mirrors `security_posture_compromise_9_marked_closed.rs` shape (the
//! Phase-2b G12-E equivalent — CLOSED-at-Phase-N marker grep-assert).

#![allow(clippy::unwrap_used)]

use std::fs;
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
#[ignore = "phase-4-foundation R4-FP-3 RED-PHASE — G26-A wave-10 un-ignores. \
    Pin source: r2-test-landscape.md §2.12 row 3 + sec-3.5-r1-13. §13.11 UCAN revocation \
    observance closure narrative landed at Phase-4-Foundation Track B close."]
fn security_posture_phase_4_foundation_section_13_11_closed() {
    let posture = workspace_root().join("docs/SECURITY-POSTURE.md");
    let body = fs::read_to_string(&posture).expect("read SECURITY-POSTURE.md");

    // The §13.11 section MUST be present.
    let has_section = body.contains("## 13.11")
        || body.contains("### 13.11")
        || body.contains("§13.11")
        || body.contains("13.11");
    assert!(
        has_section,
        "SECURITY-POSTURE.md MUST carry §13.11 section after Phase-4-Foundation Track B \
         shipped the UCAN revocation observance closure (PR #199) — sec-3.5-r1-13"
    );

    // The closure narrative MUST name the canonical components: PR #199
    // reference + the API name + the namespace-mismatch root-cause label.
    assert!(
        body.contains("PR #199") || body.contains("#199"),
        "SECURITY-POSTURE.md §13.11 MUST reference PR #199 (the closing PR) so \
         the historical fix is traceable"
    );
    assert!(
        body.contains("revoke_capability_by_grant_cid"),
        "SECURITY-POSTURE.md §13.11 MUST name the shipped API \
         `Engine::revoke_capability_by_grant_cid` so future readers can trace the \
         closure surface"
    );
    assert!(
        body.contains("namespace")
            && (body.contains("mismatch") || body.contains("namespace mismatch")),
        "SECURITY-POSTURE.md §13.11 MUST name the root cause (namespace mismatch) \
         per sec-3.5-r1-13 historical narrative"
    );

    // CLOSED-at-phase marker per existing pattern (see
    // `security_posture_compromise_9_marked_closed.rs`).
    let has_closed_marker = body.contains("CLOSED at Phase 4-Foundation")
        || body.contains("CLOSED at Phase-4-Foundation")
        || body.contains("CLOSED at Track B")
        || body.contains("HISTORICAL");
    assert!(
        has_closed_marker,
        "SECURITY-POSTURE.md §13.11 MUST carry a `CLOSED at Phase 4-Foundation` (or \
         equivalent HISTORICAL) marker so future readers can grep for closure state"
    );
}
