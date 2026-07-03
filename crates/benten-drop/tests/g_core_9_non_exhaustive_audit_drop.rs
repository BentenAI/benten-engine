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

// R10-council F-02/F-11: `EncryptedEnvelope` is the Inv-16 codepoint-dispatched
// envelope — SemVer-frozen `#[non_exhaustive]` so a future dispatched shape lands
// additively. The `_` arm stays reachable ONLY while the attribute is present;
// removing it turns the catch-all into `unreachable_patterns` → `-D warnings`
// build break (§11 HALT-AND-SURFACE).
#[test]
fn encrypted_envelope_audit_arm_coverage_non_exhaustive() {
    use benten_drop::layer_c::{BindingContext, EncryptedEnvelope};
    fn audit(e: &EncryptedEnvelope) -> &'static str {
        match e {
            EncryptedEnvelope::HpkeBase { .. } => "HpkeBase",
            EncryptedEnvelope::HpkeMultiBase { .. } => "HpkeMultiBase",
            _ => "Unknown",
        }
    }
    let env = EncryptedEnvelope::HpkeBase {
        format_version: 2,
        binding: BindingContext::DropSealedSender {
            aad_version: 0x01,
            codepoint: 0x6510,
            audience_did: b"did:key:zAudience".to_vec(),
            body_cid: benten_drop::layer_c::self_describing_cid(&[0u8; 32]),
            recipient_key_generation: 0,
        },
        enc: vec![0u8; 4],
        ciphertext: vec![0u8; 4],
    };
    assert_eq!(audit(&env), "HpkeBase");
}

// R10-council F-02/F-11 CARVE-OUT: the drop-side `BindingContext` is
// INTENTIONALLY exhaustive-by-design (wire-keying — one codepoint per variant:
// `0x6500`/`0x6510`), the same posture as `MembershipSetKind`/`RoleId`. It has
// NO `#[non_exhaustive]` and NO `_` arm; this exhaustive match (both variants,
// no catch-all) HALT-AND-SURFACEs if a variant is ever added without a wire
// decision — the correct posture for a closed frozen wire contract.
#[test]
fn binding_context_audit_exhaustive_by_design_no_catch_all() {
    use benten_drop::layer_c::BindingContext;
    fn audit(b: &BindingContext) -> &'static str {
        match b {
            BindingContext::DropPlaintextSender { .. } => "DropPlaintextSender",
            BindingContext::DropSealedSender { .. } => "DropSealedSender",
        }
    }
    let b = BindingContext::DropSealedSender {
        aad_version: 0x01,
        codepoint: 0x6510,
        audience_did: b"did:key:zAudience".to_vec(),
        body_cid: benten_drop::layer_c::self_describing_cid(&[0u8; 32]),
        recipient_key_generation: 0,
    };
    assert_eq!(audit(&b), "DropSealedSender");
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
