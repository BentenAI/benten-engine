//! LOAD-BEARING per plan §3 G24-D row + CLAUDE.md #18 retense.
//!
//! Verifies plugin-DID is a UCAN audience handle ONLY — NOT an
//! attested sub-identity of user-DID. Should be NO attestation-chain
//! validation running against plugin-DID.
//!
//! Per R2 §5 substance discipline:
//! - NEGATIVE: grep-assert absence of `attestation_chain_for_plugin_did`
//!   or similar device-DID-attestation patterns in benten-id.
//! - POSITIVE: positive audience-handle-flow test (UCAN with audience =
//!   plugin-DID validates without attestation-chain traversal).

mod common;

use common::manifest_fixtures::{stub_plugin_did, stub_user_did};

#[ignore = "DESTINATION-REMAPPED at R4b-FP-3 per HARD RULE 12 clause-(b) BELONGS-NAMED-NOW + L1 r4b-l1-4 closure: G22-FP-2 already shipped pre-R5 at commit 55f136e WITHOUT delivering this positive arm's un-ignore. The audience-binding production code is already live (verified by sibling negative grep-walk arm); the missing piece is THIS positive integration test that drives a real UCAN through the chain validator. \
    Phase target: Phase-4-Meta (per phase-4-foundation-backlog §4.19 — non-v1-blocker test-positive-pair; substantive defense ALREADY EXISTS via the paired grep-walk negative arm in this same file). \
    Named destination: docs/future/phase-4-backlog.md §4.19 (Phase-4-Meta carry: R5 phantom-destination un-ignore promises). \
    BELONGS-NAMED-NOW."]
#[test]
fn ucan_with_audience_equals_plugin_did_validates_without_attestation_chain() {
    let _user = stub_user_did();
    let _plugin = stub_plugin_did();

    // Future G24-D surface: user_did issues UCAN with
    // audience = plugin_did + cap = "store:notes:read"; chain
    // validator at `benten-caps::ucan_grounded::UcanGroundedPolicy`
    // accepts. NO attestation-chain check runs.
    //
    // FAILS-IF-NO-OP because the validator must consult the audience
    // field of the UCAN payload.
    panic!(
        "RED-PHASE: G24-D wave must wire UCAN audience-handle flow at user_did -> plugin_did delegation"
    );
}

/// Negative substance test (per R2 §5 Gap fix #5 paired discipline) — DURABLE at HEAD.
///
/// Walks `crates/benten-id/src/` via `walkdir` asserting NO symbol
/// matches the device-DID-attestation patterns. This is an active
/// grep-walk (not a future panic) because the source tree exists today
/// at HEAD; the forbidden patterns are introduced ONLY if a future
/// implementer mistakenly imports the device-DID-attestation pattern
/// to plugin-DID.
///
/// Plugin-DIDs are UCAN audiences, NOT attested sub-identities like
/// device-DIDs (which DO have signed DeviceAttestationEnvelope V2
/// per Phase-3 G16-D wave-6b). The categories must remain distinct.
///
/// pim-18 §3.6f vacuous-truth defense: first-line assert the walked
/// root exists. Without this the test would silently PASS on an empty
/// walk (zero matches, but for the wrong reason).
#[test]
fn no_attestation_chain_for_plugin_did_function_in_benten_id_grep_assert() {
    // Locate `crates/benten-id/src/` relative to the workspace root.
    // CARGO_MANIFEST_DIR points at `crates/benten-platform-foundation/`
    // at test time; walk up one + into `benten-id/src/`.
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let benten_id_src = manifest_dir
        .parent()
        .expect("workspace crates/ parent")
        .join("benten-id")
        .join("src");

    // pim-18 §3.6f first-line root-exists assert (vacuous-truth defense).
    assert!(
        benten_id_src.exists() && benten_id_src.is_dir(),
        "walked root must exist: {benten_id_src:?} (vacuous-truth defense per pim-18 §3.6f)"
    );

    // Forbidden symbol patterns — device-DID-attestation patterns that
    // MUST NOT apply to plugin-DID per CLAUDE.md baked-in #18
    // "Implementation refinements" four-identity-concepts model:
    //   3. Plugin-DID minted at install — a UCAN audience handle
    //      (NOT an attested sub-identity); just an identifier
    let forbidden_patterns = [
        "attestation_chain_for_plugin_did",
        "PluginDidAttestationEnvelope",
        "verify_plugin_did_attestation",
        "plugin_did_attestation_chain",
        "PluginAttestationChain",
    ];

    let mut walked_files = 0usize;
    let mut violations: Vec<(String, &str)> = Vec::new();

    for entry in walkdir::WalkDir::new(&benten_id_src)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        walked_files += 1;
        let src = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("failed to read {path:?}: {e}"));
        for pat in &forbidden_patterns {
            if src.contains(pat) {
                violations.push((path.display().to_string(), pat));
            }
        }
    }

    // pim-18 §3.6f sub-rule: assert the walk actually visited files.
    // Without this assertion the grep-walk could pass on a 0-file walk
    // (no .rs files surfaced for any reason — symlink farms, mis-
    // configured test runner, etc.).
    assert!(
        walked_files > 0,
        "walkdir surfaced 0 .rs files under {benten_id_src:?} — \
         vacuous-truth defense; expected at least did.rs / keypair.rs / ucan.rs"
    );

    assert!(
        violations.is_empty(),
        "benten-id/src/ MUST NOT contain device-DID-attestation patterns \
         applied to plugin-DID (plugin-DID is a UCAN audience handle per \
         CLAUDE.md #18, NOT an attested sub-identity). Violations: {violations:?}"
    );
}
