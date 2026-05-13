//! Phase-4-Foundation R4-FP-1 — T11 LOAD-BEARING pin: admin UI v0
//! install without clock injection surfaces E_UCAN_CLOCK_NOT_INJECTED.
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §2 MAJOR row
//! r4-tc-4 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md`
//! §T11 ("Wallclock fail-closed posture inheritance") + sec-3.5-r1-7
//! carry + D-4F-15 transparent-clock-injection ratification.
//!
//! ## What this pin establishes
//!
//! Per threat-model §T11 + sec-3.5-r1-7 carry: "Phase-3 G16-B-B-rest
//! (PR #158) removed `DEFAULT_NOW_SECS` constant fallback at
//! `UcanGroundedPolicy`; any code path that constructs a
//! `UcanGroundedPolicy`-evaluating chain WITHOUT injecting a wall-clock
//! surfaces typed `E_UCAN_CLOCK_NOT_INJECTED` instead of silently using
//! clock=0 (which falsely admitted expired UCANs). Admin UI v0 install
//! + materializer + every cap-evaluating path MUST inherit this
//! discipline."
//!
//! Per D-4F-15 transparent-clock-injection: install-record + content-
//! signature verification requires injected clock; engine-side at
//! `ManifestStore::load_verified`; plugin authors do NOT thread clock
//! themselves.
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer wires install flow but forgets to thread injected
//! clock. Test sees install succeed when clock missing — silent
//! fail-OPEN that falsely admits expired UCANs in install record's
//! consent chain.

#![allow(clippy::unwrap_used)]

mod common;

use common::manifest_fixtures::{stub_plugin_did, stub_user_did};

#[ignore = "RED-PHASE (Phase 4-Foundation R5 G24-D-FP-1 wave un-ignores) — \
    T11 LOAD-BEARING admin UI v0 install fail-closed clock-injection arm; routes \
    through install_plugin's UCAN-consent-chain validation; surfaces typed \
    E_UCAN_CLOCK_NOT_INJECTED when engine built without clock injection. Couples \
    to D-4F-15 transparent-clock-injection ratification + install_plugin lifecycle \
    hardening. Named destination: plan §3 G24-D-FP-1 (plugin_lifecycle + engine \
    clock-injection seam at install boundary). HARD RULE 12 clause-(b) BELONGS-NAMED-NOW."]
#[test]
fn admin_ui_v0_install_without_clock_injection_surfaces_e_ucan_clock_not_injected() {
    let _plugin = stub_plugin_did();
    let _user = stub_user_did();

    // G24-D wave wires this. Substantive shape:
    //
    //   use benten_platform_foundation::plugin_lifecycle::install_plugin;
    //   use benten_platform_foundation::plugin_manifest::ManifestStore;
    //
    //   // Engine built WITHOUT clock injection (omit the .with_clock(...)
    //   // builder call). Per Phase-3 PR #158 + D-4F-15, this MUST cause
    //   // install_plugin's UCAN-evaluating path to fail-closed with
    //   // typed E_UCAN_CLOCK_NOT_INJECTED.
    //   let mut engine = common::manifest_fixtures::
    //       test_engine_without_clock_injection();
    //
    //   let manifest = common::manifest_fixtures::admin_ui_v0_manifest();
    //   let install_record = common::manifest_fixtures::
    //       stub_install_record(common::manifest_fixtures::stub_cid_zero());
    //
    //   let attempt = install_plugin(&mut engine, manifest, install_record);
    //
    //   let err = attempt.expect_err(
    //       "T11 LOAD-BEARING: admin UI v0 install WITHOUT injected \
    //        clock MUST fail-closed with typed E_UCAN_CLOCK_NOT_INJECTED \
    //        — silent fail-OPEN falsely admits expired UCANs in the \
    //        consent chain"
    //   );
    //   assert!(
    //       matches!(err.code(), ErrorCode::E_UCAN_CLOCK_NOT_INJECTED),
    //       "T11: must surface typed E_UCAN_CLOCK_NOT_INJECTED \
    //        (Phase-3 PR #158 invariant); got {:?}", err.code()
    //   );
    //
    //   // Defense-in-depth: install state UNCHANGED — fail-closed
    //   // discipline means no partial state commits:
    //   let installed = engine.manifest_store().installed_plugins();
    //   assert!(installed.is_empty(),
    //       "T11: fail-closed install MUST NOT commit partial state");
    //
    //   // OK arm boundary: same install WITH clock injection succeeds
    //   // (defense isn't over-strict; sec-3.5-r1-7 carry must thread
    //   // properly):
    //   let mut engine_with_clock = common::manifest_fixtures::
    //       test_engine_with_clock_injection(/* now_secs */ 1_700_000_000);
    //   let install2 = install_plugin(
    //       &mut engine_with_clock,
    //       common::manifest_fixtures::admin_ui_v0_manifest(),
    //       common::manifest_fixtures::stub_install_record(
    //           common::manifest_fixtures::stub_cid_zero()
    //       ),
    //   );
    //   assert!(install2.is_ok(),
    //       "T11 boundary: install WITH injected clock must succeed; \
    //        sec-3.5-r1-7 carry isn't over-strict");
    //
    //   // D-4F-15 transparent-clock-injection: plugin authors do NOT
    //   // thread clock themselves; the manifest doesn't carry a
    //   // clock-injection field:
    //   let manifest_fields = std::mem::size_of::<
    //       benten_platform_foundation::PluginManifest
    //   >();
    //   // Manifest struct has no clock_injection field (compile-time
    //   // structural assertion via field count or named-field check).
    //   let _ = manifest_fields; // sentinel for substantive walk
    //
    // OBSERVABLE consequence: clock-injection failure surfaces typed
    // error at engine boundary; transparent to plugin authors.
    panic!(
        "RED-PHASE: G24-D must wire admin UI v0 install clock-injection \
         requirement (T11 LOAD-BEARING). Substantive: missing-clock-\
         fail-closed + typed E_UCAN_CLOCK_NOT_INJECTED + no-partial-\
         state-commit + with-clock-OK boundary + transparent-injection."
    );
}
