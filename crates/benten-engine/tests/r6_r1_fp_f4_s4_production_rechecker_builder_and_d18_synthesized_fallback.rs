//! R6 R1 FP-F4 §S4 production-arm test pin — `ProductionManifestEnvelopeRechecker`
//! + `ProductionEngineBuilder` + Row D-18 synthesized-fallback hardening.
//!
//! Closes Rows D-4 + D-18.
//!
//! ## Test shape
//!
//! 1. `ProductionEngineBuilder::new().open(":memory:")` constructs an
//!    engine with the substantive rechecker installed.
//! 2. The substantive rechecker rejects synthesized-fallback
//!    `node-id:N` peer-DIDs with `UnresolvedDeny` (Row D-18 closure).
//! 3. The `is_synthesized_node_id` helper has positive + negative
//!    inputs covered.
//! 4. The Noop rechecker (default) STILL admits synthesized DIDs
//!    via `NotApplicable` — the hardening only fires when the
//!    substantive rechecker is installed (per Row D-18's
//!    substantive-rechecker-installed-detection-couple-not-naive-blanket
//!    clause).

use benten_engine::manifest_envelope_recheck::{
    ManifestEnvelopeRecheckOutcome, ManifestEnvelopeRechecker, NoopManifestEnvelopeRechecker,
    is_synthesized_node_id,
};
use benten_engine::production_engine_builder::ProductionEngineBuilder;
use benten_engine::production_manifest_envelope_rechecker::ProductionManifestEnvelopeRechecker;

/// **§S4 arm 1 — ProductionEngineBuilder constructs.**
#[test]
fn production_engine_builder_open_in_memory() {
    let _engine = ProductionEngineBuilder::new()
        .open(":memory:")
        .expect("in-memory engine opens via ProductionEngineBuilder");
}

/// **§S4 arm 2 — ProductionManifestEnvelopeRechecker rejects
/// synthesized DIDs.**
#[test]
fn production_rechecker_rejects_synthesized_node_id_with_unresolved_deny() {
    let rechecker = ProductionManifestEnvelopeRechecker::new();
    let outcome = rechecker.recheck_row("node-id:7", "any-zone", "any-key");
    assert_eq!(outcome, ManifestEnvelopeRecheckOutcome::UnresolvedDeny);
}

/// **§S4 arm 3 — `is_synthesized_node_id` positive + negative.**
#[test]
fn is_synthesized_node_id_positive_and_negative_inputs() {
    // Positive: synthesized fallback.
    assert!(is_synthesized_node_id("node-id:0"));
    assert!(is_synthesized_node_id("node-id:99"));
    assert!(is_synthesized_node_id("node-id:abc"));

    // Negative: real DIDs.
    assert!(!is_synthesized_node_id("did:key:z6Mk..."));
    assert!(!is_synthesized_node_id(""));
    assert!(!is_synthesized_node_id("node-x:1"));
    assert!(!is_synthesized_node_id("did:key:znode-id:1"));
}

/// **§S4 arm 4 — Noop default admits synthesized DIDs.** Per Row D-18,
/// the always-mounted Noop continues to admit (NotApplicable); only
/// the substantive `ProductionManifestEnvelopeRechecker` triggers the
/// fail-CLOSED. This prevents over-fire on default-Noop test fixtures.
#[test]
fn noop_rechecker_admits_synthesized_node_id_via_not_applicable() {
    let noop = NoopManifestEnvelopeRechecker;
    let outcome = noop.recheck_row("node-id:42", "any-zone", "any-key");
    assert_eq!(outcome, ManifestEnvelopeRecheckOutcome::NotApplicable);
}

/// **§S4 arm 5 — ProductionRechecker admits resolvable DIDs.** At
/// v1-beta the resolvable arm is NotApplicable (preserves Phase-3
/// baseline for resolvable peers); the full chain walk is the
/// G-COMP-1 deliverable per Row D-4 narrative.
#[test]
fn production_rechecker_resolvable_did_admits_via_not_applicable_v1_beta_posture() {
    let rechecker = ProductionManifestEnvelopeRechecker::new();
    let outcome = rechecker.recheck_row("did:key:z6Mk-resolvable", "any-zone", "any-key");
    assert_eq!(outcome, ManifestEnvelopeRecheckOutcome::NotApplicable);
}
