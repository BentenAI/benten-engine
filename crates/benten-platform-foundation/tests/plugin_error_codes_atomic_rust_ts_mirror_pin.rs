//! G24-D row + §3.5g cross-language rule-mirror.
//!
//! Atomic Rust + TS mirror for 15 new ErrorCodes minted at G24-D
//! (companion-with-canary routing per wave; not bundled at G26-A).
//!
//! Per `dispatch-conventions.md` §3.5g + memory
//! `feedback_pim_cross_language_rule_mirror.md`: when both TS + Rust
//! encode the SAME rule (ErrorCode catalog), edits MUST atomically
//! update both sides + drift-defense surface.
//!
//! Per `feedback_pim_cite_drift_fp1_recurrence.md` §3.5h: mini-review
//! APPROVE doesn't substitute for workspace pre-merge cite-drift gate;
//! this pin reaches into the CATALOG_VARIANT_COUNT drift-defense
//! surface in `crates/benten-errors/tests/stable_shape.rs`.

#[test]
#[ignore = "RED-PHASE: G24-D wave mints 15 new ErrorCodes atomically Rust+TS; un-ignore at G24-D landing"]
fn plugin_manifest_error_codes_present_in_rust_enum() {
    // Future surface: 15 new ErrorCode variants in
    // crates/benten-errors/src/lib.rs:
    //   PluginManifestInvalid,
    //   PluginInstallRecordUserSignatureInvalid,
    //   PluginContentPeerSignatureInvalid,
    //   PluginContentPeerKeyRotated,
    //   PluginAuthorNotTrusted,
    //   PluginInstallConsentRequired,
    //   PluginDelegationOutsideManifestEnvelope,
    //   PluginPrivateNamespaceDelegationForbidden,
    //   PluginContentCidMismatch,
    //   PluginNewVersionAvailable,
    //   PluginHeterogeneityIncompatible,
    //   PluginMetaCompositionCycleRejected,
    //   DeviceAttestationForgedAtPluginShare,
    //   PluginLibraryIndexTamper,
    //   RegistryDiscoveryTimeout (reserved for Phase 4-Meta; 0
    //     production call sites at Phase 4-Foundation).
    //
    // Each variant has matched as_str() / as_static_str() / from_str()
    // arms per crates/benten-errors/src/lib.rs documented protocol.
    //
    // FAILS-IF-NO-OP because the Rust enum must contain each variant
    // by name.
    panic!("RED-PHASE: G24-D wave must mint 15 new ErrorCode variants atomic Rust+TS per §3.5g");
}

#[test]
#[ignore = "RED-PHASE: G24-D wave updates CATALOG_VARIANT_COUNT 118 -> 135 (15 G24-D + others); un-ignore at G24-D landing"]
fn catalog_variant_count_post_g24_d_matches_135() {
    // Per plan §3 G23-A through G24-D summary: CATALOG_VARIANT_COUNT
    // moves 118 -> 135 across Phase 4-Foundation waves. G24-D
    // contributes 15 new codes.
    //
    // The actual assertion lives in
    // crates/benten-errors/tests/stable_shape.rs::
    // catalog_variant_count_matches_enum.
    //
    // This test serves as a sibling-pin reminding G24-D implementer
    // that the count update is part of the atomic Rust+TS landing.
    panic!("RED-PHASE: G24-D wave must update CATALOG_VARIANT_COUNT to reflect 15 new variants");
}

#[test]
#[ignore = "RED-PHASE: G24-D wave updates TS ErrorCode catalog mirror; un-ignore at G24-D landing"]
fn plugin_manifest_error_codes_present_in_ts_catalog_mirror() {
    // Future surface: the TS-side ErrorCode catalog mirror at
    // bindings/napi/ (or wherever the TS catalog lives) MUST contain
    // string-form entries for each of the 15 new codes.
    //
    // The drift-defense surface (parity test between Rust + TS)
    // catches gaps. Per pim-cross-language-rule-mirror, this is
    // §3.5g atomic-update territory.
    panic!("RED-PHASE: G24-D wave must mirror 15 new ErrorCodes in TS catalog per §3.5g");
}
