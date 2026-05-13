//! LOAD-BEARING per plan §3 G24-D row + post-R1-triage Q6 ratification.
//!
//! Plugin authors do NOT thread clock through their plugin code; the
//! engine surface injects clock at manifest-load. Plugins MAY override
//! for tests.
//!
//! Fail-closed if clock not injected (sec-3.5-r1-7):
//! `E_UCAN_CLOCK_NOT_INJECTED` (existing Phase-3 ErrorCode).
//!
//! **R4b-FP-1 Seam 2** — both arms un-ignored against the new
//! `PluginManifest::validate_with_clock(now_secs)` + `install_plugin`
//! seam in `plugin_lifecycle.rs`. The seam fail-closes when
//! `now_secs == MANIFEST_CLOCK_NOT_INJECTED_SENTINEL` AND the manifest
//! declares time-bounded requirements (e.g. `host:time:now`).

mod common;

use benten_errors::ErrorCode;
use benten_id::keypair::Keypair;
use benten_platform_foundation::plugin_library::PluginLibrary;
use benten_platform_foundation::plugin_lifecycle::{
    InMemoryInstallCascade, InstallContext, InstallerShape, install_plugin,
};
use benten_platform_foundation::plugin_manifest::MANIFEST_CLOCK_NOT_INJECTED_SENTINEL;

#[test]
fn manifest_validate_consults_engine_injected_clock_not_plugin_local_clock() {
    // R4b-FP-1 Seam 2 — direct test against
    // PluginManifest::validate_with_clock. Substantive per pim-2
    // §3.6b: builds two manifests (one with `host:time:now`, one
    // without); sentinel clock fail-closes the time-bounded one,
    // admits the unbounded one; with a real clock, both admit.
    let alice = Keypair::generate();

    // (1) Time-bounded manifest (declares host:time:now).
    let time_bounded = common::manifest_fixtures::signed_manifest_by(
        &alice,
        "time-bounded",
        &["host:time:now", "store:notes:read"],
    );
    assert!(
        time_bounded.declares_time_bounded(),
        "test scaffold: manifest should declare time-bounded behavior"
    );
    let err = time_bounded
        .validate_with_clock(MANIFEST_CLOCK_NOT_INJECTED_SENTINEL)
        .expect_err("clock-not-injected MUST fail-closed for time-bounded manifest");
    assert_eq!(
        err,
        ErrorCode::UcanClockNotInjected,
        "Seam 2 fail-closed: time-bounded manifest + sentinel clock MUST surface \
         typed E_UCAN_CLOCK_NOT_INJECTED; would-FAIL if seam skipped sentinel check"
    );
    // Boundary: same manifest with a real clock admits.
    time_bounded
        .validate_with_clock(1_700_000_000)
        .expect("time-bounded manifest WITH real clock MUST admit");

    // (2) Time-unbounded manifest (no host:time:* requires) — sentinel
    // clock is fine because no time-bounded behavior to validate.
    let unbounded =
        common::manifest_fixtures::signed_manifest_by(&alice, "unbounded", &["store:notes:read"]);
    assert!(
        !unbounded.declares_time_bounded(),
        "test scaffold: unbounded manifest should NOT declare time-bounded behavior"
    );
    unbounded
        .validate_with_clock(MANIFEST_CLOCK_NOT_INJECTED_SENTINEL)
        .expect(
            "time-unbounded manifest with sentinel clock MUST admit \
             (seam is fail-closed only for time-bounded manifests)",
        );
}

#[test]
fn admin_ui_v0_install_without_clock_injection_surfaces_e_ucan_clock_not_injected() {
    // R4b-FP-1 Seam 2 + Seam 1 integration — install_plugin lifecycle
    // threads `ctx.now_secs` into `validate_with_clock`. Substantive
    // per pim-2 §3.6b sub-rule 4: install of the admin-UI-v0-shaped
    // manifest (declares host:time:now) WITHOUT clock injection
    // surfaces typed E_UCAN_CLOCK_NOT_INJECTED at the install boundary.
    //
    // Would-FAIL-if-no-op'd: install_plugin skips validate_with_clock
    // OR uses a hardcoded clock instead of threading ctx.now_secs.
    let alice = Keypair::generate();
    let user_kp = Keypair::generate();
    let user_did = user_kp.public_key().to_did();

    // Manifest declares host:time:now → fail-closed on sentinel clock.
    let manifest = common::manifest_fixtures::signed_manifest_by(
        &alice,
        "admin-ui-v0-clock-injection",
        &[
            "host:time:now",
            "store:plugins:read",
            "private:admin-ui:logs",
        ],
    );
    let bytes = serde_ipld_dagcbor::to_vec(&manifest).expect("encode");
    let expected_cid = manifest.content_cid;

    // R6-FP-A-fp caller-mint-first.
    let mut store = benten_id::plugin_did::PluginDidStore::new();
    let plugin_did_clock = common::manifest_fixtures::mint_and_insert_plugin_did(&mut store);
    let install_record = common::manifest_fixtures::signed_install_record(
        &user_kp,
        expected_cid,
        plugin_did_clock.clone(),
        3,
    );

    let mut library = PluginLibrary::new();
    let mut cascade = InMemoryInstallCascade::new();
    let mut private_ns = InMemoryInstallCascade::new();
    let trust_list: Vec<benten_id::did::Did> = vec![];

    let mut ctx_no_clock = InstallContext {
        cap_minter: &mut cascade,
        private_ns: &mut private_ns,
        now_secs: MANIFEST_CLOCK_NOT_INJECTED_SENTINEL, // engine built WITHOUT clock injection
        installer_shape: InstallerShape::FullPeer,
        user_trust_list: &trust_list,
        user_did: &user_did,
        version_chain: None,
        prior_installed_cid: None,
        expected_plugin_did: &plugin_did_clock,
    };

    let attempt = install_plugin(
        &mut library,
        &mut store,
        &mut ctx_no_clock,
        &bytes,
        &expected_cid,
        &install_record,
        1,
        &|_| None,
    );

    let err = attempt.expect_err(
        "T11 LOAD-BEARING: admin UI v0 install WITHOUT injected clock MUST fail-closed",
    );
    assert_eq!(
        err,
        ErrorCode::UcanClockNotInjected,
        "T11: must surface typed E_UCAN_CLOCK_NOT_INJECTED (Phase-3 PR #158 invariant); got {err:?}"
    );

    // Defense-in-depth: fail-closed install MUST NOT commit partial
    // state — no library entry, no minted grants, no provisioned NS.
    assert!(
        library.is_empty(),
        "T11: fail-closed install MUST NOT commit library state"
    );
    assert!(
        cascade.minted_grants().is_empty(),
        "T11: fail-closed install MUST NOT mint root grants"
    );
    // R6-FP-A-fp: post-caller-mint-first, the caller pre-inserts the
    // handle BEFORE install_plugin. install_plugin itself no longer
    // mutates the store on either success OR fail. Assert the store
    // still holds exactly the pre-inserted handle (no extra).
    assert_eq!(
        store.len(),
        1,
        "T11: fail-closed install MUST NOT add anything to plugin_did_store \
         beyond the caller-pre-inserted handle"
    );

    // Boundary: same install path WITH a real clock injected MUST
    // admit (defense isn't over-strict; sec-3.5-r1-7 carry threads
    // properly).
    let mut library2 = PluginLibrary::new();
    let mut store2 = benten_id::plugin_did::PluginDidStore::new();
    let plugin_did_clock_b = common::manifest_fixtures::mint_and_insert_plugin_did(&mut store2);
    let install_record_b = common::manifest_fixtures::signed_install_record(
        &user_kp,
        expected_cid,
        plugin_did_clock_b.clone(),
        4,
    );
    let mut cascade2 = InMemoryInstallCascade::new();
    let mut private_ns2 = InMemoryInstallCascade::new();
    let mut ctx_with_clock = InstallContext {
        cap_minter: &mut cascade2,
        private_ns: &mut private_ns2,
        now_secs: 1_700_000_000, // real clock injected
        installer_shape: InstallerShape::FullPeer,
        user_trust_list: &trust_list,
        user_did: &user_did,
        version_chain: None,
        prior_installed_cid: None,
        expected_plugin_did: &plugin_did_clock_b,
    };
    let outcome = install_plugin(
        &mut library2,
        &mut store2,
        &mut ctx_with_clock,
        &bytes,
        &expected_cid,
        &install_record_b,
        1,
        &|_| None,
    )
    .expect("admin UI v0 install WITH injected clock MUST succeed");
    assert_eq!(outcome.grants_minted, 3);
    assert_eq!(library2.len(), 1);
}
