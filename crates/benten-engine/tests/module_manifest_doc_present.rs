//! Phase 2b G10-B — `docs/MODULE-MANIFEST.md` presence + load-bearing-
//! anchor drift detector.
//!
//! Every section the G10-B brief calls out as documented (D9 canonical
//! DAG-CBOR, D16 dual-CID + summary error shape, signature-reservation
//! forward-compat, Compromise #N+5, Compromise #N+8, install +
//! uninstall + idempotence + cap-retraction multi-manifest overlap)
//! MUST appear by name in the doc. If a future contributor drops one of
//! the load-bearing anchors, this test surfaces the regression
//! immediately rather than letting the doc drift away from the code.
//!
//! Owned by R5 G10-B.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn read_doc() -> String {
    let path = workspace_root().join("docs").join("MODULE-MANIFEST.md");
    assert!(
        path.exists(),
        "docs/MODULE-MANIFEST.md must exist (G10-B doc-drift item) — tried {}",
        path.display()
    );
    std::fs::read_to_string(&path).expect("read docs/MODULE-MANIFEST.md")
}

#[test]
fn module_manifest_doc_present() {
    let body = read_doc();
    // The doc must be non-trivial (>1KB) — a stub file would defeat
    // the documentation contract.
    assert!(
        body.len() > 1024,
        "docs/MODULE-MANIFEST.md must be substantive (>1KB); got {} bytes",
        body.len()
    );
}

#[test]
fn module_manifest_doc_carries_load_bearing_anchors() {
    let body = read_doc();
    let required_anchors: &[&str] = &[
        // D9 canonical-bytes (the load-bearing decision)
        "DAG-CBOR",
        "D9-RESOLVED",
        // D16 install-CID-pin
        "D16-RESOLVED-FURTHER",
        "expected_cid",
        "install_module",
        "uninstall_module",
        // D16 dual-CID diff shape
        "computed_cid",
        "manifest summary",
        // Compromises documented
        "Compromise #N+5",
        "Compromise #N+8",
        // Forward-compat reservation
        "skip_serializing_if",
        "Phase 3",
        // Cap-retraction multi-manifest overlap rule
        "multi-manifest overlap",
        // Idempotence
        "Idempotence",
        // System-zone storage
        "system:ModuleManifest",
        "system:ModuleManifestRevocation",
        // Error code anchors
        "E_MODULE_MANIFEST_CID_MISMATCH",
        "E_MODULE_MIGRATIONS_REQUIRE_PERSISTENCE",
    ];
    for anchor in required_anchors {
        assert!(
            body.contains(anchor),
            "docs/MODULE-MANIFEST.md is missing load-bearing anchor {anchor:?} — \
             G10-B contract requires every D9/D16/Compromise reference to remain \
             discoverable to operators reading the doc",
        );
    }
}
