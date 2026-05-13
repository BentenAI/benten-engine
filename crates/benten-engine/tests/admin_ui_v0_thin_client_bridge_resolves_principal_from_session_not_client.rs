//! Phase-4-Foundation R3 Family F1 — RED-PHASE pin for thin-client
//! bridge resolving the engine-call principal from
//! `DidKeyedSession::resolve(token)`, NEVER from a client-supplied
//! principal field.
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.11
//! supplementary pin (per Family F1 brief "3-4 more per R2 §2.11");
//! closes T2 defense 3 second clause (admin-ui-v0-threat-model.md §T2:
//! "`Engine::call_as` invocation through the thin-client bridge MUST
//! receive the resolved session principal from `DidKeyedSession::
//! resolve(token)` — never from a client-supplied principal field").
//!
//! ## Distinction from sibling pin
//!
//! `admin_ui_v0_thin_client_did_handshake_required_for_writes.rs`
//! exercises this defense as part of a broader 4-arm write-boundary
//! pin. This file is a focused unit pin on the resolution path
//! specifically — what happens at the bridge when a request carries
//! BOTH a valid session token AND a client-asserted principal field.

#![allow(clippy::unwrap_used)]

mod common;

#[test]
#[ignore = "DESTINATION-REMAPPED at R6-FP-BF per HARD RULE rule-12 clause-(b) BELONGS-NAMED-NOW. G24-F shipped DidKeyedSession::resolve at thin_client.rs:497-525 returning Principal from token only; the admin_ui_v0 thin-client BRIDGE surface that consumes Principal (per CLAUDE.md #17 shape (b)) is NOT YET BUILT. Named destination: docs/future/phase-4-backlog.md §4.22 (Phase-4-Meta thin-client bridge surface). T2 defense 3 second clause; substantive shape preserved in body comment."]
fn admin_ui_v0_thin_client_bridge_resolves_principal_from_session_not_client() {
    // G24-F wave wires this. Substantive shape:
    //
    //   let harness = common::admin_ui_v0_harness::AdminUiV0TestHarness::
    //       new_thin_client_against_full_peer();
    //   let origin_a = "https://benten.localhost:8443";
    //
    //   // Establish session as admin-UI-DID:
    //   let admin_ui_did = harness.admin_ui_plugin_did();
    //   let challenge = harness.full_peer_emit_challenge();
    //   let sig = harness.thin_client_sign_challenge_as(
    //       &admin_ui_did, &challenge
    //   );
    //   let token = harness.thin_client_establish_session(
    //       &challenge, &sig, origin_a
    //   ).unwrap();
    //
    //   // Adversarial: request asserts principal = user-DID (the
    //   // privileged trust-anchor), but session token resolves to
    //   // admin-UI-DID. Per T2 defense 3: bridge MUST use the
    //   // resolved-from-session principal, NOT the asserted field.
    //   let user_did = harness.user_did();
    //   let trace = harness.trace_capture(|h| {
    //       h.thin_client_call_with_both_token_and_asserted_principal(
    //           Some(&token),
    //           /* client_asserted_principal: */ &user_did,
    //           "create_workflow",
    //           serde_json::json!({ "name": "principal_forge_attempt" }),
    //           origin_a,
    //       )
    //   });
    //
    //   // Per pim-18 §3.6f SUBSTANCE: inspect the trace for the
    //   // principal Engine::call_as was actually invoked with:
    //   let call_as_invocations = trace.calls_to("Engine::call_as");
    //   assert!(
    //       !call_as_invocations.is_empty(),
    //       "Bridge MUST invoke Engine::call_as for write requests"
    //   );
    //   for call in &call_as_invocations {
    //       assert_eq!(
    //           call.principal_arg, admin_ui_did,
    //           "Bridge MUST resolve principal from session token \
    //            (admin-UI-DID), NOT use client-asserted user-DID \
    //            field per T2 defense 3; saw call with principal {:?}",
    //           call.principal_arg,
    //       );
    //       assert_ne!(
    //           call.principal_arg, user_did,
    //           "Bridge MUST NOT use client-asserted principal field; \
    //            saw user-DID principal — bridge is honoring forgery"
    //       );
    //   }
    //
    // OBSERVABLE consequence: defense against principal-forgery via
    // client-supplied field. Defends the inner-attack of T2 — even with
    // a valid session, the client can't elevate to user-DID by
    // asserting it in the request.
    unimplemented!(
        "G24-F wires thin-client bridge principal-resolution pin per \
         T2 defense 3 second clause"
    );
}
