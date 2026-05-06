//! G15-B (wave-5a) — `SECURITY-POSTURE.md` drift-detector forward-pointer
//! resolution.
//!
//! Pre-G15-B the SECURITY-POSTURE narrative carried a forward-pointer
//! advertising a future drift-detector. G15-B lands the drift-detector
//! (`crates/benten-ivm/tests/algorithm_b_drift_detector.rs`); this test
//! pins the doc to reference the test path so the doc surface tracks the
//! actual implementation surface (pim-1 §3.5b post-fix doc-coupling).

#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR is `<repo>/tests/phase_3_workspace`; the
    // workspace root is two levels up.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .map(std::path::Path::to_path_buf)
        .expect("workspace root")
}

#[test]
fn security_posture_drift_detector_forward_pointer_resolved_to_g15_b() {
    let path = workspace_root().join("docs/SECURITY-POSTURE.md");
    let body = std::fs::read_to_string(&path).expect("read SECURITY-POSTURE.md");

    // Forward-pointer phrasing MUST be gone.
    assert!(
        !body.contains("drift-detector lands at Phase 3 G15-B"),
        "SECURITY-POSTURE.md still carries the G15-B forward-pointer; \
         G15-B must resolve it"
    );

    // Resolution MUST reference either the test pin path or assert the
    // landing — keeps the assertion tolerant of editorial wording while
    // pinning the architectural claim.
    let resolved =
        body.contains("algorithm_b_drift_detector") || body.contains("drift-detector landed");
    assert!(
        resolved,
        "SECURITY-POSTURE.md must reference the G15-B drift-detector \
         test pin path (`algorithm_b_drift_detector`) or assert the \
         drift-detector landing"
    );
}
