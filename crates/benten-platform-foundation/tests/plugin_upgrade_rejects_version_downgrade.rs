//! Phase-4-Foundation R4-FP-1 — T10-upgrade (b) pin: plugin upgrade
//! rejects DAG-version downgrade (CID not a descendant of installed).
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §1 BLOCKER row
//! r4-tc-3 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md`
//! §T10 defense step 4(b) + D-4F-14 DAG-shape version chain extension
//! of Phase-1 anchor + Version Node pattern.
//!
//! ## What this pin establishes
//!
//! Per threat-model §T10 defense step 4(b): "DAG-shaped version chain
//! monotonicity (per D-4F-14: version chain extends Phase-1 anchor +
//! Version Node pattern to DAG-shape; CURRENT can advance to any
//! reachable descendant; upgrade rejected if new version is NOT a
//! descendant of installed in the DAG)."
//!
//! NOTE per CLAUDE.md baked-in #18 + D-4F-13: "manifest-schema-version
//! downgrade" is RETIRED — CID covers shape; pull-not-push means
//! receiver controls. This pin is about the DAG-shape version chain
//! (anchor + Version Node descendants); NOT about a separate
//! manifest schema version field.
//!
//! Per pim-2-amendment §3.6b sub-rule 4: T10-upgrade (b) is a
//! SEPARATE pin from (a) same-author per per-finding granularity.
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer wires same-author check (a) but skips DAG-descendant
//! check. Attacker offers an "upgrade" CID that's an ANCESTOR of the
//! installed version (a known-vulnerable older version); upgrade
//! flow accepts silently; user downgraded to vulnerable code without
//! re-consent.

#![allow(clippy::unwrap_used)]

mod common;

use benten_errors::ErrorCode;
use common::manifest_fixtures::{stub_peer_did_alice, stub_plugin_did, stub_user_did};

#[test]
#[allow(clippy::too_many_lines)]
fn plugin_upgrade_rejects_cid_not_a_dag_descendant_of_installed_version() {
    // **R4b-FP-1 Seam 1** un-ignore — substantive T10-upgrade (b) per
    // pim-2-amendment §3.6b sub-rule 4: install_plugin's upgrade arm
    // (driven by ctx.version_chain + ctx.prior_installed_cid) rejects
    // when the new CID is NOT a DAG-descendant of the prior installed
    // CID. Would-FAIL if seam skipped the is_ancestor_of check.
    use benten_core::version_chain::DagVersionChain;
    use benten_id::keypair::Keypair;
    use benten_platform_foundation::plugin_library::PluginLibrary;
    use benten_platform_foundation::plugin_lifecycle::{
        InMemoryInstallCascade, InstallContext, InstallerShape, install_plugin,
    };

    let _ = (stub_peer_did_alice(), stub_plugin_did(), stub_user_did());

    let alice = Keypair::generate();
    let user_kp = Keypair::generate();
    let user_did = user_kp.public_key().to_did();

    // Build a 3-version chain v1 → v2 → v3 + a v2-fork branch (under
    // v1 as parallel branch). CIDs come from the manifest content
    // they sign, so build manifests in order.
    let v1 = common::manifest_fixtures::signed_manifest_by(
        &alice,
        "downgrade-test",
        &["store:notes:read"],
    );
    let v2 = common::manifest_fixtures::signed_manifest_by(
        &alice,
        "downgrade-test-v2",
        &["store:notes:read", "store:notes:write"],
    );
    let v3 = common::manifest_fixtures::signed_manifest_by(
        &alice,
        "downgrade-test-v3",
        &[
            "store:notes:read",
            "store:notes:write",
            "store:plugins:read",
        ],
    );
    let v2_fork = common::manifest_fixtures::signed_manifest_by(
        &alice,
        "downgrade-test-v2-fork",
        &["store:notes:read", "store:contacts:read"],
    );

    let mut chain = DagVersionChain::new(v1.content_cid);
    chain
        .add_version(v1.content_cid, v2.content_cid)
        .expect("v1 → v2 OK");
    chain
        .add_version(v2.content_cid, v3.content_cid)
        .expect("v2 → v3 OK");
    chain
        .add_version(v1.content_cid, v2_fork.content_cid)
        .expect("v1 → v2-fork OK");

    // Pre-install v3 = CURRENT (no version_chain or prior_installed_cid
    // — fresh install).
    let bytes_v3 = serde_ipld_dagcbor::to_vec(&v3).expect("encode");
    let install_v3 = common::manifest_fixtures::signed_install_record(
        &user_kp,
        v3.content_cid,
        benten_id::did::Did::from_string_unchecked("did:key:z6MkUpgradeV3".to_string()),
        3,
    );

    let mut library = PluginLibrary::new();
    let mut store = benten_id::plugin_did::PluginDidStore::new();
    let mut cascade = InMemoryInstallCascade::new();
    let mut private_ns = InMemoryInstallCascade::new();
    let trust_list: Vec<benten_id::did::Did> = vec![];
    {
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
        install_plugin(
            &mut library,
            &mut store,
            &mut ctx,
            &bytes_v3,
            &v3.content_cid,
            &install_v3,
            1,
            &|_| None,
        )
        .expect("fresh install of v3 admits");
    }
    assert_eq!(library.len(), 1);

    // ATTACK: attempt to "upgrade" to v1 (an ANCESTOR of v3).
    let bytes_v1 = serde_ipld_dagcbor::to_vec(&v1).expect("encode");
    let install_v1 = common::manifest_fixtures::signed_install_record(
        &user_kp,
        v1.content_cid,
        benten_id::did::Did::from_string_unchecked("did:key:z6MkUpgradeV1Attack".to_string()),
        4,
    );
    let mut cascade2 = InMemoryInstallCascade::new();
    let mut private_ns2 = InMemoryInstallCascade::new();
    let mut ctx_downgrade = InstallContext {
        cap_minter: &mut cascade2,
        private_ns: &mut private_ns2,
        now_secs: 1_700_000_000,
        installer_shape: InstallerShape::FullPeer,
        user_trust_list: &trust_list,
        user_did: &user_did,
        version_chain: Some(&chain),
        prior_installed_cid: Some(v3.content_cid),
    };
    let downgrade_attempt = install_plugin(
        &mut library,
        &mut store,
        &mut ctx_downgrade,
        &bytes_v1,
        &v1.content_cid,
        &install_v1,
        2,
        &|_| None,
    );
    let err = downgrade_attempt
        .expect_err("T10-upgrade (b): downgrade to non-descendant MUST be REJECTED");
    assert_eq!(
        err,
        ErrorCode::PluginManifestInvalid,
        "T10-upgrade (b): downgrade rejection MUST surface typed code; got {err:?}"
    );
    // Defense-in-depth: library state unchanged (still has v3).
    assert_eq!(
        library.len(),
        1,
        "rejected downgrade MUST NOT alter library state"
    );
    assert!(
        library.get(&v3.content_cid).is_some(),
        "v3 entry MUST remain"
    );
    assert!(
        library.get(&v1.content_cid).is_none(),
        "rejected v1 MUST NOT have been inserted"
    );

    // ATTACK 2: cross-branch upgrade to v2-fork (NOT a descendant of v3).
    let bytes_fork = serde_ipld_dagcbor::to_vec(&v2_fork).expect("encode");
    let install_fork = common::manifest_fixtures::signed_install_record(
        &user_kp,
        v2_fork.content_cid,
        benten_id::did::Did::from_string_unchecked("did:key:z6MkUpgradeForkAttack".to_string()),
        5,
    );
    let mut cascade3 = InMemoryInstallCascade::new();
    let mut private_ns3 = InMemoryInstallCascade::new();
    let mut ctx_fork = InstallContext {
        cap_minter: &mut cascade3,
        private_ns: &mut private_ns3,
        now_secs: 1_700_000_000,
        installer_shape: InstallerShape::FullPeer,
        user_trust_list: &trust_list,
        user_did: &user_did,
        version_chain: Some(&chain),
        prior_installed_cid: Some(v3.content_cid),
    };
    let fork_attempt = install_plugin(
        &mut library,
        &mut store,
        &mut ctx_fork,
        &bytes_fork,
        &v2_fork.content_cid,
        &install_fork,
        3,
        &|_| None,
    );
    assert!(
        fork_attempt.is_err(),
        "T10-upgrade (b): cross-fork upgrade MUST be rejected"
    );

    // POSITIVE arm — legitimate v3 → v3 same-CID re-install OR upgrade
    // to a fresh descendant (we'd need v4 in chain to test that; this
    // path validates the "same CID = re-install" branch).
    let mut cascade4 = InMemoryInstallCascade::new();
    let mut private_ns4 = InMemoryInstallCascade::new();
    let install_v3_redo = common::manifest_fixtures::signed_install_record(
        &user_kp,
        v3.content_cid,
        benten_id::did::Did::from_string_unchecked("did:key:z6MkUpgradeV3Redo".to_string()),
        6,
    );
    let mut ctx_same = InstallContext {
        cap_minter: &mut cascade4,
        private_ns: &mut private_ns4,
        now_secs: 1_700_000_000,
        installer_shape: InstallerShape::FullPeer,
        user_trust_list: &trust_list,
        user_did: &user_did,
        version_chain: Some(&chain),
        prior_installed_cid: Some(v3.content_cid),
    };
    let same_attempt = install_plugin(
        &mut library,
        &mut store,
        &mut ctx_same,
        &bytes_v3,
        &v3.content_cid,
        &install_v3_redo,
        4,
        &|_| None,
    );
    assert!(
        same_attempt.is_ok(),
        "T10-upgrade (b) boundary: same-CID re-install MUST admit (not over-strict): {same_attempt:?}"
    );
}
