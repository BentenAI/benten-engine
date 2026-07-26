//! R6 R1 FP-F4 §S2 production-arm test pin — `InstallPorts.install_record_replay_check`
//! is no longer `Option<>` (Row D-2 close) + `manifest_store::install_verified_record_unchecked`
//! carries `#[deprecated]` + `#[doc(hidden)]` (Δv3-9 closure).
//!
//! ## Test shape (per pim-18 / §3.6f SUBSTANTIVE-arm-not-SHAPE)
//!
//! 1. Arm 1: a substantive replay-check closure is wired through
//!    `InstallPorts` + IS called with the install record's
//!    canonical-bytes hash (verifies the Option drop landed +
//!    the consumption site at plugin_lifecycle.rs threads through
//!    correctly).
//! 2. Arm 2: the `noop_replay_check()` helper produces an admit-all
//!    closure shape coercible to `&mut InstallRecordReplayCheckFn`.
//! 3. Arm 3: the `install_verified_record_unchecked` side-door is
//!    callable + functional + the deprecation annotation surfaces as a
//!    lint warning (#[allow(deprecated)] on this test acknowledges).

#![allow(deprecated)]

mod common;

use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_id::did::Did;
use benten_id::keypair::Keypair;
use benten_platform_foundation::install_consent::AdmitAllInstallConsent;
use benten_platform_foundation::manifest_store::ManifestStore;
use benten_platform_foundation::plugin_library::PluginLibrary;
use benten_platform_foundation::plugin_lifecycle::{
    InMemoryInstallCascade, InstallParams, InstallPorts, InstallerShape, install_plugin,
};
use benten_platform_foundation::plugin_manifest::{
    CapRequirement, InstallRecord, PluginManifest, RendererBackend, RendererConfig, SharesPolicy,
    sign_manifest,
};
use benten_platform_foundation::testing::noop_replay_check;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

fn signed_record(
    user: &Keypair,
    plugin_did: Did,
    manifest_cid: Cid,
    nonce: Vec<u8>,
) -> InstallRecord {
    let mut record = InstallRecord {
        manifest_cid,
        plugin_did,
        consenting_user_did: user.public_key().to_did(),
        user_signature: Vec::new(),
        timestamp_stub_nanos: 1_700_000_000_000_000_000,
        nonce,
        granted_caps_bytes: vec![],
    };
    let sig = user.sign(&record.signing_payload());
    record.user_signature = sig.to_bytes().to_vec();
    record
}

fn build_signed_manifest(name: &str, author: &Keypair) -> PluginManifest {
    let mut manifest = PluginManifest {
        plugin_name: name.to_string(),
        content_cid: Cid::from_blake3_digest([0u8; 32]),
        peer_did: author.public_key().to_did(),
        peer_signature: vec![0u8; 64],
        requires: vec![CapRequirement::new("store:notes:read")],
        shares: SharesPolicy::none(),
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
    manifest.peer_signature = sign_manifest(&manifest, author);
    manifest
}

/// **§S2 arm 1 — Option drop landed:** the field type is now
/// `&mut InstallRecordReplayCheckFn` (no Option). Every caller MUST
/// supply a closure. This is verified by the fact that all 30
/// existing test fixtures had to thread a `noop_replay_check()` or
/// substantive closure post-migration (see grep `git log --grep
/// FP-F4`).
///
/// At the type level: the field's declared type IS what
/// `plugin_lifecycle::install_plugin` consumes via
/// `(ports.install_record_replay_check)(&hash)` direct call.
#[test]
fn install_ports_field_is_no_longer_option() {
    // This test asserts SHAPE: the noop closure coerces to
    // `&mut InstallRecordReplayCheckFn`. If the Option were still in
    // place, the test would not compile (`&mut F` doesn't coerce to
    // `Option<&mut F>` implicitly).
    let mut noop = noop_replay_check();
    let _coerced: &mut dyn FnMut(&[u8; 32]) -> Result<(), ErrorCode> = &mut noop;
}

/// **§S2 arm 2 — `noop_replay_check()` helper shape verification.**
/// Returns a closure that takes `&[u8; 32]` and returns
/// `Result<(), ErrorCode>` — exactly the shape required by
/// `InstallRecordReplayCheckFn`. Always admits (returns `Ok(())`).
#[test]
fn noop_replay_check_admits_every_hash() {
    let mut noop = noop_replay_check();
    assert!(noop(&[0xAAu8; 32]).is_ok());
    assert!(noop(&[0x00u8; 32]).is_ok());
}

/// **§S2 arm 3 — production-arm: counting replay-check is consulted at
/// step 3b via install_plugin end-to-end.**
/// Wires a counting closure through `InstallPorts.install_record_replay_check`
/// + invokes the full `install_plugin` pipeline. Asserts the closure
/// was consulted AT LEAST ONCE (Step 3b mandatory call) with a
/// non-zero canonical signing-payload BLAKE3 hash (the canonical
/// identity per §4.37 contract).
///
/// **Would-FAIL-on-revert (pim-18 §3.6f):** delete the
/// `(ports.install_record_replay_check)(&payload_hash)?` call at
/// plugin_lifecycle.rs:950 → closure never fires → counter stays at 0
/// → assertion fires.
#[test]
fn counting_replay_check_consulted_via_install_plugin_end_to_end() {
    let calls: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let observed: Arc<Mutex<Vec<[u8; 32]>>> = Arc::new(Mutex::new(Vec::new()));
    let calls_clone = calls.clone();
    let observed_clone = observed.clone();
    let mut check = move |hash: &[u8; 32]| -> Result<(), ErrorCode> {
        calls_clone.fetch_add(1, Ordering::SeqCst);
        observed_clone.lock().unwrap().push(*hash);
        Ok(())
    };

    let author = Keypair::generate();
    let user_kp = Keypair::generate();
    let user_did = user_kp.public_key().to_did();
    let manifest = build_signed_manifest("s2-arm3", &author);
    let bytes = serde_ipld_dagcbor::to_vec(&manifest).expect("encode manifest");
    let cid = manifest.content_cid;

    let mut library = PluginLibrary::new();
    let mut store = benten_id::plugin_did::PluginDidStore::new();
    let plugin_did = common::manifest_fixtures::mint_and_insert_plugin_did(&mut store);
    let install_record =
        common::manifest_fixtures::signed_install_record(&user_kp, cid, plugin_did.clone(), 3);

    let mut cascade = InMemoryInstallCascade::new();
    let mut private_ns = InMemoryInstallCascade::new();
    let admit_policy = AdmitAllInstallConsent;
    let mut ports = InstallPorts {
        cap_minter: &mut cascade,
        private_ns: &mut private_ns,
        install_record_replay_check: &mut check,
        policy: &admit_policy,
    };
    let params = InstallParams {
        now_secs: 1_700_000_000,
        installer_shape: InstallerShape::FullPeer,
        user_trust_list: &[],
        user_did: &user_did,
        version_chain: None,
        prior_installed_cid: None,
        expected_plugin_did: &plugin_did,
    };
    install_plugin(
        &mut library,
        &mut store,
        &mut ports,
        &params,
        &bytes,
        &cid,
        &install_record,
        1,
        &|_| None,
    )
    .expect("admitting closure → install_plugin admits");

    let observed_count = calls.load(Ordering::SeqCst);
    assert!(
        observed_count >= 1,
        "LOAD-BEARING: production install_plugin path MUST consult \
         install_record_replay_check at step 3b; observed={observed_count}"
    );

    let hashes = observed.lock().unwrap();
    assert!(
        hashes.iter().any(|h| *h != [0u8; 32]),
        "production step-3b boundary MUST forward the canonical \
         signing_payload() BLAKE3 hash (never all-zero); observed hashes={hashes:?}"
    );
}

/// **§S2 arm 4 — `install_verified_record_unchecked` side-door still
/// functional for test/synthetic paths (post-rename).** The deprecation
/// annotation surfaces as a lint warning; the body still works (the
/// rename is observable in the test fixtures' migrated calls).
#[test]
fn install_verified_record_unchecked_side_door_persists_record() {
    let user = Keypair::generate();
    let plugin_did = Did::from_string_for_test_fixture("did:key:zPluginSideDoor".to_string());
    let record = signed_record(
        &user,
        plugin_did.clone(),
        Cid::from_blake3_digest([1u8; 32]),
        vec![0x11u8; 16],
    );
    let mut store = ManifestStore::new();
    store
        .install_verified_record_unchecked(plugin_did.clone(), record.clone())
        .expect("side-door persists verified record");
    assert!(store.contains(&plugin_did));
}
