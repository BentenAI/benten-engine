//! R3-E RED-PHASE pin for G20-B FINAL Phase-3-close compromise table
//! (wave-8b).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.8 G20-B):
//!
//! - `tests/security_posture_phase_3_close_compromise_table_present`
//!
//! ## Ownership
//!
//! Per r2-test-landscape §13 ambiguous-ownership pre-emption: R3-E owns
//! the G20-B FINAL pin asserting the docs-sweep retensed every closed
//! compromise. The per-compromise individual closure pins are owned by
//! the wave that closes them (R3-A #12, R3-B #17/18/21/2/10, R3-C #11,
//! R3-D #16/19/20) in `security_posture_compromises.rs`.

#![allow(clippy::unwrap_used)]

/// Extract the markdown section for `### Compromise #N — ...` up to (but not
/// including) the next `### ` or `## ` heading.
fn extract_compromise_section(posture: &str, n: u32) -> &str {
    let needle = format!("### Compromise #{n} ");
    let Some(start) = posture.find(&needle) else {
        panic!("SECURITY-POSTURE.md must contain `### Compromise #{n}` heading at G20-B close");
    };
    let tail = &posture[start..];
    // Find the next sibling heading. Sibling headings are either another
    // `### Compromise #` or any `## ` (top-level section) after the start.
    let mut end_offset = tail.len();
    let body = &tail[needle.len()..];
    if let Some(next_h3) = body.find("\n### ") {
        end_offset = needle.len() + next_h3;
    }
    if let Some(next_h2) = body.find("\n## ") {
        let candidate = needle.len() + next_h2;
        if candidate < end_offset {
            end_offset = candidate;
        }
    }
    &tail[..end_offset]
}

#[test]
fn security_posture_phase_3_close_compromise_table_present() {
    // G20-B FINAL closure pin (un-ignored at Phase-3 close per
    // r2-test-landscape §2.8 G20-B + dispatch-conventions §3.5b
    // HARDENED post-fix doc-coupling sweep).
    //
    // OBSERVABLE consequence: the canonical compromise narrative
    // reflects the Phase-3 close state. Defends against the doc-coupling
    // failure mode (compromise closed in code, doc never updated).
    let posture_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("SECURITY-POSTURE.md");
    let posture = std::fs::read_to_string(&posture_path).unwrap();

    // Every Phase-3-CLOSED compromise must be marked CLOSED in its
    // section heading + cite a Phase-3 G-N reference (traceability).
    for compromise in [2_u32, 10, 11, 12, 16, 17, 18, 19, 20, 21] {
        let section = extract_compromise_section(&posture, compromise);
        let lc = section.to_lowercase();
        assert!(
            lc.contains("closed"),
            "Compromise #{compromise} must be marked CLOSED at G20-B Phase-3 close \
             (section: {first_line})",
            first_line = section.lines().next().unwrap_or("")
        );
        let cites_g_n = section.contains("G13")
            || section.contains("G14")
            || section.contains("G15")
            || section.contains("G17")
            || section.contains("G18")
            || section.contains("G20")
            || section.contains("Phase 3")
            || section.contains("Phase-3")
            || section.contains("Phase 2b") // #11/#10 partial closure precedents
            || section.contains("Phase-2b");
        assert!(
            cites_g_n,
            "Compromise #{compromise} closure must cite the closing G-N for traceability"
        );
    }

    // Compromise #22 (public-relay metadata leakage) is INTRODUCED at
    // Phase-3 close + DEFERRED to a NAMED future destination per
    // R4-FP/R3-C net-r4-r1-1 + HARD RULE rule-12 clause-b. The entry
    // must EXIST and name a specific destination.
    let section_22 = extract_compromise_section(&posture, 22);
    let lc_22 = section_22.to_lowercase();
    assert!(
        lc_22.contains("public-relay") || lc_22.contains("public relay"),
        "Compromise #22 must narrate public-relay metadata leakage"
    );
    assert!(
        section_22.contains("Phase 7")
            || section_22.contains("Garden")
            || section_22.contains("Phase 9")
            || section_22.contains("hardened-deployment"),
        "Compromise #22 must name a specific deferral destination \
         (Phase 7 Garden-relay-infrastructure OR Phase 9 hardened-deployment posture)"
    );

    // No Phase-3-pending entries remain:
    assert!(
        !posture.contains("Phase-3-pending"),
        "SECURITY-POSTURE.md must have no Phase-3-pending entries at G20-B close"
    );
    assert!(
        !posture.contains("Phase 3 pending"),
        "SECURITY-POSTURE.md must have no `Phase 3 pending` entries at G20-B close"
    );
}
