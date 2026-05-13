//! Phase-4-Foundation R4-FP-1 — T9a LOAD-BEARING pin: schema with
//! forged peer-DID signature rejected at materializer entry.
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §1 BLOCKER row
//! r4-tc-2 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md` §T9
//! ("Schema content provenance via peer-DID signatures") + CLAUDE.md
//! baked-in #18 four-identity-concepts ratification (peer-DID signature
//! on original content = provenance).
//!
//! ## What this pin establishes
//!
//! Per threat-model §T9a: attacker publishes a schema with peer-DID
//! claim `Alice` but signature actually from attacker's key. Defense:
//! materializer refuses to walk a schema Node whose peer-DID signature
//! doesn't verify against the claimed peer-DID's current public key
//! (resolved via `benten-id` DID resolver).
//!
//! New seam (per threat-model §T9 defense step 1):
//! `crates/benten-platform-foundation/src/schema_provenance_validation.rs::
//! verify_schema_provenance(schema_node: &Node, id_resolver: &DidResolver) -> Result`.
//!
//! T9a is one of the THREE BLOCKER pins for r4-tc-2 (per pim-2-amendment
//! §3.6b sub-rule 4 per-finding granularity — T9a + T9b + trust-list
//! all separate pins, NOT collapsed).
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer wires materializer entry but skips peer-DID signature
//! verification (relies only on content-CID for "trust"). Attacker
//! constructs a hostile schema with claimed peer-DID = trusted author;
//! content-CID hashes to its own bytes (passes content-addressing); no
//! signature check means hostile schema materializes. Every untrusted
//! schema becomes a T1 attack vector.

#![allow(clippy::unwrap_used)]

mod common;

#[test]
#[ignore = "RED-PHASE: G23-B wires schema_provenance_validation at materializer entry; un-ignore at G23-B landing. Pin source: r4-triage §1 r4-tc-2 + threat-model §T9a LOAD-BEARING."]
fn schema_with_forged_peer_did_signature_rejected_at_materializer_entry() {
    // G23-B wave wires this. Substantive shape:
    //
    //   use benten_platform_foundation::schema_provenance_validation::
    //       verify_schema_provenance;
    //
    //   let alice_did = common::manifest_fixtures::stub_peer_did_alice();
    //   let attacker_did = common::manifest_fixtures::stub_peer_did_attacker();
    //
    //   // Hostile schema construction:
    //   //   1. Attacker constructs schema content (any valid bytes).
    //   //   2. Attacker signs with attacker's key.
    //   //   3. Attacker CLAIMS peer-DID = alice_did in the schema Node.
    //   let hostile_schema_node = common::schema_fixtures::
    //       hostile_schema_with_forged_peer_did(
    //           /* claimed_peer_did */ alice_did.clone(),
    //           /* actual_signing_key */ attacker_did.clone(),
    //           /* schema_content */ b"hostile content bytes",
    //       );
    //
    //   // Resolve alice_did to her actual current public key via
    //   // benten-id DID resolver. The hostile signature was made
    //   // with attacker's key — won't verify against alice's pubkey.
    //   let resolver = common::manifest_fixtures::test_did_resolver();
    //
    //   // LOAD-BEARING: materializer entry MUST refuse the schema.
    //   let result = verify_schema_provenance(&hostile_schema_node, &resolver);
    //
    //   let err = result.expect_err(
    //       "T9a LOAD-BEARING: schema with forged peer-DID signature \
    //        MUST be REJECTED — signature verification against \
    //        claimed peer-DID's resolved public key MUST fail"
    //   );
    //   assert!(
    //       matches!(err.code(),
    //           ErrorCode::E_PLUGIN_CONTENT_PEER_SIGNATURE_INVALID),
    //       "T9a: must surface typed E_PLUGIN_CONTENT_PEER_SIGNATURE_INVALID \
    //        per arch-r1-3 ErrorCode split; got {:?}",
    //       err.code()
    //   );
    //
    //   // Defense-in-depth: confirm DID resolver was actually consulted
    //   // (not just structural shape check):
    //   let resolution_trace = resolver.captured_resolve_calls();
    //   assert!(
    //       resolution_trace.iter().any(|c| c.did == alice_did),
    //       "T9a: defense MUST consult DID resolver for claimed peer-DID; \
    //        zero resolve calls means signature verification was skipped"
    //   );
    //
    // OBSERVABLE consequence: signature forgery rejected at the
    // materializer boundary; T1 attack surface narrowed.
    panic!(
        "RED-PHASE: G23-B must wire schema_provenance_validation + \
         materializer-entry signature verification (T9a LOAD-BEARING). \
         Substantive: real hostile-schema construction + DID resolver \
         lookup + typed E_PLUGIN_CONTENT_PEER_SIGNATURE_INVALID + \
         resolver-call trace."
    );
}
