//! R6 R1 fix-pass (Bundle L1, L1-r6r1-MAJ-2 closure) — benten-drop
//! `#[non_exhaustive]` audit arm-coverage pins.
//!
//! Closes L1-r6r1-MAJ-2 (benten-drop enums explicitly named APPLY in
//! V1-FROZEN-INTERFACE.md item 11 lack `#[non_exhaustive]` — phase-wide
//! freeze-contract drift on the Drop crypto consumer side).
//!
//! Companion to `crates/benten-engine/tests/g_core_9_non_exhaustive_audit.rs`
//! — the engine-side audit covers types the engine crate links against;
//! the drop crate's audit lives here because benten-engine does NOT
//! depend on benten-drop (deliberate dependency-graph stratification).
//!
//! Carve-out preserved: `DropContentMode` is INTENTIONALLY exhaustive-by-design
//! per V1-FROZEN-INTERFACE.md item 15(c) pattern + the
//! `tf3f_drop_content_mode_no_inline_tiny_arm` structural pin. NOT covered here.

#[test]
fn drop_bundle_version_audit_arm_coverage() {
    use benten_drop::bundle::DropBundleVersion;
    fn audit(v: DropBundleVersion) -> &'static str {
        match v {
            DropBundleVersion::V1 => "V1",
            DropBundleVersion::Synthetic(_) => "Synthetic",
            _ => "Unknown",
        }
    }
    assert_eq!(audit(DropBundleVersion::V1), "V1");
    assert_eq!(audit(DropBundleVersion::Synthetic(42)), "Synthetic");
}

#[test]
fn drop_envelope_sig_error_audit_arm_coverage() {
    use benten_drop::envelope_sig::EnvelopeSigError;
    fn audit(e: &EnvelopeSigError) -> &'static str {
        match e {
            EnvelopeSigError::VerifyingKeyMalformed(_) => "VerifyingKeyMalformed",
            EnvelopeSigError::SignatureMalformed(_) => "SignatureMalformed",
            EnvelopeSigError::VerifyFailed(_) => "VerifyFailed",
            _ => "Unknown",
        }
    }
    assert_eq!(
        audit(&EnvelopeSigError::VerifyFailed("test".to_string())),
        "VerifyFailed"
    );
}

// F-full R6-R2 F-03 (Shard C): the Layer-C error enums are SemVer-frozen
// `#[non_exhaustive]` so a future failure mode lands additively. The `_` arm
// only stays reachable while the attribute is present — REMOVING
// `#[non_exhaustive]` would make the catch-all a `unreachable_patterns`
// warning that `-D warnings` turns into a build break (§11 HALT-AND-SURFACE).

#[test]
fn layer_c_error_audit_arm_coverage_non_exhaustive() {
    use benten_drop::layer_c::LayerCError;
    fn audit(e: &LayerCError) -> &'static str {
        match e {
            LayerCError::AeadAuthenticationFailed => "AeadAuthenticationFailed",
            LayerCError::InnerSenderDidForged => "InnerSenderDidForged",
            LayerCError::SenderOriginAuthFailed => "SenderOriginAuthFailed",
            LayerCError::UnsupportedCodepoint(_) => "UnsupportedCodepoint",
            LayerCError::StanzaCountMismatch { .. } => "StanzaCountMismatch",
            _ => "Unknown",
        }
    }
    assert_eq!(
        audit(&LayerCError::AeadAuthenticationFailed),
        "AeadAuthenticationFailed"
    );
}

#[test]
fn admit_error_audit_arm_coverage_non_exhaustive() {
    use benten_drop::layer_c::abuse_control::AdmitError;
    fn audit(e: &AdmitError) -> &'static str {
        match e {
            AdmitError::MissingDeliveryToken => "MissingDeliveryToken",
            AdmitError::TokenExpiredOrNotYetValid => "TokenExpiredOrNotYetValid",
            AdmitError::RateLimitExceeded => "RateLimitExceeded",
            AdmitError::TokenBindingMismatch => "TokenBindingMismatch",
            _ => "Unknown",
        }
    }
    assert_eq!(
        audit(&AdmitError::MissingDeliveryToken),
        "MissingDeliveryToken"
    );
}

#[test]
fn group_error_audit_arm_coverage_non_exhaustive() {
    use benten_drop::layer_c::group_posture::GroupError;
    fn audit(e: &GroupError) -> &'static str {
        match e {
            GroupError::AeadAuthenticationFailed => "AeadAuthenticationFailed",
            GroupError::SenderOriginAuthFailed => "SenderOriginAuthFailed",
            GroupError::WrongGroupCodepoint { .. } => "WrongGroupCodepoint",
            GroupError::StanzaCountMismatch { .. } => "StanzaCountMismatch",
            _ => "Unknown",
        }
    }
    assert_eq!(
        audit(&GroupError::AeadAuthenticationFailed),
        "AeadAuthenticationFailed"
    );
}

#[test]
fn drop_bundle_error_audit_arm_coverage_non_exhaustive() {
    use benten_drop::bundle::DropBundleError;
    fn audit(e: &DropBundleError) -> &'static str {
        match e {
            DropBundleError::EnvelopeSignatureInvalid { .. } => "EnvelopeSignatureInvalid",
            DropBundleError::UnsupportedDropVersion { .. } => "UnsupportedDropVersion",
            DropBundleError::UnsupportedDropMode { .. } => "UnsupportedDropMode",
            DropBundleError::PerNodeAeadAuthenticationFailed { .. } => {
                "PerNodeAeadAuthenticationFailed"
            }
            DropBundleError::PerNodeSignatureInvalid { .. } => "PerNodeSignatureInvalid",
            DropBundleError::AuthorizationGrantFailed(_) => "AuthorizationGrantFailed",
            DropBundleError::CodecError(_) => "CodecError",
            _ => "Unknown",
        }
    }
    assert_eq!(
        audit(&DropBundleError::CodecError("test".to_string())),
        "CodecError"
    );
}
