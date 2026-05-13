//! R6-FP-A substantive test pins (pim-2 §3.6b + sec-r6r1-1 BLOCKER
//! closure + arch-r6-r1-5 ErrorCode split + sec-r6r1-7 Any-tightening +
//! sec-r6r1-8 PluginAuthor-removal).
//!
//! Each test exercises the SPECIFIC arm + asserts an OBSERVABLE
//! consequence + would-FAIL-if-no-op'd. NO RED-PHASE-BODY: every test
//! drives production code through `plugin_lifecycle::install_plugin`
//! AND/OR `PluginManifest::validate`.

mod common;

use benten_errors::ErrorCode;
use benten_id::did::Did;
use benten_id::keypair::Keypair;
use benten_platform_foundation::plugin_library::PluginLibrary;
use benten_platform_foundation::plugin_lifecycle::{
    InMemoryInstallCascade, InstallContext, InstallerShape, install_plugin,
};
use benten_platform_foundation::plugin_manifest::{
    SharesPolicy, SharesPolicyDefault, SharesRule, SharesTarget,
};

#[test]
fn r6fp_a_arch_r6_r1_5_consenting_user_mismatch_surfaces_typed_consenting_user_code() {
    // arch-r6-r1-5 split (R6-FP-A): install_record's consenting_user_did
    // != ctx.user_did MUST surface
    // `PluginInstallRecordConsentingUserMismatch` — DISTINCT from the
    // null-consent (`PluginInstallConsentRequired`) and the manifest-cid-
    // mismatch (`PluginInstallRecordManifestCidMismatch`) arms.
    //
    // Would-FAIL if: install_plugin rolled all three arms back into the
    // umbrella PluginInstallConsentRequired (the pre-R6-FP-A shape).
    let alice = Keypair::generate();
    let user_kp = Keypair::generate();
    let user_did = user_kp.public_key().to_did();
    let other_kp = Keypair::generate();
    let other_user_did = other_kp.public_key().to_did();

    let manifest = common::manifest_fixtures::signed_manifest_by(
        &alice,
        "consent-user-mismatch-test",
        &["store:notes:read"],
    );
    let bytes = serde_ipld_dagcbor::to_vec(&manifest).expect("encode");
    let expected_cid = manifest.content_cid;
    let stub_plugin_did =
        Did::from_string_unchecked("did:key:z6MkR6FpAConsentUserMismatchStub".to_string());

    // Sign the InstallRecord with `other_kp` (so its
    // `consenting_user_did` = other_user_did, NOT the one we pass in
    // InstallContext.user_did).
    let record = common::manifest_fixtures::signed_install_record(
        &other_kp,
        expected_cid,
        stub_plugin_did.clone(),
        13,
    );

    let mut library = PluginLibrary::new();
    let mut store = benten_id::plugin_did::PluginDidStore::new();
    let mut cascade = InMemoryInstallCascade::new();
    let mut private_ns = InMemoryInstallCascade::new();
    let trust_list: Vec<Did> = vec![];
    let mut ctx = InstallContext {
        cap_minter: &mut cascade,
        private_ns: &mut private_ns,
        now_secs: 1_700_000_000,
        installer_shape: InstallerShape::FullPeer,
        user_trust_list: &trust_list,
        user_did: &user_did, // !=  record.consenting_user_did
        version_chain: None,
        prior_installed_cid: None,
    };
    let err = install_plugin(
        &mut library,
        &mut store,
        &mut ctx,
        &bytes,
        &expected_cid,
        &record,
        1,
        &|_| None,
    )
    .expect_err("install MUST reject when record's consenting_user_did != ctx.user_did");
    assert_eq!(
        err,
        ErrorCode::PluginInstallRecordConsentingUserMismatch,
        "arch-r6-r1-5 split: consenting_user_did mismatch MUST surface the typed \
         PluginInstallRecordConsentingUserMismatch variant, NOT the umbrella \
         PluginInstallConsentRequired. Would-FAIL if the seam re-collapsed."
    );
    assert!(library.is_empty(), "rejected install MUST NOT commit");
    let _ = other_user_did; // suppress unused warning
}

#[test]
fn r6fp_a_sec_r6r1_1_blocker_plugin_did_binding_load_bearing() {
    // sec-r6r1-1 BLOCKER closure (R6-FP-A): the install record's signed
    // plugin_did is now LOAD-BEARING — it becomes the install identity.
    // Previously install_plugin silently minted a fresh DID via OsRng
    // and the record's plugin_did was signed-but-ignored, defeating
    // the consent-payload integrity guarantee.
    //
    // SUBSTANTIVE pin: build two records with two DIFFERENT signed
    // plugin_dids, install both. The library MUST carry the record's
    // signed plugin_did, NOT a fresh-minted random.
    //
    // Would-FAIL if: install_plugin reverted to OsRng mint + discard
    // (the resulting library entry's plugin_did would NOT match either
    // record's signed value).
    let alice = Keypair::generate();
    let user_kp = Keypair::generate();
    let user_did = user_kp.public_key().to_did();

    let manifest = common::manifest_fixtures::signed_manifest_by(
        &alice,
        "did-binding-load-bearing",
        &["store:notes:read"],
    );
    let bytes = serde_ipld_dagcbor::to_vec(&manifest).expect("encode");
    let expected_cid = manifest.content_cid;

    // Record A: signs over plugin_did = "did:key:z6MkR6FpAPluginIdentityA".
    let plugin_did_a = Did::from_string_unchecked("did:key:z6MkR6FpAPluginIdentityA".to_string());
    let record_a = common::manifest_fixtures::signed_install_record(
        &user_kp,
        expected_cid,
        plugin_did_a.clone(),
        17,
    );

    let mut library = PluginLibrary::new();
    let mut store = benten_id::plugin_did::PluginDidStore::new();
    let mut cascade = InMemoryInstallCascade::new();
    let mut private_ns = InMemoryInstallCascade::new();
    let trust_list: Vec<Did> = vec![];
    let mut ctx = InstallContext {
        cap_minter: &mut cascade,
        private_ns: &mut private_ns,
        now_secs: 1_700_000_000,
        installer_shape: InstallerShape::FullPeer,
        user_trust_list: &trust_list,
        user_did: &user_did,
        version_chain: None,
        prior_installed_cid: None,
    };
    let outcome = install_plugin(
        &mut library,
        &mut store,
        &mut ctx,
        &bytes,
        &expected_cid,
        &record_a,
        1,
        &|_| None,
    )
    .expect("install with record_a MUST admit");

    // OBSERVABLE CONSEQUENCE: the library entry carries plugin_did_a,
    // NOT a fresh random. If install_plugin reverted to mint-and-
    // discard, the entry's plugin_did would not equal plugin_did_a.
    assert_eq!(
        outcome.entry.plugin_did, plugin_did_a,
        "sec-r6r1-1 BLOCKER closure: library entry's plugin_did MUST equal \
         the InstallRecord's signed plugin_did (the user's consent surface), \
         NOT a freshly-minted random. Would-FAIL if mint-and-discard reverted."
    );

    // Defense-in-depth assertion: the cascade minted grants under
    // audience=plugin_did_a (NOT some random). This confirms the
    // record's plugin_did flowed into the cap-cascade.
    let grants = cascade.minted_grants();
    assert!(!grants.is_empty(), "cap-cascade MUST mint grants");
    for (_, audience_did, _scope, _cid) in grants {
        assert_eq!(
            audience_did, &plugin_did_a,
            "cap-cascade audience MUST be the record's signed plugin_did (sec-r6r1-1)"
        );
    }
}

#[test]
fn r6fp_a_sec_r6r1_7_shares_any_with_no_rules_now_rejected_at_validate() {
    // sec-r6r1-7 hardening (R6-FP-A): `SharesPolicyDefault::Any` paired
    // with `rules: None` was the footgun arm — a manifest could ship
    // "shares any everything to anything" without specifying a single
    // rule. `validate()` now rejects this shape with
    // E_PLUGIN_MANIFEST_INVALID.
    //
    // Would-FAIL if: validate() reverted to admitting Any+None.
    let alice = Keypair::generate();
    let mut manifest = common::manifest_fixtures::signed_manifest_by(
        &alice,
        "any-no-rules-test",
        &["store:notes:read"],
    );
    // Override shares to the footgun shape.
    manifest.shares = SharesPolicy {
        default: SharesPolicyDefault::Any,
        rules: None,
    };
    // Re-sign so peer-signature still verifies — but validate() check
    // doesn't depend on signature here; we want validate() to reject
    // the shares-shape regardless.
    let err = manifest.validate().expect_err("Any+None MUST be rejected");
    assert_eq!(
        err,
        ErrorCode::PluginManifestInvalid,
        "sec-r6r1-7: SharesPolicyDefault::Any + rules:None MUST be rejected at \
         validate() — Would-FAIL if validator regressed to admit"
    );

    // POSITIVE arm: Any + rules with at least one entry IS valid.
    // (The validator now requires Any to carry explicit rules so the
    // consent-UX has something to render.)
    let any_kp = Keypair::generate();
    let mut manifest_ok = common::manifest_fixtures::signed_manifest_by(
        &any_kp,
        "any-with-rules-test",
        &["store:notes:read"],
    );
    manifest_ok.shares = SharesPolicy {
        default: SharesPolicyDefault::Any,
        rules: Some(vec![SharesRule {
            cap_pattern: "store:notes:read".to_string(),
            target: SharesTarget::Any,
        }]),
    };
    // No need to re-compute content_cid for validate-shape testing —
    // validate() doesn't consult content_cid. The validator should
    // admit Any+rules.
    manifest_ok
        .validate()
        .expect("Any + non-empty rules MUST validate");
}

#[test]
fn r6fp_a_sec_r6r1_8_shares_target_plugin_author_variant_removed() {
    // sec-r6r1-8 closure (R6-FP-A): the `SharesTarget::PluginAuthor(Did)`
    // variant was dead code (matches() returned false unconditionally,
    // silently denying while masquerading as a feature). Removed
    // entirely.
    //
    // SUBSTANTIVE pin: type-level — the enum variants are now exactly
    // [Any, PluginDid]. This is a compile-time + match-exhaustiveness
    // guard. The test exercises both remaining variants to confirm
    // they are the canonical set.
    //
    // Would-FAIL if: SharesTarget::PluginAuthor were re-added without
    // implementing the resolver-lookup wiring (per the doc-comment).
    let target_a = SharesTarget::Any;
    let target_b = SharesTarget::PluginDid(Did::from_string_unchecked(
        "did:key:z6MkSharesTargetSet".to_string(),
    ));

    // The exhaustive match below would not compile if PluginAuthor
    // returned to the enum — that's the type-level pin.
    fn assert_only_two_variants(t: &SharesTarget) -> &'static str {
        match t {
            SharesTarget::Any => "any",
            SharesTarget::PluginDid(_) => "plugin-did",
        }
    }
    assert_eq!(assert_only_two_variants(&target_a), "any");
    assert_eq!(assert_only_two_variants(&target_b), "plugin-did");
}
