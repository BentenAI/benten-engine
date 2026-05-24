//! G24-D substantive pipeline end-to-end test.
//!
//! Phase-4-Meta-Core G-CORE-0 migration (plan §1.A.FROZEN item 7,
//! `docs/future/phase-4-backlog.md §4.33`, HARD-RULE-12 clause-(a)):
//! all arms previously importing the deleted
//! `module_ecosystem::{install_plugin, install_plugin_persisting_did}`
//! precursors are migrated to the canonical
//! `plugin_lifecycle::install_plugin` (the 11-step pipeline with full
//! Layer-1 cap cascade + Layer-2 consent + Layer-3 envelope per
//! CLAUDE.md #18). The non-install helpers — `decide_upgrade_consent`,
//! `verify_install_record`, `verify_upgrade_author_continuity` — stay
//! on `module_ecosystem` (their non-install surfaces were preserved
//! at G-CORE-0).
//!
//! Exercises the FULL plugin manifest lifecycle through real
//! signing / verification / install / library / DAG version chain
//! / cap-change-triggered upgrade-consent paths against the API
//! surface landed at G24-D canary + substantive impl.
//!
//! Per pim-2 §3.6b: this is the "exercises the SPECIFIC arm + observable
//! consequence + would-FAIL-if-no-op'd" durable pin for G24-D primary
//! scope. Family F3 R3 RED-PHASE pins are companion verification at
//! finer-grained substance; this test covers the umbrella pipeline.

use benten_caps::plugin_delegation::{
    DelegationDecision, SharesPolicyView, check_delegation_within_envelope,
    is_private_namespace_cap,
};
use benten_core::Cid;
use benten_core::version_chain::DagVersionChain;
use benten_errors::ErrorCode;
use benten_id::keypair::Keypair;
use benten_id::plugin_did::{PluginDidStore, mint as mint_plugin_did};
use benten_platform_foundation::module_ecosystem::{
    UpgradeConsentDecision, decide_upgrade_consent, verify_install_record,
    verify_upgrade_author_continuity,
};
use benten_platform_foundation::plugin_library::PluginLibrary;
use benten_platform_foundation::plugin_lifecycle::{
    InMemoryInstallCascade, InMemoryUninstallCascade, InstallParams, InstallPorts, InstallerShape,
    UninstallPorts, install_plugin, uninstall_plugin,
};
use benten_platform_foundation::plugin_manifest::{
    CapRequirement, InstallRecord, PluginManifest, RendererBackend, RendererConfig, SharesPolicy,
    SharesPolicyDefault, SharesRule, SharesTarget, detect_composition_cycle, sign_manifest,
};
use benten_platform_foundation::workflow_to_plugin::{WorkflowHandle, promote_workflow_to_plugin};

/// SharesPolicyView adapter for the manifest's SharesPolicy so the
/// caps-crate delegation check can be exercised from the
/// platform-foundation crate (without forcing a backward dep).
struct PolicyAdapter<'a>(&'a SharesPolicy);

impl<'a> SharesPolicyView for PolicyAdapter<'a> {
    fn permits(&self, cap_pattern: &str, target_plugin_did: &benten_id::did::Did) -> bool {
        self.0.permits_delegation(cap_pattern, target_plugin_did)
    }
}

fn build_manifest(
    plugin_name: &str,
    author_keypair: &Keypair,
    requires: Vec<CapRequirement>,
    shares: SharesPolicy,
) -> PluginManifest {
    let author_did = author_keypair.public_key().to_did();
    let mut manifest = PluginManifest {
        plugin_name: plugin_name.to_string(),
        // Placeholder; replaced by compute_content_cid below.
        content_cid: Cid::from_blake3_digest([0u8; 32]),
        peer_did: author_did,
        peer_signature: vec![0u8; 64],
        requires,
        shares,
        renderer_config: Some(RendererConfig {
            output_format: "html_json".to_string(),
            renderer_backends: Some(vec![RendererBackend::BrowserWasm32]),
            hosting_target: None,
            bundle_size_budget_kb: Some(256),
        }),
        composes_plugins: None,
        accepts_content: None,
        requires_schema_authors: None,
        requires_plugin_authors: None,
    };
    manifest.content_cid = manifest.compute_content_cid();
    manifest.peer_signature = sign_manifest(&manifest, author_keypair);
    manifest
}

/// G-CORE-0 helper: drive a single install through the canonical
/// `plugin_lifecycle::install_plugin` pipeline with caller-mint-first
/// plugin-DID + signed InstallRecord. Returns the freshly-minted
/// plugin-DID so callers can assert against it (parity with the
/// pre-deletion `install_plugin_persisting_did` return shape).
#[allow(clippy::too_many_arguments)]
fn drive_install(
    library: &mut PluginLibrary,
    store: &mut PluginDidStore,
    cascade: &mut InMemoryInstallCascade,
    private_ns: &mut InMemoryInstallCascade,
    user_kp: &Keypair,
    manifest_bytes: &[u8],
    cid: &Cid,
    installer_shape: InstallerShape,
    installed_at_nanos: u64,
    nonce_byte: u8,
) -> Result<benten_id::did::Did, ErrorCode> {
    let plugin_did = {
        let handle = mint_plugin_did();
        let did = handle.did().clone();
        store
            .insert(handle)
            .expect("test fixture inserts unique plugin-DID");
        did
    };
    let mut record = InstallRecord {
        manifest_cid: *cid,
        plugin_did: plugin_did.clone(),
        consenting_user_did: user_kp.public_key().to_did(),
        user_signature: vec![0u8; 64],
        timestamp_stub_nanos: 1_700_000_000_000_000_000,
        nonce: vec![nonce_byte; 16],
        granted_caps_bytes: vec![],
    };
    let payload = record.signing_payload();
    record.user_signature = user_kp.sign(&payload).to_bytes().to_vec();

    let user_did = user_kp.public_key().to_did();
    let mut noop_replay_check_1 = benten_platform_foundation::testing::noop_replay_check();
    let noauth_policy_1 = benten_platform_foundation::install_consent::AdmitAllInstallConsent;
    let mut ports = InstallPorts {
        cap_minter: cascade,
        private_ns,
        install_record_replay_check: &mut noop_replay_check_1,
        policy: &noauth_policy_1,
    };
    let params = InstallParams {
        now_secs: 1_700_000_000,
        installer_shape,
        user_trust_list: &[],
        user_did: &user_did,
        version_chain: None,
        prior_installed_cid: None,
        expected_plugin_did: &plugin_did,
    };
    install_plugin(
        library,
        store,
        &mut ports,
        &params,
        manifest_bytes,
        cid,
        &record,
        installed_at_nanos,
        &|_| None,
    )?;
    Ok(plugin_did)
}

#[test]
fn full_install_pipeline_real_signatures_succeeds() {
    let author = Keypair::generate();
    let user_kp = Keypair::generate();
    let manifest = build_manifest(
        "test-app",
        &author,
        vec![CapRequirement::new("store:notes:read")],
        SharesPolicy::none(),
    );

    // Real signature MUST verify (substance — not a stub).
    manifest
        .verify_content_cid_matches()
        .expect("content_cid round-trip");
    manifest
        .verify_peer_signature()
        .expect("peer signature verifies");

    let bytes = serde_ipld_dagcbor::to_vec(&manifest).expect("encode");
    let mut library = PluginLibrary::new();
    let mut store = PluginDidStore::new();
    let mut cascade = InMemoryInstallCascade::new();
    let mut private_ns = InMemoryInstallCascade::new();
    let cid = manifest.content_cid;

    drive_install(
        &mut library,
        &mut store,
        &mut cascade,
        &mut private_ns,
        &user_kp,
        &bytes,
        &cid,
        InstallerShape::FullPeer,
        1,
        1,
    )
    .expect("install succeeds");

    assert!(library.get(&cid).is_some());
    assert_eq!(library.active("test-app"), Some(&cid));
}

#[test]
fn install_pipeline_rejects_substituted_content() {
    let author = Keypair::generate();
    let attacker = Keypair::generate();
    let user_kp = Keypair::generate();
    let real = build_manifest(
        "victim",
        &author,
        vec![CapRequirement::new("store:notes:read")],
        SharesPolicy::none(),
    );

    // Attacker swaps the peer_did to their own but keeps the same CID;
    // signature verification fails because the new peer_did won't match
    // the old signature.
    let mut substituted = real.clone();
    substituted.peer_did = attacker.public_key().to_did();
    // CID recomputed since peer_did is part of the body.
    substituted.content_cid = substituted.compute_content_cid();

    let bytes = serde_ipld_dagcbor::to_vec(&substituted).expect("encode");
    let mut library = PluginLibrary::new();
    let mut store = PluginDidStore::new();
    let mut cascade = InMemoryInstallCascade::new();
    let mut private_ns = InMemoryInstallCascade::new();
    let result = drive_install(
        &mut library,
        &mut store,
        &mut cascade,
        &mut private_ns,
        &user_kp,
        &bytes,
        &substituted.content_cid,
        InstallerShape::FullPeer,
        1,
        2,
    );
    // Substitution caught: peer signature was over the ORIGINAL author's
    // peer_did, so swapping fails verify_peer_signature.
    let err = result.expect_err("substitution rejected");
    assert!(
        matches!(err, ErrorCode::PluginContentPeerSignatureInvalid),
        "got {err:?}"
    );
    assert!(library.is_empty(), "library remains empty on rejection");
}

#[test]
fn install_pipeline_rejects_thin_client_with_sandbox_exec() {
    let author = Keypair::generate();
    let user_kp = Keypair::generate();
    let manifest = build_manifest(
        "sandbox-app",
        &author,
        vec![CapRequirement::new("host:sandbox:exec")],
        SharesPolicy::none(),
    );

    let bytes = serde_ipld_dagcbor::to_vec(&manifest).expect("encode");
    let mut library = PluginLibrary::new();
    let mut store = PluginDidStore::new();
    let mut cascade = InMemoryInstallCascade::new();
    let mut private_ns = InMemoryInstallCascade::new();
    let result = drive_install(
        &mut library,
        &mut store,
        &mut cascade,
        &mut private_ns,
        &user_kp,
        &bytes,
        &manifest.content_cid,
        InstallerShape::ThinClient,
        1,
        3,
    );
    let err = result.expect_err("thin-client + sandbox rejected");
    assert!(
        matches!(err, ErrorCode::PluginHeterogeneityIncompatible),
        "got {err:?}"
    );
}

#[test]
fn install_record_signed_by_user_did_verifies() {
    let user = Keypair::generate();
    let plugin_handle = mint_plugin_did();
    let manifest_cid = Cid::from_blake3_digest([7u8; 32]);
    let nonce = vec![0xABu8; 16];

    let mut record = InstallRecord {
        manifest_cid,
        plugin_did: plugin_handle.did().clone(),
        consenting_user_did: user.public_key().to_did(),
        user_signature: Vec::new(),
        timestamp_stub_nanos: 1_700_000_000_000_000_000,
        nonce: nonce.clone(),
        granted_caps_bytes: vec![],
    };
    let sig = user.sign(&record.signing_payload());
    record.user_signature = sig.to_bytes().to_vec();

    record.verify_user_signature().expect("user signature ok");
    verify_install_record(&record).expect("install record verifies");

    // Tamper: swap nonce; signature must reject.
    let mut tampered = record.clone();
    tampered.nonce = vec![0xFFu8; 16];
    assert!(
        matches!(
            tampered.verify_user_signature(),
            Err(ErrorCode::PluginInstallRecordUserSignatureInvalid)
        ),
        "tampered nonce rejected"
    );
}

#[test]
fn private_namespace_cap_unconditionally_denied_cross_plugin() {
    let policy = SharesPolicy {
        default: SharesPolicyDefault::Any, // even "Any" doesn't override private
        rules: None,
    };
    let view = PolicyAdapter(&policy);
    let target = mint_plugin_did();
    let decision = check_delegation_within_envelope(
        "private:did:key:z6MkSourcePlugin:notes",
        target.did(),
        &view,
    );
    assert_eq!(decision, DelegationDecision::PrivateNamespaceForbidden);
    assert!(is_private_namespace_cap("private:did:key:z6MkSomePlugin:*"));
}

#[test]
fn shares_policy_runtime_delegation_within_envelope_admitted_outside_denied() {
    let target = mint_plugin_did();
    let policy = SharesPolicy {
        default: SharesPolicyDefault::Matching,
        rules: Some(vec![SharesRule {
            cap_pattern: "store:notes:read".to_string(),
            target: SharesTarget::PluginDid(target.did().clone()),
        }]),
    };
    let view = PolicyAdapter(&policy);
    assert_eq!(
        check_delegation_within_envelope("store:notes:read", target.did(), &view),
        DelegationDecision::Permitted
    );
    let other = mint_plugin_did();
    assert_eq!(
        check_delegation_within_envelope("store:notes:read", other.did(), &view),
        DelegationDecision::OutsideEnvelope
    );
    assert_eq!(
        check_delegation_within_envelope("store:notes:write", target.did(), &view),
        DelegationDecision::OutsideEnvelope
    );
}

#[test]
fn composition_cycle_rejected_at_install() {
    let author = Keypair::generate();
    let manifest_a = build_manifest(
        "a",
        &author,
        vec![CapRequirement::new("store:a:read")],
        SharesPolicy::none(),
    );
    let mut manifest_b = build_manifest(
        "b",
        &author,
        vec![CapRequirement::new("store:b:read")],
        SharesPolicy::none(),
    );
    manifest_b.composes_plugins = Some(vec![manifest_a.content_cid]);
    manifest_b.content_cid = manifest_b.compute_content_cid();
    manifest_b.peer_signature = sign_manifest(&manifest_b, &author);
    // Make A compose B → cycle (A→B→A).
    let mut manifest_a_cycle = manifest_a.clone();
    manifest_a_cycle.composes_plugins = Some(vec![manifest_b.content_cid]);
    manifest_a_cycle.content_cid = manifest_a_cycle.compute_content_cid();
    manifest_a_cycle.peer_signature = sign_manifest(&manifest_a_cycle, &author);

    let a_cid = manifest_a_cycle.content_cid;
    let b_cid = manifest_b.content_cid;
    let mb = manifest_b.clone();
    let resolver = |cid: &Cid| -> Option<PluginManifest> {
        if *cid == b_cid {
            Some(mb.clone())
        } else {
            None
        }
    };

    // Construct a cycle: a_cycle composes b, b composes a_cycle.
    let mut mb_cycle = manifest_b.clone();
    mb_cycle.composes_plugins = Some(vec![a_cid]);
    let mb_cycle_ref = mb_cycle.clone();
    let resolver_cycle = |cid: &Cid| -> Option<PluginManifest> {
        if *cid == b_cid {
            Some(mb_cycle_ref.clone())
        } else {
            None
        }
    };

    let result = detect_composition_cycle(a_cid, &manifest_a_cycle, &resolver_cycle);
    assert!(matches!(
        result,
        Err(ErrorCode::PluginMetaCompositionCycleRejected)
    ));

    // No cycle path
    let result_no_cycle = detect_composition_cycle(a_cid, &manifest_a_cycle, &resolver);
    assert!(result_no_cycle.is_ok());
}

#[test]
fn cap_change_triggered_consent_subset_silent_growth_consent_required() {
    let author = Keypair::generate();
    let old = build_manifest(
        "app",
        &author,
        vec![
            CapRequirement::new("store:notes:read"),
            CapRequirement::new("store:notes:write"),
        ],
        SharesPolicy::none(),
    );
    let new_subset = build_manifest(
        "app",
        &author,
        vec![CapRequirement::new("store:notes:read")],
        SharesPolicy::none(),
    );
    assert_eq!(
        decide_upgrade_consent(&old, &new_subset),
        UpgradeConsentDecision::Silent
    );

    let new_widened = build_manifest(
        "app",
        &author,
        vec![
            CapRequirement::new("store:notes:read"),
            CapRequirement::new("store:notes:write"),
            CapRequirement::new("host:time:now"),
        ],
        SharesPolicy::none(),
    );
    assert_eq!(
        decide_upgrade_consent(&old, &new_widened),
        UpgradeConsentDecision::ConsentRequired
    );
}

#[test]
fn upgrade_with_different_peer_did_rejected_as_reinstall() {
    let author_a = Keypair::generate();
    let author_b = Keypair::generate();
    let old = build_manifest(
        "app",
        &author_a,
        vec![CapRequirement::new("store:notes:read")],
        SharesPolicy::none(),
    );
    let new_diff_author = build_manifest(
        "app",
        &author_b,
        vec![CapRequirement::new("store:notes:read")],
        SharesPolicy::none(),
    );
    assert!(matches!(
        verify_upgrade_author_continuity(&old, &new_diff_author),
        Err(ErrorCode::PluginAuthorNotTrusted)
    ));
    // Same author continues smoothly.
    let new_same_author = build_manifest(
        "app",
        &author_a,
        vec![CapRequirement::new("store:notes:read")],
        SharesPolicy::none(),
    );
    assert!(verify_upgrade_author_continuity(&old, &new_same_author).is_ok());
}

#[test]
fn plugin_library_holds_all_versions_active_ref_per_name() {
    let author = Keypair::generate();
    let user_kp = Keypair::generate();
    let v1 = build_manifest(
        "app",
        &author,
        vec![CapRequirement::new("store:notes:read")],
        SharesPolicy::none(),
    );
    let v2 = build_manifest(
        "app",
        &author,
        vec![
            CapRequirement::new("store:notes:read"),
            CapRequirement::new("store:notes:write"),
        ],
        SharesPolicy::none(),
    );

    let mut library = PluginLibrary::new();
    let mut store = PluginDidStore::new();
    let mut cascade = InMemoryInstallCascade::new();
    let mut private_ns = InMemoryInstallCascade::new();
    let v1_cid = v1.content_cid;
    let v2_cid = v2.content_cid;
    let bytes_v1 = serde_ipld_dagcbor::to_vec(&v1).expect("encode");
    let bytes_v2 = serde_ipld_dagcbor::to_vec(&v2).expect("encode");

    drive_install(
        &mut library,
        &mut store,
        &mut cascade,
        &mut private_ns,
        &user_kp,
        &bytes_v1,
        &v1_cid,
        InstallerShape::FullPeer,
        1,
        4,
    )
    .expect("v1 ok");
    drive_install(
        &mut library,
        &mut store,
        &mut cascade,
        &mut private_ns,
        &user_kp,
        &bytes_v2,
        &v2_cid,
        InstallerShape::FullPeer,
        2,
        5,
    )
    .expect("v2 ok");

    // Both versions held in library.
    assert_eq!(library.len(), 2);
    assert!(library.get(&v1_cid).is_some());
    assert!(library.get(&v2_cid).is_some());

    // Active ref tracks most-recent install (v2).
    assert_eq!(library.active("app"), Some(&v2_cid));

    // versions_of enumerates both.
    let versions = library.versions_of("app");
    assert_eq!(versions.len(), 2);
}

#[test]
fn dag_version_chain_supports_branches_and_forks() {
    let root = Cid::from_blake3_digest([0u8; 32]);
    let v1 = Cid::from_blake3_digest([1u8; 32]);
    let v2_main = Cid::from_blake3_digest([2u8; 32]);
    let v2_fork = Cid::from_blake3_digest([3u8; 32]);
    let v3_merge = Cid::from_blake3_digest([4u8; 32]);

    let mut dag = DagVersionChain::new(root);
    dag.add_version(root, v1).unwrap();
    dag.add_version(v1, v2_main).unwrap();
    dag.add_version(v1, v2_fork).unwrap();
    // Merge node has two parents.
    dag.add_version(v2_main, v3_merge).unwrap();
    dag.add_version(v2_fork, v3_merge).unwrap();

    assert!(dag.is_ancestor_of(&v1, &v3_merge));
    assert!(dag.is_descendant_of(&v3_merge, &root));

    // Tip is v3_merge after merge collapses the two parallel branches.
    let tips = dag.tips();
    assert_eq!(tips, vec![v3_merge]);

    // CURRENT pointer is per-device-local (per ratification #2).
    dag.set_current(v3_merge).unwrap();
    assert_eq!(dag.current(), Some(&v3_merge));
}

#[test]
fn workflow_promoted_to_plugin_via_manifest_addition() {
    let author = Keypair::generate();
    let workflow_cid = Cid::from_blake3_digest([42u8; 32]);
    let workflow = WorkflowHandle {
        subgraph_cid: workflow_cid,
        name: "my-workflow".to_string(),
    };
    let manifest = promote_workflow_to_plugin(
        &workflow,
        author.public_key().to_did(),
        vec![CapRequirement::new("store:notes:read")],
        SharesPolicy::none(),
    );
    // Manifest carries the workflow's name + author.
    assert_eq!(manifest.plugin_name, "my-workflow");
    assert_eq!(manifest.peer_did, author.public_key().to_did());
}

#[test]
fn uninstall_removes_library_entry_and_revokes_plugin_did() {
    let author = Keypair::generate();
    let user_kp = Keypair::generate();
    let manifest = build_manifest(
        "doomed",
        &author,
        vec![CapRequirement::new("store:notes:read")],
        SharesPolicy::none(),
    );
    let cid = manifest.content_cid;
    let bytes = serde_ipld_dagcbor::to_vec(&manifest).expect("encode");
    let mut library = PluginLibrary::new();
    let mut store = PluginDidStore::new();
    let mut cascade = InMemoryInstallCascade::new();
    let mut private_ns = InMemoryInstallCascade::new();

    // G-CORE-0: the canonical `plugin_lifecycle::install_plugin`
    // already persists the minted plugin-DID handle into the store
    // atomically (Step 8 caller-mint-first contract) — so the
    // subsequent uninstall path's PluginDidStore::revoke call
    // substantively succeeds. This supersedes the deleted
    // `install_plugin_persisting_did` precursor (G24-D-FP-1 ergonomic
    // seam that the canonical pipeline now subsumes).
    let plugin_did = drive_install(
        &mut library,
        &mut store,
        &mut cascade,
        &mut private_ns,
        &user_kp,
        &bytes,
        &cid,
        InstallerShape::FullPeer,
        1,
        6,
    )
    .expect("install ok");
    // Baseline: store carries the plugin-DID.
    assert!(
        store.get(&plugin_did).is_some(),
        "Baseline: install_plugin persists the handle (Step 8 caller-mint-first)"
    );

    let mut uninstall_cascade = InMemoryUninstallCascade::new();
    let mut uninstall_private = InMemoryUninstallCascade::new();
    let mut uninstall_subs = InMemoryUninstallCascade::new();
    let mut ctx = UninstallPorts {
        cap_revoker: &mut uninstall_cascade,
        private_ns: &mut uninstall_private,
        subscriptions: &mut uninstall_subs,
    };
    let outcome = uninstall_plugin(&mut library, &mut store, &mut ctx, &cid).expect("uninstall ok");
    assert!(outcome.library_entry_removed);
    assert!(
        outcome.plugin_did_revoked,
        "G24-D-FP-1 substantive: plugin_did_revoked MUST be true since \
         the canonical install pipeline persists the handle"
    );
    assert!(library.get(&cid).is_none());
    assert_eq!(library.active("doomed"), None);
    assert!(
        store.get(&plugin_did).is_none(),
        "G24-D-FP-1: store no longer carries the revoked plugin-DID"
    );
}
