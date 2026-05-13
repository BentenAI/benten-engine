//! Phase-4-Foundation R4-FP-1 — T9 pin: schema author not in admin UI
//! trust-list prompts user (Q3 default = EMPTY per Ben's ratification).
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §1 BLOCKER row
//! r4-tc-2 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md` §T9
//! defense step 5 ("User-trusted-peer-DID list in admin UI manifest
//! envelope") + r4-triage §7 ratification "Q3 schema-author trust-list
//! default: EMPTY + admin UI prompts at first encounter".
//!
//! ## What this pin establishes
//!
//! Per threat-model §T9 defense step 5: admin UI's
//! `requires.schema_authors` enumerates peer-DIDs the admin UI is
//! allowed to materialize schemas from. **Per Ben's Q3 ratification
//! (r4-triage §7)**: the default trust-list is EMPTY — admin UI v0
//! ships with no pre-trusted peer-DIDs; first encounter with each
//! peer-DID prompts the user for explicit trust extension.
//!
//! Defense: schema from unknown peer-DID → materializer returns a
//! "user-prompt-required" outcome (NOT auto-reject, NOT auto-accept).
//! Pair with subsequent user-accept/decline tests (deferred to G24-B
//! workflow editor wave).
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer wires DID resolution + signature verification but
//! skips the trust-list check. Any schema signed by ANY peer-DID
//! (resolution + signature valid) materializes silently — user never
//! consents to introducing a new schema author. Q3 ratification
//! becomes paper.
//!
//! Alternative (broken) shape: implementer ships default trust-list
//! NON-EMPTY (e.g., baked-in Benten authors). Test asserts default-
//! empty matches the Q3 ratification specifically.

#![allow(clippy::unwrap_used)]

mod common;

#[test]
#[ignore = "DESTINATION-REMAPPED at R4b-FP-3 per HARD RULE 12 clause-(b) BELONGS-NAMED-NOW + L1 r4b-l1-6 closure: G24-B shipped at commit c6dfc12 WITHOUT delivering this trust-list prompt path — the `ProvenanceOutcome::UserPromptRequired` surface never built. v1 admin UI ships with default-trust-not-shown (per Ben ratification Q3 at r4-triage §7 default trust-list = EMPTY); the explicit user-prompt UI surface is an enhancement deferred to Phase-4-Meta. Named destination: docs/future/phase-4-backlog.md §4.19 (Phase-4-Meta carry: R5 phantom-destination un-ignore promises)."]
fn schema_author_not_in_admin_ui_trust_list_returns_user_prompt_outcome() {
    // G24-B wave wires this. Substantive shape:
    //
    //   use benten_platform_foundation::schema_provenance_validation::{
    //       verify_schema_provenance, ProvenanceOutcome,
    //   };
    //
    //   // Construct admin UI v0 manifest with default-empty schema-author
    //   // trust-list per Ben Q3 ratification:
    //   let admin_ui_manifest = common::manifest_fixtures::admin_ui_v0_manifest();
    //   assert!(
    //       admin_ui_manifest.requires_schema_authors.is_none()
    //           || admin_ui_manifest.requires_schema_authors.as_ref().unwrap().is_empty(),
    //       "Ben Q3 ratification: admin UI v0 default trust-list MUST \
    //        be EMPTY (None or empty Vec) — got {:?}",
    //       admin_ui_manifest.requires_schema_authors
    //   );
    //
    //   // Construct a schema signed by a peer-DID NOT in any trust-list:
    //   let unknown_peer = common::manifest_fixtures::stub_peer_did_alice();
    //   let schema_node = common::schema_fixtures::schema_signed_by_peer_did(
    //       unknown_peer.clone(),
    //       common::manifest_fixtures::alice_signing_key(),
    //   );
    //
    //   let resolver = common::manifest_fixtures::test_did_resolver();
    //
    //   // Materializer-entry verification:
    //   //   - Signature verifies (alice signed; DID resolver returns
    //   //     alice's pubkey).
    //   //   - HLC monotonic; no rotation issues.
    //   //   - BUT alice_did NOT in admin UI's trust-list (empty).
    //   //   - Outcome: ProvenanceOutcome::UserPromptRequired { peer_did, schema_cid }
    //   let outcome = verify_schema_provenance_with_trust_list(
    //       &schema_node,
    //       &resolver,
    //       &admin_ui_manifest.requires_schema_authors,
    //   ).unwrap();
    //
    //   match outcome {
    //       ProvenanceOutcome::UserPromptRequired { peer_did, .. } => {
    //           assert_eq!(peer_did, unknown_peer,
    //               "T9 trust-list: prompt outcome MUST carry the \
    //                unknown peer-DID for user decision");
    //       },
    //       other => panic!(
    //           "T9 trust-list: schema from non-trusted peer MUST surface \
    //            UserPromptRequired outcome (NOT auto-accept, NOT \
    //            auto-reject) — Ben Q3 ratification; got {:?}", other
    //       ),
    //   }
    //
    //   // Subsequent: user explicitly trusts alice; second schema from
    //   // alice materializes silently (no re-prompt).
    //   let admin_ui_manifest_with_alice = common::manifest_fixtures::
    //       admin_ui_v0_manifest_with_trust_list(vec![unknown_peer.clone()]);
    //   let outcome2 = verify_schema_provenance_with_trust_list(
    //       &schema_node,
    //       &resolver,
    //       &admin_ui_manifest_with_alice.requires_schema_authors,
    //   ).unwrap();
    //   assert!(
    //       matches!(outcome2, ProvenanceOutcome::Trusted),
    //       "T9 trust-list: schema from peer in trust-list MUST \
    //        materialize silently; got {:?}", outcome2
    //   );
    //
    // OBSERVABLE consequence: user-mediated trust extension at first
    // encounter; Ben Q3 ratification enforced structurally.
    panic!(
        "RED-PHASE: G24-B must wire schema-author trust-list prompt \
         path (T9 + Ben Q3 ratification — default trust-list EMPTY). \
         Substantive: empty-default assertion + unknown-author prompt + \
         trusted-author silent path."
    );
}
