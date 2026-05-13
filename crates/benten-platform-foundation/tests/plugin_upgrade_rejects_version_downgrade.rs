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

use common::manifest_fixtures::{stub_peer_did_alice, stub_plugin_did, stub_user_did};

#[ignore = "RED-PHASE-BODY: panic-stub body needs substantive G24-D-FP / wave-N rewrite against landed API surface"]
#[test]
fn plugin_upgrade_rejects_cid_not_a_dag_descendant_of_installed_version() {
    let _plugin = stub_plugin_did();
    let _user = stub_user_did();

    // G24-D-FP-1 wave wires this. Substantive shape:
    //
    //   use benten_platform_foundation::plugin_lifecycle::upgrade_plugin;
    //
    //   let mut engine = common::manifest_fixtures::test_engine_with_user_did();
    //   let plugin_did = stub_plugin_did();
    //   let alice = stub_peer_did_alice();
    //
    //   // Plugin version DAG:
    //   //   v1 (anchor) → v2 → v3 (installed; CURRENT)
    //   //                  ↘ v2-fork (separate branch)
    //   let v1 = common::manifest_fixtures::stub_cid_zero();
    //   let v2 = common::manifest_fixtures::stub_cid_one();
    //   let v3 = common::manifest_fixtures::stub_cid_two();
    //
    //   // Install v3 from alice; DAG: v1 → v2 → v3.
    //   common::manifest_fixtures::install_plugin_with_version_chain(
    //       &mut engine, plugin_did.clone(),
    //       /* peer_did */ alice.clone(),
    //       /* dag_chain */ vec![v1, v2, v3],
    //       /* current */ v3,
    //   ).unwrap();
    //
    //   // Attack: deliver an "upgrade" CID that's an ANCESTOR (v1)
    //   // — a known-vulnerable older version. Same peer-DID (alice).
    //   let downgrade_attempt = upgrade_plugin(
    //       &mut engine,
    //       /* plugin_did */ plugin_did.clone(),
    //       /* new_content_cid */ v1,
    //       /* signing_peer_did */ alice.clone(),
    //       /* signature */
    //       common::manifest_fixtures::valid_signature_by(&alice, &v1),
    //   );
    //
    //   // T10-upgrade (b) defense: downgrade to NON-DESCENDANT MUST
    //   // be REJECTED. CURRENT can advance to any reachable descendant
    //   // per D-4F-14; v1 is an ancestor, not a descendant of v3.
    //   let err = downgrade_attempt.expect_err(
    //       "T10-upgrade (b): upgrade to CID that's NOT a DAG descendant \
    //        of installed CURRENT MUST be REJECTED — version chain \
    //        monotonicity per D-4F-14"
    //   );
    //   assert!(
    //       matches!(err.code(),
    //           ErrorCode::E_PLUGIN_MANIFEST_INVALID
    //           | ErrorCode::E_PLUGIN_INSTALL_CONSENT_REQUIRED),
    //       "T10-upgrade (b): must surface typed downgrade rejection; \
    //        got {:?}", err.code()
    //   );
    //
    //   // Defense-in-depth: CURRENT pointer UNCHANGED:
    //   let current = engine.manifest_store().current_cid_for(&plugin_did);
    //   assert_eq!(current, v3,
    //       "T10-upgrade (b): rejected downgrade MUST NOT mutate \
    //        CURRENT pointer");
    //
    //   // Cross-branch fork upgrade also rejected (v2-fork is NOT a
    //   // descendant of v3):
    //   let v2_fork = common::manifest_fixtures::stub_cid_v2_fork();
    //   let fork_attempt = upgrade_plugin(
    //       &mut engine,
    //       plugin_did.clone(),
    //       v2_fork,
    //       alice.clone(),
    //       common::manifest_fixtures::valid_signature_by(&alice, &v2_fork),
    //   );
    //   assert!(
    //       fork_attempt.is_err(),
    //       "T10-upgrade (b): cross-fork upgrade MUST be rejected at \
    //        upgrade surface — per CLAUDE.md #18 cross-fork merge \
    //        requires user-initiated merge flow (ratification #8)"
    //   );
    //
    // OBSERVABLE consequence: DAG-monotonicity enforcement at upgrade
    // boundary; cross-fork transitions surfaced as merge-not-upgrade.
    panic!(
        "RED-PHASE: G24-D-FP-1 must wire DAG-descendant check in \
         upgrade_plugin (T10-upgrade (b) per-finding-granular pin). \
         Substantive: v1→v2→v3 chain + downgrade-to-v1-rejected + \
         CURRENT unchanged + cross-fork rejected."
    );
}
