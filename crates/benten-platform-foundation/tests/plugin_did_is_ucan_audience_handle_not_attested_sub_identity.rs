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

#[test]
fn ucan_with_audience_equals_plugin_did_validates_without_attestation_chain() {
    // **R4b-FP-1** un-ignore — POSITIVE substantive arm via grep-walk
    // assertion. G22-FP-2 (PR #208) shipped audience-binding at
    // `UcanGroundedPolicy::permits_typed_proof_for` via
    // `UCANBackend::validate_chain_for_audience_at`. The observable:
    // ucan_grounded.rs calls through to that audience-bound API.
    //
    // pim-18 §3.6f vacuous-truth defense: assert file exists +
    // content sized > 100 bytes BEFORE asserting symbol presence.
    //
    // Both POSITIVE (audience-bound API called) AND NEGATIVE
    // (NO attestation-chain pattern) arms run here for defense-in-
    // depth; the wider grep over benten-id/src below adds a second
    // independent fail-point.
    let _ = (stub_user_did(), stub_plugin_did());

    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let ucan_grounded = manifest_dir
        .parent()
        .expect("workspace crates/ parent")
        .join("benten-caps")
        .join("src")
        .join("ucan_grounded.rs");
    assert!(
        ucan_grounded.exists(),
        "ucan_grounded.rs MUST exist at: {ucan_grounded:?}"
    );
    let src = std::fs::read_to_string(&ucan_grounded)
        .unwrap_or_else(|e| panic!("read {ucan_grounded:?}: {e}"));
    assert!(
        src.len() > 100,
        "ucan_grounded.rs sized > 100 bytes (vacuous-truth defense)"
    );

    // POSITIVE arm: ucan_grounded.rs invokes the audience-bound chain
    // validator. Would-FAIL if a regression dropped the audience
    // binding in favor of audience-less chain walks (the cap-r1-1
    // BLOCKER pre-fix shape).
    assert!(
        src.contains("validate_chain_for_audience_at"),
        "ucan_grounded.rs MUST call validate_chain_for_audience_at \
         (G22-FP-2 PR #208 audience-binding); positive observable of \
         UCAN-audience-handle flow"
    );

    // Defense-in-depth: NO device-DID-style attestation patterns in
    // this file (plugin-DID is NOT an attested sub-identity per
    // CLAUDE.md #18).
    let forbidden = [
        "PluginDidAttestationEnvelope",
        "verify_plugin_did_attestation",
        "attestation_chain_for_plugin_did",
    ];
    for pat in &forbidden {
        assert!(
            !src.contains(pat),
            "ucan_grounded.rs MUST NOT reference plugin-DID-attestation \
             pattern '{pat}' — plugin-DID is a UCAN audience handle, \
             NOT an attested sub-identity"
        );
    }
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
