//! Phase-4-Foundation R3 Family F1 — RED-PHASE pin for admin UI v0
//! thin-client writes REQUIRING a completed DID-keyed handshake.
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.6
//! row 8 (LOAD-BEARING substantive); closes T2 + br-r1-1
//! (admin-ui-v0-threat-model.md §T2 defense 1).
//!
//! ## What this pin establishes
//!
//! Per T2 defense 1 + 3: every write routed through the thin-client
//! bridge MUST carry a session-token resolvable to a completed DID-keyed
//! handshake. Reaching the bridge without a token, or with an unresolvable
//! token, MUST fail at the write boundary BEFORE any state is mutated.
//!
//! This pin defends against the failure shape where a thin client can
//! invoke writes by simply attaching plausibly-shaped (but unsigned)
//! request bodies — i.e., the bridge accepts client-asserted principal
//! field instead of resolving it from the session.

#![allow(clippy::unwrap_used)]

mod common;

#[test]
#[ignore = "phase-4-foundation R3 RED-PHASE — G24-A + G24-F wave-7 wires this. Pin source: r2-test-landscape.md §2.6 row 8 + T2 defense 1. LOAD-BEARING substantive: bridge invocation without session-token → DENIED; client-asserted principal field IGNORED; engine state UNCHANGED."]
fn admin_ui_v0_thin_client_did_handshake_required_for_writes() {
    // G24-A + G24-F wave wires this. Substantive shape:
    //
    //   let harness = common::admin_ui_v0_harness::AdminUiV0TestHarness::
    //       new_thin_client_against_full_peer();
    //
    //   let origin_a = "https://benten.localhost:8443";
    //
    //   // Pre-state snapshot for "engine state unchanged" defense:
    //   let pre_audit_seq = harness.full_peer_audit_sequence();
    //   let pre_workflows = harness.full_peer_list_workflows();
    //
    //   // (1) Attempt write WITHOUT session token:
    //   let write_no_token = harness.thin_client_call_raw(
    //       /* session_token: */ None,
    //       "create_workflow",
    //       serde_json::json!({ "name": "unauth_attempt" }),
    //       origin_a,
    //   );
    //   match write_no_token {
    //       Ok(_) => panic!(
    //           "Write WITHOUT session-token MUST be denied at thin-client \
    //            bridge per T2 defense 1; bridge accepted unauth request"
    //       ),
    //       Err(e) => {
    //           assert!(
    //               e.code() == "E_THIN_CLIENT_NO_SESSION"
    //               || e.code() == "E_NO_CAPABILITY",
    //               "Unauth write MUST surface typed ErrorCode; saw {:?}",
    //               e.code(),
    //           );
    //       }
    //   }
    //
    //   // (2) Attempt write with client-ASSERTED principal field (no
    //   // actual signed handshake). Per T2 defense 3: "MUST receive the
    //   // resolved session principal from DidKeyedSession::resolve(token)
    //   // — never from a client-supplied principal field":
    //   let write_asserted_principal = harness.thin_client_call_with_asserted_principal(
    //       /* claimed_principal: */ harness.user_did(),
    //       "create_workflow",
    //       serde_json::json!({ "name": "fake_user_assertion" }),
    //       origin_a,
    //   );
    //   match write_asserted_principal {
    //       Ok(_) => panic!(
    //           "Bridge MUST IGNORE client-asserted principal field per \
    //            T2 defense 3; bridge accepted forged principal"
    //       ),
    //       Err(_) => {} // OK — defense fires.
    //   }
    //
    //   // (3) Engine state unchanged — no writes leaked through:
    //   assert_eq!(
    //       harness.full_peer_audit_sequence(), pre_audit_seq,
    //       "Engine audit sequence MUST be unchanged after rejected \
    //        unauth writes per T2 defense 1+3"
    //   );
    //   assert_eq!(
    //       harness.full_peer_list_workflows(), pre_workflows,
    //       "Engine workflow list MUST be unchanged after rejected \
    //        unauth writes; defense is bypassed at WRITE-boundary"
    //   );
    //
    //   // (4) Compare against an authenticated handshake — sanity:
    //   let challenge = harness.full_peer_emit_challenge();
    //   let sig = harness.thin_client_sign_challenge(&challenge);
    //   let token = harness.thin_client_establish_session(
    //       &challenge, &sig, origin_a
    //   ).unwrap();
    //   let authed_write = harness.thin_client_call_raw(
    //       Some(&token), "create_workflow",
    //       serde_json::json!({ "name": "authed_attempt" }),
    //       origin_a,
    //   );
    //   assert!(
    //       authed_write.is_ok(),
    //       "With valid session-token, write MUST succeed (regression-guard)"
    //   );
    //
    // OBSERVABLE consequence: write-boundary defense against unauth +
    // forged-principal attacks. Defends T2 across all bridge surfaces.
    unimplemented!(
        "G24-A + G24-F wire thin-client handshake-required-for-writes \
         pin per T2 defense 1+3"
    );
}
