//! Phase 2b G7-C — `docs/SANDBOX-LIMITS.md` presence + content pins.
//!
//! Pin source: plan §3 G7-C
//! (`tests/sandbox_limits_doc_present` — D21 + D24 + wsa-14 UX text documented).
//!
//! `docs/SANDBOX-LIMITS.md` is the operator-facing source of truth for
//! the four SANDBOX enforcement axes (memory / wallclock / fuel /
//! output), the per-call instance lifecycle (D17), the wallclock
//! defaults (D24), the severity priority (D21), and the wsa-14 UX text.
//! These tests assert the doc EXISTS at the pinned path and that the
//! load-bearing sections are present.
//!
//! The doc is owned by G7-C; G7-A may add bench-table updates; G10-B
//! may extend the doc's cross-references; the section presence pinned
//! here MUST hold across those edits.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn read_sandbox_limits_md() -> String {
    let path = workspace_root().join("docs").join("SANDBOX-LIMITS.md");
    std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "docs/SANDBOX-LIMITS.md must exist at workspace root (G7-C owned). \
             Read failed: {e}. Path: {path:?}"
        )
    })
}

/// `sandbox_limits_doc_present` — plan §3 G7-C row.
///
/// The doc must exist and document D21 (severity priority) + D24
/// (wallclock defaults) + wsa-14 (`E_SANDBOX_UNAVAILABLE_ON_WASM` UX
/// text) per the plan-§3-G7-C requirement.
#[test]
fn sandbox_limits_doc_present() {
    let doc = read_sandbox_limits_md();
    let lower = doc.to_ascii_lowercase();

    // D21 — severity priority must be documented.
    assert!(
        lower.contains("severity priority")
            || lower.contains("severity ordering")
            || (lower.contains("severity") && lower.contains("d21")),
        "docs/SANDBOX-LIMITS.md MUST document D21 severity priority \
         (MEMORY > WALLCLOCK > FUEL > OUTPUT). Search-key: 'severity priority'."
    );
    // The four axes must each be named.
    for axis in ["memory", "wallclock", "fuel", "output"] {
        assert!(
            lower.contains(axis),
            "docs/SANDBOX-LIMITS.md MUST name the '{axis}' enforcement axis."
        );
    }

    // D24 — wallclock defaults must be documented.
    assert!(
        lower.contains("d24") || lower.contains("30,000") || lower.contains("30000"),
        "docs/SANDBOX-LIMITS.md MUST document D24 wallclock default (30,000 ms)."
    );

    // D17 — per-call instance lifecycle architecture explainer.
    assert!(
        lower.contains("per-call") && lower.contains("instance"),
        "docs/SANDBOX-LIMITS.md MUST document D17 per-call instance lifecycle \
         (no pool; fresh wasmtime::Instance per call)."
    );

    // wsa-14 — UX text must be present verbatim.
    let wsa_14_text = "SANDBOX is unavailable in browser/wasm32 builds. Author handlers in browser \
                       context for execution against a Node-WASI peer";
    assert!(
        doc.contains(wsa_14_text),
        "docs/SANDBOX-LIMITS.md MUST contain the wsa-14 UX text verbatim. \
         Expected substring: {wsa_14_text:?}"
    );
}

/// `sandbox_limits_doc_names_compromise_4_closure` — cross-reference pin.
///
/// `docs/SANDBOX-LIMITS.md` should reference the SECURITY-POSTURE.md
/// Compromise #4 closure so operators reading either document can find
/// the other.
#[test]
fn sandbox_limits_doc_names_compromise_4_closure() {
    let doc = read_sandbox_limits_md();
    let lower = doc.to_ascii_lowercase();
    assert!(
        lower.contains("compromise #4") || lower.contains("security-posture"),
        "docs/SANDBOX-LIMITS.md SHOULD cross-reference docs/SECURITY-POSTURE.md \
         Compromise #4 closure so operators can find the other doc."
    );
}
