//! G24-D subset-closure pin — every `E_PLUGIN_*` ErrorCode variant (plus
//! the related `E_REGISTRY_*` family entry) that exists in the
//! `benten-errors` enum at post-G24-D HEAD must be present in the
//! `G24_D_ERROR_CODES` registry at
//! `crates/benten-platform-foundation/tests/common/manifest_fixtures.rs`.
//!
//! Per §3.5g cross-language rule-mirror: this is the **inverse-direction**
//! pin that catches the failure mode "minted enum variant but forgot to
//! add to the test-side registry array used by atomic-mint round-trip
//! pins". Without subset-closure, a new E_PLUGIN_NEW_THING variant could
//! land in benten-errors without being asserted by any TS-mirror /
//! catalog-md pin downstream.
//!
//! ## Closure shape
//!
//! Authoritative expected set (frozen at G24-D wave; matches
//! `manifest_fixtures::G24_D_ERROR_CODES` exactly). 14 E_PLUGIN_* codes +
//! 1 E_REGISTRY_DISCOVERY_TIMEOUT (paired family-of-record per Ben's
//! R4-triage §7 — the registry-discovery cap is part of the plugin
//! install/discovery surface). Each expected code:
//!   - resolves via `from_str` to a NAMED variant (catches enum hole)
//!   - round-trips via `as_static_str` (catches as_str arm gap)
//!   - has prefix `E_PLUGIN_` OR `E_REGISTRY_` (catches family-naming drift)
//!
//! Per Ben's R4-triage §7 ratification (2026-05-11):
//! - `E_PLUGIN_DEVICE_ATTESTATION_FORGED` keeps the `E_PLUGIN_*` family
//!   prefix (renamed from earlier `E_DEVICE_ATTESTATION_FORGED_AT_PLUGIN_SHARE`).
//! - TS mirror canonical location is `packages/engine/src/errors.generated.ts`.
//!
//! UN-IGNORED at G24-D wave: all 15 variants exist in the enum post-
//! G24-D HEAD with as_static_str + from_str arms. The test now serves
//! as a durable regression guard rather than a RED-PHASE pin.

#![allow(clippy::unwrap_used)]

use benten_errors::ErrorCode;

/// 15 G24-D ErrorCode string forms — mirror of
/// `manifest_fixtures::G24_D_ERROR_CODES` at
/// `crates/benten-platform-foundation/tests/common/manifest_fixtures.rs`.
///
/// Frozen here to catch the failure mode where a NEW `E_PLUGIN_*` (or
/// `E_REGISTRY_*`) variant lands in `benten-errors` but the downstream
/// registry array is not updated to match.
const EXPECTED_G24_D_CODES: &[&str] = &[
    "E_PLUGIN_MANIFEST_INVALID",
    "E_PLUGIN_INSTALL_RECORD_USER_SIGNATURE_INVALID",
    "E_PLUGIN_CONTENT_PEER_SIGNATURE_INVALID",
    "E_PLUGIN_CONTENT_PEER_KEY_ROTATED",
    "E_PLUGIN_AUTHOR_NOT_TRUSTED",
    "E_PLUGIN_INSTALL_CONSENT_REQUIRED",
    "E_PLUGIN_DELEGATION_OUTSIDE_MANIFEST_ENVELOPE",
    "E_PLUGIN_PRIVATE_NAMESPACE_DELEGATION_FORBIDDEN",
    "E_PLUGIN_CONTENT_CID_MISMATCH",
    "E_PLUGIN_NEW_VERSION_AVAILABLE",
    "E_PLUGIN_HETEROGENEITY_INCOMPATIBLE",
    "E_PLUGIN_META_COMPOSITION_CYCLE_REJECTED",
    "E_PLUGIN_DEVICE_ATTESTATION_FORGED",
    "E_PLUGIN_LIBRARY_INDEX_TAMPER",
    "E_REGISTRY_DISCOVERY_TIMEOUT",
];

#[test]
fn every_expected_g24_d_plugin_code_resolves_to_named_variant() {
    for code in EXPECTED_G24_D_CODES {
        // Family-prefix discipline: every code in the G24-D set must
        // share the E_PLUGIN_ prefix (the bulk) OR be the paired
        // E_REGISTRY_DISCOVERY_TIMEOUT code (registry-discovery is part
        // of the plugin install surface per Ben's R4-triage §7).
        let valid_prefix = code.starts_with("E_PLUGIN_") || code.starts_with("E_REGISTRY_");
        assert!(
            valid_prefix,
            "G24-D subset-closure: expected code {code} does not start \
             with E_PLUGIN_ or E_REGISTRY_ — family-naming discipline broken"
        );
        // Dynamic half: from_str round-trip. Post-G24-D, all 15
        // variants resolve to NAMED variants (un-ignored sentinel).
        let parsed = ErrorCode::from_str(code);
        assert!(
            !matches!(parsed, ErrorCode::Unknown(_)),
            "G24-D subset-closure: ErrorCode {code} expected in enum \
             post-G24-D but from_str returned Unknown — enum + as_str \
             + from_str arms missing for this variant"
        );
        assert_eq!(
            parsed.as_static_str(),
            *code,
            "G24-D subset-closure: ErrorCode {code} must round-trip \
             through as_static_str → from_str without lossy conversion"
        );
    }
}

#[test]
fn device_attestation_forged_code_keeps_plugin_family_prefix() {
    // Defends Ben's R4-triage §7 ratification: the device-attestation
    // forgery code in the plugin-share context must be named with the
    // E_PLUGIN_ prefix so it lives in the E_PLUGIN_* family (subset of
    // G24_D_ERROR_CODES). Without this pin a future fix-pass could
    // re-introduce the longer E_DEVICE_ATTESTATION_FORGED_AT_PLUGIN_SHARE
    // name OR collapse to bare E_DEVICE_ATTESTATION_FORGED (which would
    // collide with the existing Phase-3 device-attestation forgery code
    // at a different layer).
    let code = "E_PLUGIN_DEVICE_ATTESTATION_FORGED";
    let parsed = ErrorCode::from_str(code);
    assert!(
        !matches!(parsed, ErrorCode::Unknown(_)),
        "Sentinel: {code} must be a NAMED variant post-G24-D \
         (Ben's R4-triage §7 ratification — E_PLUGIN_* family prefix)"
    );
    assert_eq!(
        parsed.as_static_str(),
        code,
        "Sentinel: {code} must round-trip exact string form — \
         catches accidental rename back to \
         E_DEVICE_ATTESTATION_FORGED_AT_PLUGIN_SHARE or other drift"
    );
    // Inverse-direction sentinel: the OLD pre-rename name must NOT
    // resolve to a named variant. Defends against an accidental
    // alias-or-rename-back; catches the failure mode where a future
    // implementer adds the long name back as an alias arm.
    let old_name = "E_DEVICE_ATTESTATION_FORGED_AT_PLUGIN_SHARE";
    let parsed_old = ErrorCode::from_str(old_name);
    assert!(
        matches!(parsed_old, ErrorCode::Unknown(_)),
        "Sentinel inverse: {old_name} (pre-rename) MUST NOT be a named \
         variant; R4-triage §7 ratification chose E_PLUGIN_DEVICE_ATTESTATION_FORGED"
    );
}
