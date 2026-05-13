//! Phase-4-Foundation R4-FP-1 — T5a LOAD-BEARING pin: post-install
//! install-record drift detected at load-verified.
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §2 MAJOR row
//! r4-tc-4 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md` §T5
//! ("Plugin manifest envelope integrity") + defense step 1 (sec-4f-r1-9
//! verify-on-every-load).
//!
//! ## What this pin establishes
//!
//! Per threat-model §T5a + defense step 1: "Install record verified
//! on EVERY load, not just at install — (i) at engine boot, (ii) at
//! per-plugin load on first access, (iii) at per-Atrium-merge
//! boundary." Attacker swaps install-record bytes post-install
//! (writes to manifest store; restarts engine); new install record
//! has wider `requires` consent. User never re-consented.
//!
//! New seam: `crates/benten-platform-foundation/src/plugin_manifest.rs::
//! ManifestStore::load_verified(plugin_did) -> Result<InstallRecord>`.
//! Verifies: (a) user-DID signature on install record, (b) install-
//! record CID matches stored expected CID, (c) peer-DID signature on
//! plugin content matches what install record consented to.
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer wires verification at install only. Manifest-store
//! bytes mutated post-install (file system attack); next engine boot
//! loads the mutated install record without re-verifying; widened
//! `requires` envelope is silently accepted. Layer 2 consent
//! guarantee broken.

#![allow(clippy::unwrap_used)]

mod common;

use common::manifest_fixtures::{stub_plugin_did, stub_user_did};

#[ignore = "RED-PHASE-BODY: panic-stub body needs substantive G24-D-FP / wave-N rewrite against landed API surface"]
#[test]
fn plugin_manifest_post_install_record_byte_mutation_detected_at_load_verified() {
    let _plugin = stub_plugin_did();
    let _user = stub_user_did();

    // G24-D wave wires this. Substantive shape:
    //
    //   use benten_platform_foundation::plugin_manifest::ManifestStore;
    //
    //   let mut store = common::manifest_fixtures::test_manifest_store();
    //   let plugin_did = stub_plugin_did();
    //   let user_did = stub_user_did();
    //
    //   // Install: install record signed by user-DID with narrow
    //   // `requires` envelope (e.g., only store:notes:read).
    //   let original_record = common::manifest_fixtures::
    //       install_record_signed_by_user(
    //           &user_did, vec!["store:notes:read"]
    //       );
    //   store.install_plugin(
    //       plugin_did.clone(), original_record.clone()
    //   ).unwrap();
    //
    //   // Baseline load: load_verified succeeds.
    //   let loaded = store.load_verified(&plugin_did).unwrap();
    //   assert_eq!(loaded.consenting_user_did, user_did);
    //
    //   // Attack: mutate the install record bytes on disk (or in
    //   // backing store) to widen `requires` envelope. New mutated
    //   // record has WIDER requires (store:notes:read +
    //   // store:secrets:read) but original user signature.
    //   store.simulate_byte_mutation_attack(
    //       plugin_did.clone(),
    //       common::manifest_fixtures::install_record_widened_post_mutation(
    //           &user_did,
    //           vec!["store:notes:read", "store:secrets:read"],
    //       ),
    //   );
    //
    //   // T5a LOAD-BEARING: load_verified MUST detect drift at next
    //   // load — install-record CID no longer matches stored expected
    //   // CID OR user-DID signature no longer verifies over the
    //   // mutated bytes.
    //   let result = store.load_verified(&plugin_did);
    //
    //   let err = result.expect_err(
    //       "T5a LOAD-BEARING: post-install install-record drift MUST \
    //        be detected at load_verified — Layer 2 consent guarantee \
    //        broken if mutation passes silently"
    //   );
    //   assert!(
    //       matches!(err.code(),
    //           ErrorCode::E_PLUGIN_INSTALL_RECORD_USER_SIGNATURE_INVALID
    //           | ErrorCode::E_PLUGIN_MANIFEST_INVALID),
    //       "T5a: must surface typed install-record-invalid error per \
    //        arch-r1-3 ErrorCode split; got {:?}", err.code()
    //   );
    //
    //   // Defense-in-depth: do NOT auto-quarantine — surface to user
    //   // per threat-model §T5 defense step 1 "ANY mismatch → reject +
    //   // surface to user (do not auto-quarantine)":
    //   let user_notification = store.captured_user_notifications();
    //   assert!(
    //       user_notification.iter().any(|n|
    //           n.is_install_record_drift_warning(&plugin_did)),
    //       "T5a: drift detection MUST surface user notification; \
    //        zero notifications means surfacing path is silent"
    //   );
    //
    // OBSERVABLE consequence: install-record mutation rejected at
    // load_verified; user notified; defense-in-depth check at all 3
    // verify points (boot/per-load/per-merge) gets the same code path.
    panic!(
        "RED-PHASE: G24-D must wire ManifestStore::load_verified \
         post-install drift defense (T5a LOAD-BEARING). Substantive: \
         install + mutation + load_verified-rejects + typed error + \
         user-notification surface."
    );
}
