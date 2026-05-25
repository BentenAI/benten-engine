//! R6 R1 fix-pass (Bundle L2-R6-MAJOR-1) — T10-upgrade (a)
//! peer-DID-substitution rejection END-TO-END pin (install_plugin).
//!
//! Closes R6 R1 L2 finding L2-R6-MAJOR-1 (MAJOR — T10-upgrade (a)
//! standalone helper present + tested but NOT WIRED into the
//! install_plugin pipeline at HEAD).
//!
//! ## What this pin establishes
//!
//! The earlier pin `plugin_upgrade_requires_same_author_did.rs`
//! exercises `verify_upgrade_author_continuity` as a standalone
//! function — but the production install pipeline never called it.
//! Per admin-ui-v0-threat-model.md §T10 (a):
//!
//!   "Upgrade from peer-DID alice to peer-DID attacker (transitive
//!    substitution) MUST be REJECTED."
//!
//! At HEAD (pre-R6 R1 fix-pass), install_plugin step 7 enforced only
//! T10-(b) (DAG-descendant); a peer-DID substitution where attacker
//! controlled a DAG-descendant CID slipped through silently. META
//! #707 asymmetric-at-parallel-entry-points pattern: T10-(a) and
//! T10-(b) are co-defensive; this pin verifies the (a) half is wired.
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Remove the step 7a wire-in: this test would observe an attacker
//! peer-DID upgrade ADMITTED (silent peer-DID substitution against a
//! user's installed plugin).

#![allow(clippy::unwrap_used)]

mod common;

use benten_core::version_chain::DagVersionChain;
use benten_errors::ErrorCode;
use benten_id::keypair::Keypair;
use benten_platform_foundation::plugin_library::PluginLibrary;
use benten_platform_foundation::plugin_lifecycle::{
    InMemoryInstallCascade, InstallParams, InstallPorts, InstallerShape, install_plugin,
};
// R6 R1 FP integration consolidation 2026-05-24:
// B's branch was based on a0b75637 (pre-F4) when InstallPorts had
// `install_record_replay_check: Option<&mut Fn>` and no `policy` field.
// F4's Δv3-2 + S2 + S3a dropped the Option AND added the policy port.
// This test is plugin-upgrade-shape (not exercising replay or install-consent),
// so wires admit-everything helpers from foundation::testing + ::install_consent.
use benten_platform_foundation::install_consent::AdmitAllInstallConsent;
use benten_platform_foundation::testing::noop_replay_check;

#[test]
#[allow(clippy::too_many_lines)]
fn install_plugin_rejects_peer_did_substitution_on_upgrade_path() {
    let alice = Keypair::generate();
    let attacker = Keypair::generate();
    let user_kp = Keypair::generate();
    let user_did = user_kp.public_key().to_did();
    assert_ne!(
        alice.public_key().to_did(),
        attacker.public_key().to_did(),
        "test setup: alice + attacker have distinct DIDs"
    );

    // Build v1 (alice-signed) + v2_attacker (attacker-signed, but a
    // DAG-descendant of v1 in the chain — the attack scenario where
    // attacker controls a descendant CID).
    let v1 =
        common::manifest_fixtures::signed_manifest_by(&alice, "t10a-test", &["store:notes:read"]);
    let v2_attacker = common::manifest_fixtures::signed_manifest_by(
        &attacker,
        "t10a-test-attacker",
        &["store:notes:read"],
    );

    let mut chain = DagVersionChain::new(v1.content_cid);
    chain
        .add_version(v1.content_cid, v2_attacker.content_cid)
        .expect("v1 → v2_attacker OK (DAG-descendant established)");

    // Pre-install v1 = CURRENT (fresh install, alice's peer-DID is
    // baked into the install_record by the alice-trust path).
    let bytes_v1 = serde_ipld_dagcbor::to_vec(&v1).expect("encode v1");
    let mut library = PluginLibrary::new();
    let mut store = benten_id::plugin_did::PluginDidStore::new();
    let plugin_did_v1 = common::manifest_fixtures::mint_and_insert_plugin_did(&mut store);
    let install_v1 = common::manifest_fixtures::signed_install_record(
        &user_kp,
        v1.content_cid,
        plugin_did_v1.clone(),
        3,
    );

    let mut cascade = InMemoryInstallCascade::new();
    let mut private_ns = InMemoryInstallCascade::new();
    let trust_list: Vec<benten_id::did::Did> = vec![];

    {
        let mut ctx = InstallPorts {
            cap_minter: &mut cascade,
            private_ns: &mut private_ns,
            install_record_replay_check: &mut noop_replay_check(),
            policy: &AdmitAllInstallConsent,
        };
        let ctx_params = InstallParams {
            now_secs: 1_700_000_000,
            installer_shape: InstallerShape::FullPeer,
            user_trust_list: &trust_list,
            user_did: &user_did,
            version_chain: None,
            prior_installed_cid: None,
            expected_plugin_did: &plugin_did_v1,
        };
        install_plugin(
            &mut library,
            &mut store,
            &mut ctx,
            &ctx_params,
            &bytes_v1,
            &v1.content_cid,
            &install_v1,
            1,
            &|_| None,
        )
        .expect("fresh install of v1 (alice-signed) admits");
    }
    assert_eq!(library.len(), 1);

    // ATTACK: upgrade to v2_attacker (a DAG-descendant of v1; passes
    // the T10-(b) descendant check) — but signed by attacker, not alice.
    // T10-(a) MUST reject this even though the DAG-descendant check
    // would pass.
    //
    // The resolver MUST return v1 when queried for v1.content_cid (the
    // prior manifest). This is the integration scenario where the
    // engine has prior_cid recorded + resolver gives back the prior
    // manifest at upgrade time.
    let v1_for_resolver = v1.clone();
    let resolver = |cid: &benten_core::Cid| -> Option<benten_platform_foundation::plugin_manifest::PluginManifest> {
        if *cid == v1_for_resolver.content_cid {
            Some(v1_for_resolver.clone())
        } else {
            None
        }
    };

    let bytes_v2 = serde_ipld_dagcbor::to_vec(&v2_attacker).expect("encode v2_attacker");
    let plugin_did_v2 = common::manifest_fixtures::mint_and_insert_plugin_did(&mut store);
    let install_v2 = common::manifest_fixtures::signed_install_record(
        &user_kp,
        v2_attacker.content_cid,
        plugin_did_v2.clone(),
        4,
    );
    let mut cascade2 = InMemoryInstallCascade::new();
    let mut private_ns2 = InMemoryInstallCascade::new();
    let mut ctx_upgrade = InstallPorts {
        cap_minter: &mut cascade2,
        private_ns: &mut private_ns2,
        install_record_replay_check: &mut noop_replay_check(),
        policy: &AdmitAllInstallConsent,
    };
    let ctx_upgrade_params = InstallParams {
        now_secs: 1_700_000_000,
        installer_shape: InstallerShape::FullPeer,
        user_trust_list: &trust_list,
        user_did: &user_did,
        version_chain: Some(&chain),
        prior_installed_cid: Some(v1.content_cid),
        expected_plugin_did: &plugin_did_v2,
    };
    let upgrade_attempt = install_plugin(
        &mut library,
        &mut store,
        &mut ctx_upgrade,
        &ctx_upgrade_params,
        &bytes_v2,
        &v2_attacker.content_cid,
        &install_v2,
        2,
        &resolver,
    );
    let err = upgrade_attempt
        .expect_err("T10-upgrade (a): peer-DID substitution MUST be REJECTED at install_plugin");
    assert_eq!(
        err,
        ErrorCode::PluginAuthorNotTrusted,
        "T10-upgrade (a): peer-DID substitution upgrade MUST surface typed \
         PluginAuthorNotTrusted; got {err:?}"
    );

    // Defense-in-depth: library state unchanged.
    assert_eq!(
        library.len(),
        1,
        "rejected peer-DID-substitution upgrade MUST NOT alter library"
    );
    assert!(
        library.get(&v1.content_cid).is_some(),
        "v1 entry (alice-signed) MUST remain"
    );
    assert!(
        library.get(&v2_attacker.content_cid).is_none(),
        "attacker-signed v2 MUST NOT be present in library"
    );
}

#[test]
#[allow(
    clippy::too_many_lines,
    reason = "Linear end-to-end positive-control test wiring v1 install + \
              v2 same-peer-DID upgrade through install_plugin(...) — boundary \
              pin for T10-(a) over-strictness. Inlining keeps the v1→v2 \
              ordering audit-able as a single test body; helper-extraction \
              would split the would-FAIL-on-revert assertion across helpers \
              and obscure the test's intent."
)]
fn install_plugin_admits_same_peer_did_upgrade_on_upgrade_path() {
    // Positive control / boundary: same peer-DID upgrade DOES admit
    // (assuming all other gates pass). Would-FAIL if the T10-(a) check
    // is over-strict and rejects legitimate same-author upgrades.
    let alice = Keypair::generate();
    let user_kp = Keypair::generate();
    let user_did = user_kp.public_key().to_did();

    let v1 = common::manifest_fixtures::signed_manifest_by(
        &alice,
        "t10a-positive-v1",
        &["store:notes:read"],
    );
    let v2 = common::manifest_fixtures::signed_manifest_by(
        &alice,
        "t10a-positive-v2",
        &["store:notes:read"],
    );

    let mut chain = DagVersionChain::new(v1.content_cid);
    chain
        .add_version(v1.content_cid, v2.content_cid)
        .expect("v1 → v2");

    let bytes_v1 = serde_ipld_dagcbor::to_vec(&v1).expect("encode v1");
    let mut library = PluginLibrary::new();
    let mut store = benten_id::plugin_did::PluginDidStore::new();
    let plugin_did_v1 = common::manifest_fixtures::mint_and_insert_plugin_did(&mut store);
    let install_v1 = common::manifest_fixtures::signed_install_record(
        &user_kp,
        v1.content_cid,
        plugin_did_v1.clone(),
        3,
    );

    let mut cascade = InMemoryInstallCascade::new();
    let mut private_ns = InMemoryInstallCascade::new();
    let trust_list: Vec<benten_id::did::Did> = vec![];

    {
        let mut ctx = InstallPorts {
            cap_minter: &mut cascade,
            private_ns: &mut private_ns,
            install_record_replay_check: &mut noop_replay_check(),
            policy: &AdmitAllInstallConsent,
        };
        let ctx_params = InstallParams {
            now_secs: 1_700_000_000,
            installer_shape: InstallerShape::FullPeer,
            user_trust_list: &trust_list,
            user_did: &user_did,
            version_chain: None,
            prior_installed_cid: None,
            expected_plugin_did: &plugin_did_v1,
        };
        install_plugin(
            &mut library,
            &mut store,
            &mut ctx,
            &ctx_params,
            &bytes_v1,
            &v1.content_cid,
            &install_v1,
            1,
            &|_| None,
        )
        .expect("fresh install of v1 admits");
    }

    let v1_for_resolver = v1.clone();
    let resolver = |cid: &benten_core::Cid| -> Option<benten_platform_foundation::plugin_manifest::PluginManifest> {
        if *cid == v1_for_resolver.content_cid {
            Some(v1_for_resolver.clone())
        } else {
            None
        }
    };

    let bytes_v2 = serde_ipld_dagcbor::to_vec(&v2).expect("encode v2");
    let plugin_did_v2 = common::manifest_fixtures::mint_and_insert_plugin_did(&mut store);
    let install_v2 = common::manifest_fixtures::signed_install_record(
        &user_kp,
        v2.content_cid,
        plugin_did_v2.clone(),
        4,
    );
    let mut cascade2 = InMemoryInstallCascade::new();
    let mut private_ns2 = InMemoryInstallCascade::new();
    let mut ctx_upgrade = InstallPorts {
        cap_minter: &mut cascade2,
        private_ns: &mut private_ns2,
        install_record_replay_check: &mut noop_replay_check(),
        policy: &AdmitAllInstallConsent,
    };
    let ctx_upgrade_params = InstallParams {
        now_secs: 1_700_000_000,
        installer_shape: InstallerShape::FullPeer,
        user_trust_list: &trust_list,
        user_did: &user_did,
        version_chain: Some(&chain),
        prior_installed_cid: Some(v1.content_cid),
        expected_plugin_did: &plugin_did_v2,
    };
    install_plugin(
        &mut library,
        &mut store,
        &mut ctx_upgrade,
        &ctx_upgrade_params,
        &bytes_v2,
        &v2.content_cid,
        &install_v2,
        2,
        &resolver,
    )
    .expect("same-peer-DID upgrade MUST admit (T10-(a) positive control)");
}
