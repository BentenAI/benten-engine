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

    // R6-FP-A-fp caller-mint-first: real keypair backing the plugin-DID.
    let mut store = benten_id::plugin_did::PluginDidStore::new();
    let plugin_did = common::manifest_fixtures::mint_and_insert_plugin_did(&mut store);

    // Sign the InstallRecord with `other_kp` (so its
    // `consenting_user_did` = other_user_did, NOT the one we pass in
    // InstallContext.user_did).
    let record = common::manifest_fixtures::signed_install_record(
        &other_kp,
        expected_cid,
        plugin_did.clone(),
        13,
    );

    let mut library = PluginLibrary::new();
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
        expected_plugin_did: &plugin_did,
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
    // sec-r6r1-1 BLOCKER closure (R6-FP-A + R6-FP-A-fp): the install
    // record's signed plugin_did is now LOAD-BEARING — it becomes the
    // install identity AND must equal `InstallContext::expected_plugin_did`
    // (caller's claim of which DID the user signed for). Previously
    // install_plugin silently minted a fresh DID via OsRng and the
    // record's plugin_did was signed-but-ignored, defeating the
    // consent-payload integrity guarantee.
    //
    // SUBSTANTIVE pin: pre-mint via the caller-mint-first pattern,
    // install with a record over that DID, assert the entry's
    // plugin_did equals the pre-minted DID AND that the cap-cascade
    // minted grants under that audience.
    //
    // Would-FAIL if: install_plugin reverted to OsRng mint + discard
    // (the resulting library entry's plugin_did would NOT match the
    // pre-minted DID).
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

    // Caller-mint-first: pre-mint + insert a real keypair handle.
    let mut store = benten_id::plugin_did::PluginDidStore::new();
    let plugin_did_a = common::manifest_fixtures::mint_and_insert_plugin_did(&mut store);
    let record_a = common::manifest_fixtures::signed_install_record(
        &user_kp,
        expected_cid,
        plugin_did_a.clone(),
        17,
    );

    let mut library = PluginLibrary::new();
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
        expected_plugin_did: &plugin_did_a,
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

    // Defense-in-depth (mr-2 closure): the keypair handle survives
    // install. The pre-fp implementation left the store empty post-
    // install (orphaned keypair); post-fp the handle remains.
    assert!(
        store.get(&plugin_did_a).is_some(),
        "mr-2 closure: caller-pre-inserted handle MUST remain in store \
         post-install; would-FAIL if Step 8 cleared it"
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
#[allow(clippy::too_many_lines)]
fn r6fp_a_mr_5_adversarial_plugin_did_substitution_rejected() {
    // mr-5 (R6-FP-A-fp): adversarial arm per pim-18 §3.6f
    // SHAPE-not-SUBSTANCE — exercises the SUBSTANTIVE defense, not
    // just the load-bearing-not-discarded shape.
    //
    // Two arms:
    //  (a) Attacker tampers with `install_record.plugin_did` AFTER user
    //      signs. Because `InstallRecord::signing_payload` binds
    //      `plugin_did_bytes`, any tamper invalidates the user-DID
    //      signature. install_plugin rejects with
    //      `PluginInstallRecordUserSignatureInvalid` at Step 3.
    //  (b) Caller passes a wrong `expected_plugin_did` (legitimate
    //      user-signed record, but the caller's claim about which
    //      plugin-DID the user signed for is different). install_plugin
    //      rejects at Step 8 with `PluginInstallRecordPluginDidMismatch`.
    //
    // Both arms are substantive: they drive `install_plugin` end-to-end
    // through real production paths, not mocks.
    let alice = Keypair::generate();
    let user_kp = Keypair::generate();
    let user_did = user_kp.public_key().to_did();

    let manifest = common::manifest_fixtures::signed_manifest_by(
        &alice,
        "adversarial-plugin-did-substitution",
        &["store:notes:read"],
    );
    let bytes = serde_ipld_dagcbor::to_vec(&manifest).expect("encode");
    let expected_cid = manifest.content_cid;

    // ARM (a) — tamper-after-sign:
    let mut store_a = benten_id::plugin_did::PluginDidStore::new();
    let plugin_did_legit = common::manifest_fixtures::mint_and_insert_plugin_did(&mut store_a);
    let plugin_did_attacker = common::manifest_fixtures::mint_and_insert_plugin_did(&mut store_a);
    let mut record_tampered = common::manifest_fixtures::signed_install_record(
        &user_kp,
        expected_cid,
        plugin_did_legit.clone(),
        23,
    );
    // Attack: swap the plugin_did to attacker's DID AFTER the user
    // signature is computed. The signed payload still encodes
    // plugin_did_legit, so the verification of the new payload (with
    // plugin_did_attacker bytes) will fail.
    record_tampered.plugin_did = plugin_did_attacker.clone();

    let mut library_a = PluginLibrary::new();
    let mut cascade_a = InMemoryInstallCascade::new();
    let mut private_ns_a = InMemoryInstallCascade::new();
    let trust_list: Vec<Did> = vec![];
    let mut ctx_a = InstallContext {
        cap_minter: &mut cascade_a,
        private_ns: &mut private_ns_a,
        now_secs: 1_700_000_000,
        installer_shape: InstallerShape::FullPeer,
        user_trust_list: &trust_list,
        user_did: &user_did,
        version_chain: None,
        prior_installed_cid: None,
        expected_plugin_did: &plugin_did_attacker,
    };
    let err_a = install_plugin(
        &mut library_a,
        &mut store_a,
        &mut ctx_a,
        &bytes,
        &expected_cid,
        &record_tampered,
        1,
        &|_| None,
    )
    .expect_err("tampered plugin_did MUST be rejected by user-signature check");
    assert_eq!(
        err_a,
        ErrorCode::PluginInstallRecordUserSignatureInvalid,
        "ARM (a): tamper-after-sign MUST surface PluginInstallRecordUserSignatureInvalid \
         (the signing_payload binds plugin_did_bytes; any tamper invalidates sig). \
         Would-FAIL if Step 3 skipped signature re-verification against tampered bytes."
    );
    assert!(library_a.is_empty(), "tampered install MUST NOT commit");

    // ARM (b) — wrong expected_plugin_did at the caller boundary:
    let mut store_b = benten_id::plugin_did::PluginDidStore::new();
    let plugin_did_signed = common::manifest_fixtures::mint_and_insert_plugin_did(&mut store_b);
    let plugin_did_caller_claim =
        common::manifest_fixtures::mint_and_insert_plugin_did(&mut store_b);
    // User legitimately signs over plugin_did_signed; caller (perhaps
    // confused or attacked at the call site) passes
    // plugin_did_caller_claim as the expected.
    let record_legit = common::manifest_fixtures::signed_install_record(
        &user_kp,
        expected_cid,
        plugin_did_signed.clone(),
        29,
    );
    let mut library_b = PluginLibrary::new();
    let mut cascade_b = InMemoryInstallCascade::new();
    let mut private_ns_b = InMemoryInstallCascade::new();
    let mut ctx_b = InstallContext {
        cap_minter: &mut cascade_b,
        private_ns: &mut private_ns_b,
        now_secs: 1_700_000_000,
        installer_shape: InstallerShape::FullPeer,
        user_trust_list: &trust_list,
        user_did: &user_did,
        version_chain: None,
        prior_installed_cid: None,
        expected_plugin_did: &plugin_did_caller_claim,
    };
    let err_b = install_plugin(
        &mut library_b,
        &mut store_b,
        &mut ctx_b,
        &bytes,
        &expected_cid,
        &record_legit,
        1,
        &|_| None,
    )
    .expect_err("caller's wrong expected_plugin_did MUST be rejected at Step 8");
    assert_eq!(
        err_b,
        ErrorCode::PluginInstallRecordPluginDidMismatch,
        "ARM (b): caller's expected_plugin_did != record.plugin_did MUST surface \
         PluginInstallRecordPluginDidMismatch at Step 8. This is the typed forensic \
         discrimination ErrorCode that R6-FP-A's first commit minted but had no firing \
         path; mr-1 BLOCKER closure gives it the firing path."
    );
    assert!(
        library_b.is_empty(),
        "Step-8-rejected install MUST NOT commit"
    );

    // ARM (c) — orphan-handle defense (mr-2 closure): legitimate
    // record + matching expected, BUT caller forgot the
    // caller-mint-first insert. install_plugin rejects with
    // PluginDidHandleNotPreInserted.
    let mut store_c = benten_id::plugin_did::PluginDidStore::new(); // EMPTY
    let throwaway_handle = benten_id::plugin_did::mint();
    let plugin_did_orphan = throwaway_handle.did().clone();
    // Do NOT insert — that's the failure mode under test.
    let record_orphan = common::manifest_fixtures::signed_install_record(
        &user_kp,
        expected_cid,
        plugin_did_orphan.clone(),
        31,
    );
    let mut library_c = PluginLibrary::new();
    let mut cascade_c = InMemoryInstallCascade::new();
    let mut private_ns_c = InMemoryInstallCascade::new();
    let mut ctx_c = InstallContext {
        cap_minter: &mut cascade_c,
        private_ns: &mut private_ns_c,
        now_secs: 1_700_000_000,
        installer_shape: InstallerShape::FullPeer,
        user_trust_list: &trust_list,
        user_did: &user_did,
        version_chain: None,
        prior_installed_cid: None,
        expected_plugin_did: &plugin_did_orphan,
    };
    let err_c = install_plugin(
        &mut library_c,
        &mut store_c,
        &mut ctx_c,
        &bytes,
        &expected_cid,
        &record_orphan,
        1,
        &|_| None,
    )
    .expect_err("orphan-handle install MUST be rejected at Step 8");
    assert_eq!(
        err_c,
        ErrorCode::PluginDidHandleNotPreInserted,
        "ARM (c) mr-2 closure: caller skipped pre-insert MUST surface \
         PluginDidHandleNotPreInserted. Would-FAIL if Step 8 silently accepted \
         the orphan keypair (the pre-fp behavior under the empty-branch BLOCKER)."
    );
    assert!(
        library_c.is_empty(),
        "orphan-rejected install MUST NOT commit"
    );
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

#[test]
fn r6fp_a_mr_7_manifest_envelope_recheck_outcome_drives_row_reject() {
    // mr-7 (R6-FP-A-fp) end-to-end pin for plugin-arch BLOCKER #2:
    // the rechecker port is now structurally wired (default flipped
    // from None → Some(Noop) in engine.rs). This pin asserts that:
    //
    //  - swapping in a rechecker that returns `OutsideEnvelope` → row
    //    is rejected with typed
    //    `E_PLUGIN_DELEGATION_OUTSIDE_MANIFEST_ENVELOPE`.
    //  - swapping back to the default `NoopManifestEnvelopeRechecker`
    //    (returns `NotApplicable`) → row admits.
    //  - the third outcome `Admitted` → row admits.
    //
    // Exercises the same `outcome_to_row_reject` function the
    // production `apply_atrium_merge` per-row loop calls (engine.rs:1406).
    // Avoids spinning a full Engine + iroh harness — the trait outcome
    // → ErrorCode mapping IS the surface tested by mr-7.
    //
    // Would-FAIL if: the `NotApplicable` arm got rerouted to Reject
    // (mr-6 hypothetical from plugin-arch-cap-policy lens), or the
    // `OutsideEnvelope` arm regressed to silently admit (the pre-R6-FP
    // default-None silent-skip).
    use benten_engine::manifest_envelope_recheck::{
        ManifestEnvelopeRecheckOutcome, NoopManifestEnvelopeRechecker, outcome_to_row_reject,
    };

    // ARM (a) — OutsideEnvelope outcome rejects with typed code.
    let outside = ManifestEnvelopeRecheckOutcome::OutsideEnvelope {
        offending_plugin_did: "did:key:z6MkOffender".to_string(),
        cap_pattern: "private:victim:secrets".to_string(),
    };
    let res_a = outcome_to_row_reject(outside, "zone-1", "key-1");
    let err_a = res_a.expect_err("OutsideEnvelope MUST reject");
    assert_eq!(
        err_a.code(),
        ErrorCode::PluginDelegationOutsideManifestEnvelope,
        "ARM (a): OutsideEnvelope outcome MUST surface typed \
         PluginDelegationOutsideManifestEnvelope. Would-FAIL if the row \
         loop's outcome → ErrorCode mapping regressed."
    );

    // ARM (b) — Noop default's NotApplicable outcome admits.
    let noop_outcome = ManifestEnvelopeRecheckOutcome::NotApplicable;
    let res_b = outcome_to_row_reject(noop_outcome, "zone-2", "key-2");
    assert!(
        res_b.is_ok(),
        "ARM (b): NotApplicable outcome (Noop default) MUST admit. \
         Engines without a real adapter wired behave observably as Phase-3 \
         baseline. Would-FAIL if the default-flip regressed to silent-reject \
         (which would break every existing test)."
    );

    // ARM (c) — Admitted outcome admits.
    let admitted = ManifestEnvelopeRecheckOutcome::Admitted;
    let res_c = outcome_to_row_reject(admitted, "zone-3", "key-3");
    assert!(
        res_c.is_ok(),
        "ARM (c): Admitted outcome MUST admit (the real-rechecker happy path)."
    );

    // STRUCTURAL: instantiating a NoopManifestEnvelopeRechecker via
    // its public surface confirms the type is constructable + matches
    // the engine builder default. Closes plugin-arch BLOCKER #2.
    // (Type is Copy + zero-sized; this is purely a name/visibility
    // check that surfaces at compile time.)
    let _: NoopManifestEnvelopeRechecker = NoopManifestEnvelopeRechecker;
}
