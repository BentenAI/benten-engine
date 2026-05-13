//! G24-D + T6b defense (sec-3.5-r1-6 reframed).
//!
//! Hostile peer constructs a malicious admin UI subgraph; computes its
//! CID; publishes to atrium under same human-readable name. Without
//! peer-DID signature verification, content-addressing alone passes
//! (hostile bytes hash to their own CID). With peer-DID signature
//! verification, hostile peer's signature doesn't match Alice's
//! peer-DID and install rejects.

mod common;

use benten_errors::ErrorCode;
use benten_id::keypair::Keypair;
use benten_platform_foundation::plugin_library::PluginLibrary;
use benten_platform_foundation::plugin_lifecycle::{
    InMemoryInstallCascade, InstallContext, InstallerShape, install_plugin,
};

#[test]
fn substituted_bundle_with_different_peer_did_signature_rejected_at_install() {
    // **R4b-FP-1 Seam 1** un-ignore — substantive substitution-defense
    // end-to-end via install_plugin's two-stage check:
    //  (i) peer-DID signature verification (existing G24-D primary).
    //  (ii) user-DID trust-list check (NEW at R4b-FP-1 Seam 1).
    //
    // Both arms surface DIFFERENT typed codes per pim-2-amendment
    // §3.6b sub-rule 4 (per-finding granularity):
    //  (a) attacker forges manifest claiming alice's peer_did but
    //      signs with attacker's key → E_PLUGIN_CONTENT_PEER_SIGNATURE_INVALID
    //      (covered by sibling plugin_manifest_substitution_at_install_rejected.rs).
    //  (b) attacker honestly signs as themselves but user's trust-list
    //      doesn't contain attacker → E_PLUGIN_AUTHOR_NOT_TRUSTED.
    //
    // This test pins arm (b) — the trust-list cross-Atrium-substitution
    // arm. Together they form defense in depth.
    let alice = Keypair::generate(); // trusted author
    let attacker = Keypair::generate(); // un-trusted; could be any author
    let user_kp = Keypair::generate();
    let user_did = user_kp.public_key().to_did();

    // Attacker constructs a manifest honestly signed by THEIR key
    // (passes the peer-DID-signature check) but published under same
    // human-readable name as alice's admin UI v0. The trust-list
    // (containing only alice) catches the substitution.
    let hostile = common::manifest_fixtures::signed_manifest_by(
        &attacker,
        "admin-ui-v0", // same plugin name as alice's
        &["private:admin-ui-private:foo", "store:plugins:read"],
    );
    let bytes = serde_ipld_dagcbor::to_vec(&hostile).expect("encode");
    let expected_cid = hostile.content_cid;
    let install_record = common::manifest_fixtures::signed_install_record(
        &user_kp,
        expected_cid,
        benten_id::did::Did::from_string_unchecked("did:key:z6MkSubstitutedBundle".to_string()),
        2,
    );

    let mut library = PluginLibrary::new();
    let mut store = benten_id::plugin_did::PluginDidStore::new();
    let mut cascade = InMemoryInstallCascade::new();
    let mut private_ns = InMemoryInstallCascade::new();
    // User's trust-list contains alice only (NOT attacker).
    let trust_list = vec![alice.public_key().to_did()];
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

    let attempt = install_plugin(
        &mut library,
        &mut store,
        &mut ctx,
        &bytes,
        &expected_cid,
        &install_record,
        1,
        &|_| None,
    );
    let err = attempt.expect_err("substituted bundle MUST be rejected by trust-list arm");
    assert_eq!(
        err,
        ErrorCode::PluginAuthorNotTrusted,
        "Substitution-defense (b): attacker honestly signed but author NOT in user's \
         trust-list MUST surface typed E_PLUGIN_AUTHOR_NOT_TRUSTED; \
         would-FAIL if install_plugin skipped trust-list check"
    );
    assert!(
        library.is_empty(),
        "rejected install MUST NOT commit library state"
    );
}
