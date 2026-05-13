//! Phase-4-Foundation R4-FP-1 — T10-upgrade (a) pin: plugin upgrade
//! requires same peer-DID author (peer-DID change at upgrade = re-install).
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §1 BLOCKER row
//! r4-tc-3 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md`
//! §T10 defense step 4(a) + plan §3 G24-D-FP-1.
//!
//! ## What this pin establishes
//!
//! Per threat-model §T10 defense step 4(a): "peer-DID signature
//! verification on the new content against the previously-installed
//! peer-DID (peer-DID change at upgrade is treated as a re-install —
//! receiver-consent again)."
//!
//! Upgrade with DIFFERENT peer-DID author is NOT a silent upgrade —
//! it's a substitution attack class (per T5b). Without this defense,
//! attacker takes over a plugin's slot by offering an "upgrade"
//! signed by attacker's peer-DID.
//!
//! Per pim-2-amendment §3.6b sub-rule 4: T10-upgrade has multiple
//! sub-arms (a) same-author, (b) reject-downgrade. This pin is the
//! (a) arm only.
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer wires upgrade flow but skips peer-DID author identity
//! check. Attacker delivers an "upgrade" CID with different peer-DID
//! signature; upgrade flow accepts; admin UI is now signed by a
//! different peer-DID without re-consent.

#![allow(clippy::unwrap_used)]

mod common;

use common::manifest_fixtures::{
    stub_peer_did_alice, stub_peer_did_attacker, stub_plugin_did, stub_user_did,
};

#[test]
#[ignore = "RED-PHASE: G24-D-FP-1 wires same-author-DID check in upgrade_plugin; un-ignore at G24-D-FP-1 landing. Pin source: r4-triage §1 r4-tc-3 + threat-model §T10 step 4(a)."]
fn plugin_upgrade_with_different_peer_did_author_rejected_or_treated_as_reinstall() {
    let _plugin = stub_plugin_did();
    let _user = stub_user_did();

    // G24-D-FP-1 wave wires this. Substantive shape:
    //
    //   use benten_platform_foundation::plugin_lifecycle::upgrade_plugin;
    //
    //   let mut engine = common::manifest_fixtures::test_engine_with_user_did();
    //   let plugin_did = stub_plugin_did();
    //   let alice = stub_peer_did_alice();
    //   let attacker = stub_peer_did_attacker();
    //
    //   // Install v1 from peer-DID alice.
    //   common::manifest_fixtures::install_plugin_signed_by(
    //       &mut engine, plugin_did.clone(),
    //       /* peer_did */ alice.clone(),
    //       /* content_cid */ common::manifest_fixtures::stub_cid_zero(),
    //   ).unwrap();
    //
    //   // Attack: deliver an "upgrade" content signed by attacker's
    //   // peer-DID (not alice).
    //   let upgrade_attempt = upgrade_plugin(
    //       &mut engine,
    //       /* plugin_did */ plugin_did.clone(),
    //       /* new_content_cid */ common::manifest_fixtures::stub_cid_one(),
    //       /* signing_peer_did */ attacker.clone(),
    //       /* signature_bytes */ vec![0u8; 64],
    //   );
    //
    //   // T10-upgrade (a) defense: upgrade with different peer-DID is
    //   // REJECTED (or surfaces as a re-install requiring new consent).
    //   // The silent-upgrade path MUST NOT admit a different peer-DID.
    //   let err = upgrade_attempt.expect_err(
    //       "T10-upgrade (a): upgrade with different peer-DID author \
    //        MUST be REJECTED (silent upgrade only valid for same \
    //        peer-DID; cross-peer transition is re-install per \
    //        threat-model §T10 step 4(a))"
    //   );
    //   assert!(
    //       matches!(err.code(),
    //           ErrorCode::E_PLUGIN_CONTENT_PEER_SIGNATURE_INVALID
    //           | ErrorCode::E_PLUGIN_AUTHOR_NOT_TRUSTED
    //           | ErrorCode::E_PLUGIN_INSTALL_CONSENT_REQUIRED),
    //       "T10-upgrade (a): must surface typed denial; got {:?}",
    //       err.code()
    //   );
    //
    //   // Defense-in-depth: installed plugin's peer-DID UNCHANGED:
    //   let current = engine.manifest_store()
    //       .installed_peer_did_for(&plugin_did);
    //   assert_eq!(current, alice,
    //       "T10-upgrade (a): rejected upgrade MUST NOT mutate the \
    //        installed peer-DID author");
    //
    //   // Companion OK arm: upgrade with SAME peer-DID alice succeeds
    //   // (the regression-guard within this pin, since pim-2 sub-rule 4
    //   // pins the per-finding boundary):
    //   let same_author_upgrade = upgrade_plugin(
    //       &mut engine,
    //       plugin_did.clone(),
    //       common::manifest_fixtures::stub_cid_two(),
    //       /* signing_peer_did */ alice.clone(),
    //       /* valid signature from alice */
    //       common::manifest_fixtures::valid_signature_by(&alice, &common::manifest_fixtures::stub_cid_two()),
    //   );
    //   assert!(same_author_upgrade.is_ok(),
    //       "T10-upgrade (a) OK arm: same peer-DID + valid signature \
    //        MUST succeed (silent within-lineage upgrade per D-4F-12)");
    //
    // OBSERVABLE consequence: cross-peer transition forces re-consent
    // path; same-peer upgrade silent per CLAUDE.md baked-in #18.
    panic!(
        "RED-PHASE: G24-D-FP-1 must wire same-author-DID check in \
         upgrade_plugin (T10-upgrade (a) per-finding-granular pin). \
         Substantive: v1 install + cross-peer upgrade-rejected + \
         no-state-mutation + same-peer-upgrade-succeeds boundary."
    );
    #[allow(unreachable_code)]
    {
        let _ = stub_peer_did_attacker();
    }
}
