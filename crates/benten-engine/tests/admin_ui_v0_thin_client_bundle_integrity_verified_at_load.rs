//! Phase-4-Foundation R3 Family F1 — RED-PHASE pin for thin-client
//! verifying admin UI v0 bundle integrity at load (content-CID
//! matches install-record consented bundle).
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.6
//! row 9 (substantive); closes T2 + T5 (admin-ui-v0-threat-model.md
//! §T2 defense 4 RETENSED — bundle integrity via user-DID-signed
//! install record, content-addressing as the verification surface).
//!
//! ## What this pin establishes
//!
//! Per T2 defense 4 (RETENSED post-R1-triage per br-r1-15 + D-4F-12):
//! admin UI v0 bundle integrity is verified at install via user-DID-
//! signed install record (Layer 2 install-time manifest envelope; signed
//! by user-DID at install consent). At LOAD time, the thin-client
//! verifies the bundle's content-CID matches what the install record
//! consented to. Content-addressing IS the bundle-integrity check.
//!
//! Per T5b defense: a substituted bundle (different content bytes, same
//! human-readable name) MUST fail this load-time check because its
//! content-CID won't match the install record's stored expected CID.

#![allow(clippy::unwrap_used)]

mod common;

#[test]
#[ignore = "DESTINATION-REMAPPED at R6-FP-BF per HARD RULE rule-12 clause-(b) BELONGS-NAMED-NOW. G24-D shipped manifest-envelope substitution defense at install_plugin; the load-time bundle-integrity verifier at the thin-client BRIDGE entry point (where the browser receives the bundle bytes pre-install) is part of the thin-client bridge surface NOT YET BUILT. Named destination: docs/future/phase-4-backlog.md §4.22. T2 defense 4 RETENSED + T5b; substantive shape preserved in body comment."]
fn admin_ui_v0_thin_client_bundle_integrity_verified_at_load() {
    // G24-A + G24-F + G24-D wire this. Substantive shape:
    //
    //   let harness = common::admin_ui_v0_harness::AdminUiV0TestHarness::
    //       new_thin_client_against_full_peer();
    //
    //   // (1) Install legit admin UI bundle; install record records
    //   // the expected content-CID:
    //   let legit_bundle_bytes = harness.legit_admin_ui_bundle_bytes();
    //   let legit_cid = harness.put_test_node_bytes(&legit_bundle_bytes);
    //   let install_record_cid = harness.install_admin_ui_at(legit_cid).unwrap();
    //   let install_record = harness.load_install_record(&install_record_cid);
    //   assert_eq!(
    //       install_record.manifest_cid, legit_cid,
    //       "Install record manifest_cid MUST equal bundle content-CID \
    //        per T2 defense 4 RETENSED + D-4F-12"
    //   );
    //
    //   // (2) Load attempt succeeds with the legit bundle:
    //   let load_legit = harness.thin_client_load_bundle(legit_cid);
    //   assert!(
    //       load_legit.is_ok(),
    //       "Legit bundle MUST load when content-CID matches install \
    //        record per T2 defense 4 RETENSED"
    //   );
    //
    //   // (3) Substitute bundle bytes — attacker tampered the file at
    //   // rest. Per T5b: the substituted content has a different CID.
    //   let tampered_bytes = {
    //       let mut b = legit_bundle_bytes.clone();
    //       b.extend_from_slice(b"// malicious append");
    //       b
    //   };
    //   let tampered_cid = harness.put_test_node_bytes(&tampered_bytes);
    //   assert_ne!(
    //       legit_cid, tampered_cid,
    //       "Sanity — tampered bundle MUST have different CID (BLAKE3 \
    //        content-addressing)"
    //   );
    //
    //   // (4) Loading the tampered bundle through the same install
    //   // record's expected-CID gate must FAIL with typed ErrorCode:
    //   let load_tampered = harness
    //       .thin_client_load_bundle_against_install_record(
    //           tampered_cid, &install_record_cid,
    //       );
    //   match load_tampered {
    //       Ok(_) => panic!(
    //           "Tampered bundle MUST fail load-time content-CID check \
    //            per T2 defense 4 RETENSED + T5b; bundle integrity \
    //            verification is no-op'd"
    //       ),
    //       Err(e) => {
    //           assert!(
    //               e.code() == "E_PLUGIN_CONTENT_CID_MISMATCH",
    //               "Tampered bundle MUST surface typed \
    //                E_PLUGIN_CONTENT_CID_MISMATCH per G24-D minted \
    //                ErrorCode; saw {:?}",
    //               e.code(),
    //           );
    //       }
    //   }
    //
    // OBSERVABLE consequence: load-time bundle integrity defense.
    // Defends against the bytes-tampered-at-rest attack class without
    // requiring a separate engine-release-key signature (per D-4F-12).
    unimplemented!(
        "G24-A + G24-F + G24-D wire admin UI thin-client \
         bundle-integrity-verified-at-load pin per T2 defense 4 \
         RETENSED + T5b"
    );
}
