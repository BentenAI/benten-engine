//! G18-A wave-5a un-ignored source-cite pins for the cross-browser-
//! determinism CI workflow (D-PHASE-3-7 + br-r1-4 MAJOR + br-r1-10 MINOR
//! + br-r4-r1-5 7 distinct engine-determinism failure-surfaces).
//!
//! Pin sources (per r2-test-landscape §2.6 G18-A):
//!
//! Per-browser-engine cells (D-PHASE-3-7):
//! - `tests/cross_browser_determinism_chromium_canonical_bytes_match`
//! - `tests/cross_browser_determinism_gecko_canonical_bytes_match`
//! - `tests/cross_browser_determinism_webkit_canonical_bytes_match`
//!
//! CID equivalence + flake budget (br-r1-4 / br-r1-10):
//! - `tests/cross_browser_determinism_cid_pin_equivalence_across_three_browsers`
//! - `tests/cross_browser_determinism_flake_budget_retry_policy_observed`
//!
//! 7 engine-determinism failure-surfaces (br-r4-r1-5):
//! - `tests/cross_browser_canonical_bytes_pin_for_node_envelope`
//! - `tests/cross_browser_canonical_bytes_pin_for_handler_version_chain`
//! - `tests/cross_browser_canonical_bytes_pin_for_attribution_frame_with_device_did`
//! - `tests/cross_browser_cid_pin_for_canonical_fixture_corpus`
//! - `tests/cross_browser_blake3_byte_identity`
//! - `tests/cross_browser_ed25519_signature_byte_identity`
//! - `tests/cross_browser_floating_point_canonicalization_under_dsl_eval`
//!
//! ## Workflow-pin shape (now LIVE)
//!
//! These pins are Rust-side anchors for the
//! `.github/workflows/cross-browser-determinism.yml` Playwright matrix
//! workflow. Per pim-3 §3.9 (R2 lens-menu correctness coverage) +
//! pim-1 §3.5b HARDENED (doc-coupling): if the YAML workflow is
//! later renamed, relocated, or has its matrix cells changed, these
//! Rust-side pins go RED.

#![allow(clippy::unwrap_used, dead_code)]

const CROSS_BROWSER_WORKFLOW_PATH: &str = ".github/workflows/cross-browser-determinism.yml";

fn workflow_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(CROSS_BROWSER_WORKFLOW_PATH)
}

fn workflow() -> String {
    std::fs::read_to_string(workflow_path()).unwrap()
}

#[test]
fn cross_browser_determinism_chromium_canonical_bytes_match() {
    let w = workflow();
    assert!(
        w.contains("chromium") || w.contains("Chromium"),
        "cross-browser-determinism.yml must declare a Chromium matrix cell per D-PHASE-3-7"
    );
    assert!(
        w.contains("canonical_bytes")
            || w.contains("canonical-bytes")
            || w.contains("dag-cbor")
            || w.contains("canonical bytes"),
        "Chromium cell must drive a canonical-bytes determinism assertion per br-r1-4 WHAT FAILS"
    );
}

#[test]
fn cross_browser_determinism_gecko_canonical_bytes_match() {
    let w = workflow();
    assert!(
        w.contains("firefox")
            || w.contains("gecko")
            || w.contains("Firefox")
            || w.contains("Gecko"),
        "cross-browser-determinism.yml must declare a Gecko/Firefox matrix cell per D-PHASE-3-7"
    );
}

#[test]
fn cross_browser_determinism_webkit_canonical_bytes_match() {
    let w = workflow();
    assert!(
        w.contains("webkit") || w.contains("WebKit"),
        "cross-browser-determinism.yml must declare a WebKit matrix cell per D-PHASE-3-7"
    );
}

#[test]
fn cross_browser_determinism_cid_pin_equivalence_across_three_browsers() {
    let w = workflow();
    assert!(
        w.contains("equivalence") || w.contains("cid_pin") || w.contains("CID"),
        "cross-browser-determinism.yml must include a cross-browser CID-equivalence reduce step per br-r1-4"
    );
    // Stronger: assert the explicit reduce-step structure.
    assert!(
        w.contains("cid-equivalence-reduce")
            || w.contains("cid_equivalence")
            || w.contains("compare reduce"),
        "cross-browser-determinism.yml must declare an explicit reduce step that compares per-browser CIDs"
    );
}

#[test]
fn cross_browser_determinism_flake_budget_retry_policy_observed() {
    let w = workflow();
    assert!(
        w.contains("retry") || w.contains("PLAYWRIGHT_BROWSER_LAUNCH_RETRIES"),
        "cross-browser-determinism.yml must declare a retry policy per br-r1-10 (1 retry on browser-launch failure)"
    );
    // Documentation surface (the policy is published per §3.5b doc-coupling).
    let posture = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("docs")
            .join("SECURITY-POSTURE.md"),
    )
    .unwrap();
    assert!(
        posture.contains("Compromise #20") || posture.contains("Compromise 20"),
        "SECURITY-POSTURE.md must document Compromise #20 per §3.5b doc-coupling"
    );
}

// ============================================================================
// br-r4-r1-5 — 7 distinct engine-determinism failure-surface pins
// ============================================================================

#[test]
fn cross_browser_canonical_bytes_pin_for_node_envelope() {
    let w = workflow();
    assert!(
        w.contains("node_envelope") || w.contains("node-envelope") || w.contains("Node envelope"),
        "cross-browser-determinism.yml MUST drive a Node-envelope canonical-bytes assertion per br-r4-r1-5 #1"
    );
}

#[test]
fn cross_browser_canonical_bytes_pin_for_handler_version_chain() {
    let w = workflow();
    assert!(
        w.contains("handler_version_chain")
            || w.contains("handler-version-chain")
            || w.contains("HandlerVersionChain"),
        "cross-browser-determinism.yml MUST drive a handler-version-chain canonical-bytes assertion per br-r4-r1-5 #2"
    );
}

#[test]
fn cross_browser_canonical_bytes_pin_for_attribution_frame_with_device_did() {
    let w = workflow();
    assert!(
        (w.contains("attribution_frame")
            || w.contains("attribution-frame")
            || w.contains("AttributionFrame"))
            && (w.contains("device_did") || w.contains("device-did") || w.contains("DID")),
        "cross-browser-determinism.yml MUST drive AttributionFrame-with-device-DID canonical-bytes assertion per br-r4-r1-5 #3"
    );
}

#[test]
fn cross_browser_cid_pin_for_canonical_fixture_corpus() {
    let w = workflow();
    assert!(
        w.contains("canonical_fixture") || w.contains("canonical-fixture") || w.contains("bafyr4i"),
        "cross-browser-determinism.yml MUST drive canonical-fixture-corpus CID-pin assertion per br-r4-r1-5 #4"
    );
}

#[test]
fn cross_browser_blake3_byte_identity() {
    let w = workflow();
    assert!(
        w.contains("blake3") || w.contains("BLAKE3"),
        "cross-browser-determinism.yml MUST drive a BLAKE3-byte-identity assertion per br-r4-r1-5 #5"
    );
}

#[test]
fn cross_browser_ed25519_signature_byte_identity() {
    let w = workflow();
    assert!(
        w.contains("ed25519") || w.contains("Ed25519"),
        "cross-browser-determinism.yml MUST drive an Ed25519-signature-byte-identity assertion per br-r4-r1-5 #6"
    );
}

#[test]
fn cross_browser_floating_point_canonicalization_under_dsl_eval() {
    let w = workflow();
    assert!(
        w.contains("floating_point")
            || w.contains("floating-point")
            || w.contains("f64")
            || w.contains("IEEE"),
        "cross-browser-determinism.yml MUST drive floating-point-canonicalization assertion per br-r4-r1-5 #7"
    );
}
