//! Phase 4-Foundation R6-FP-BF wave: closure pin for R6 R1 MAJOR
//! sdr-r6-r1-2 (`requires_schema_authors` declared but zero runtime
//! enforcement).
//!
//! Substantive arm: invoke
//! `validate_schema_author_within_manifest_envelope` against a
//! manifest with a non-empty trust-list + a schema author NOT in the
//! list; assert it returns `E_PLUGIN_AUTHOR_NOT_TRUSTED`. Cover the
//! 4 dispositions: (a) None → Ok; (b) Some(empty) → Ok; (c) Some(list)
//! with author in list → Ok; (d) Some(list) with author NOT in list
//! → Err.
//!
//! WOULD-FAIL-IF-NO-OP per pim-2 §3.6b: removing the trust-list
//! consultation in `validate_schema_author_within_manifest_envelope`
//! (returning Ok unconditionally) flips assertion (d) to a no-op +
//! the un-trusted-author arm no longer fails.

#![allow(clippy::unwrap_used)]

use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_id::did::Did;
use benten_id::keypair::Keypair;
use benten_platform_foundation::plugin_manifest::{
    CapRequirement, PluginManifest, SharesPolicy, SharesPolicyDefault,
    validate_schema_author_within_manifest_envelope,
};

fn synthesize_manifest(requires_schema_authors: Option<Vec<Did>>) -> PluginManifest {
    let kp = Keypair::generate();
    PluginManifest {
        plugin_name: "fixture".to_string(),
        content_cid: Cid::from_blake3_digest([0u8; 32]),
        peer_did: Did::from_public_key(kp.public_key()),
        peer_signature: vec![0u8; 64],
        requires: vec![CapRequirement::new("read:Note.body")],
        shares: SharesPolicy {
            default: SharesPolicyDefault::None,
            rules: None,
        },
        renderer_config: None,
        composes_plugins: None,
        accepts_content: None,
        requires_schema_authors,
        requires_plugin_authors: None,
    }
}

#[test]
fn trust_list_none_accepts_any_schema_author() {
    let manifest = synthesize_manifest(None);
    let any_author = Did::from_public_key(Keypair::generate().public_key());
    let r = validate_schema_author_within_manifest_envelope(&any_author, &manifest);
    assert!(
        r.is_ok(),
        "None trust-list = default-empty must accept any DID"
    );
}

#[test]
fn trust_list_empty_vec_accepts_any_schema_author() {
    let manifest = synthesize_manifest(Some(Vec::new()));
    let any_author = Did::from_public_key(Keypair::generate().public_key());
    let r = validate_schema_author_within_manifest_envelope(&any_author, &manifest);
    assert!(
        r.is_ok(),
        "empty Vec trust-list = default-empty must accept any DID"
    );
}

#[test]
fn trust_list_with_author_in_list_accepts() {
    let trusted_kp = Keypair::generate();
    let trusted_did = Did::from_public_key(trusted_kp.public_key());
    let manifest = synthesize_manifest(Some(vec![trusted_did.clone()]));
    let r = validate_schema_author_within_manifest_envelope(&trusted_did, &manifest);
    assert!(r.is_ok(), "schema author IS in non-empty trust-list");
}

#[test]
fn trust_list_with_author_not_in_list_rejects() {
    let trusted_did = Did::from_public_key(Keypair::generate().public_key());
    let manifest = synthesize_manifest(Some(vec![trusted_did]));

    let untrusted_did = Did::from_public_key(Keypair::generate().public_key());
    let r = validate_schema_author_within_manifest_envelope(&untrusted_did, &manifest);
    assert_eq!(
        r,
        Err(ErrorCode::PluginAuthorNotTrusted),
        "schema author NOT in non-empty trust-list must surface \
         E_PLUGIN_AUTHOR_NOT_TRUSTED"
    );
}
