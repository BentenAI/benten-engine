//! Phase 2b G11-2b — `docs/PAPER-PROTOTYPE-REVALIDATION.md` presence
//! + structural shape pin.
//!
//! Pairs with `crates/benten-eval/tests/sandbox_rate_full_revalidation_g11_2b.rs`
//! (which parses the SANDBOX-rate line and asserts ≤ 30%). This test
//! lives at the engine layer because the doc references engine-side
//! plumbing (manifest-by-name resolution, AttributionFrame
//! propagation) that is not crate-local to benten-eval.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

#[test]
fn paper_prototype_revalidation_doc_present() {
    let root = workspace_root();
    let doc_path = root.join("docs/PAPER-PROTOTYPE-REVALIDATION.md");
    let body = std::fs::read_to_string(&doc_path).unwrap_or_else(|e| {
        panic!(
            "docs/PAPER-PROTOTYPE-REVALIDATION.md MUST exist at \
             Phase-2b close ({}); error: {}. G11-2b-A owns this \
             file per plan §3.",
            doc_path.display(),
            e
        );
    });

    // Structural pins — the doc MUST carry these phrases for the
    // companion gate test (`sandbox_rate_full_revalidation_g11_2b`)
    // to find what it parses.
    assert!(
        body.to_ascii_lowercase().contains("sandbox rate:"),
        "PAPER-PROTOTYPE-REVALIDATION.md MUST carry a parseable \
         `SANDBOX rate: NN.N%` line for the companion full-revalidation \
         gate test."
    );
    assert!(
        body.contains("12-primitive") || body.contains("12 primitive"),
        "PAPER-PROTOTYPE-REVALIDATION.md MUST reference the \
         12-primitive vocabulary (READ, WRITE, TRANSFORM, BRANCH, \
         ITERATE, WAIT, CALL, RESPOND, EMIT, SANDBOX, SUBSCRIBE, \
         STREAM)."
    );
    assert!(
        body.contains("30%"),
        "PAPER-PROTOTYPE-REVALIDATION.md MUST reference the 30% \
         exit-criterion gate (plan §1 exit-criterion #1)."
    );
}
