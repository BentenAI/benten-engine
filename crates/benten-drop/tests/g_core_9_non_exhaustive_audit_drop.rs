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
