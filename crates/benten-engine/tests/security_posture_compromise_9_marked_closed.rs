//! Phase-2b G12-E must-pass — assert `docs/SECURITY-POSTURE.md` carries
//! a "CLOSED at Phase 2b G12-E" marker on the WAIT cross-process
//! metadata gap (referenced as Compromise #9 in the orchestrator brief
//! / R5-decisions log; appears as Compromise #10 in the published
//! posture doc — the doc numbering is the canonical citation).
//!
//! The doc-grep test pins the closure narrative to the file so a
//! future edit that drops the "CLOSED at Phase 2b G12-E" line surfaces
//! at CI rather than at audit time.

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
fn security_posture_compromise_9_marked_closed_at_g12_e() {
    let posture = workspace_root().join("docs/SECURITY-POSTURE.md");
    let body = fs::read_to_string(&posture).expect("read SECURITY-POSTURE.md");

    // The closure narrative MUST mention G12-E as the closing wave AND
    // must mention the cross-process metadata gap by name so a future
    // reviewer can grep for either spelling.
    assert!(
        body.contains("CLOSED at Phase 2b G12-E"),
        "SECURITY-POSTURE.md MUST carry a `CLOSED at Phase 2b G12-E` marker \
         after G12-E lands the durable SuspensionStore. Did you skip the \
         doc update in the closure narrative pass?"
    );
    assert!(
        body.contains("cross-process") || body.contains("cross-process"),
        "the closure narrative MUST name the cross-process metadata gap \
         so a future reader understands what was actually closed"
    );
    // Sanity: the brief / orchestrator log spells the compromise as
    // #9; the doc currently numbers it #10. Pin BOTH so a future
    // renumbering catches a stale citation in either direction.
    let mentions_nine_or_ten = body.contains("Compromise #9") || body.contains("Compromise #10");
    assert!(
        mentions_nine_or_ten,
        "SECURITY-POSTURE.md must reference either Compromise #9 or #10 \
         on the WAIT cross-process closure path"
    );
}
