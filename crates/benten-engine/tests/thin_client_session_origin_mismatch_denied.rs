//! Phase-4-Foundation R3 Family F1 — RED-PHASE pin for G24-F thin-client
//! session token issued for origin A being rejected at origin B at
//! session-establishment time.
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.11
//! row 2 (LOAD-BEARING substantive); closes T2 + sec-4f-r1-5
//! (admin-ui-v0-threat-model.md §T2 defense 3 — origin pinning at
//! full-peer write boundary).
//!
//! ## Companion pin
//!
//! This file pins the **session-establishment-time** origin defense.
//! The mid-session wraparound case (Family F1 gap #2 per R2 §5 risk #2)
//! lives in
//! `thin_client_session_origin_mismatch_denied_mid_session_wraparound.rs`.

#![allow(clippy::unwrap_used)]

mod common;

#[test]
#[ignore = "phase-4-foundation R3 RED-PHASE — G24-F wave-7 wires this. Pin source: r2-test-landscape.md §2.11 row 2 + T2 defense 3. LOAD-BEARING: token issued for origin A presented from origin B at establishment → DENIED with typed E_THIN_CLIENT_ORIGIN_MISMATCH."]
fn thin_client_session_origin_mismatch_denied_at_establishment() {
    // G24-F wave wires this. Substantive shape:
    //
    //   let harness = common::admin_ui_v0_harness::AdminUiV0TestHarness::
    //       new_thin_client_against_full_peer();
    //
    //   // Full peer is bound to localhost; admin UI is served from
    //   // origin_a (the user's legit local admin UI tab origin):
    //   let origin_a = "https://benten.localhost:8443";
    //   let origin_b = "https://evil.example";
    //
    //   // Origin-A handshake completes:
    //   let challenge = harness.full_peer_emit_challenge();
    //   let signature = harness.thin_client_sign_challenge(&challenge);
    //   let token_a = harness.thin_client_establish_session(
    //       &challenge, &signature, origin_a
    //   ).unwrap();
    //   assert_eq!(token_a.bound_origin, origin_a);
    //
    //   // Token issued for origin_a presented FROM origin_b is rejected:
    //   let read_attempt = harness.thin_client_read_with_session(
    //       &token_a, b"some/cid", origin_b,
    //   );
    //   match read_attempt {
    //       Ok(_) => panic!(
    //           "Token bound to origin_a MUST NOT be honored when \
    //            presented from origin_b per T2 defense 3; the engine \
    //            is missing origin-binding enforcement at the bridge"
    //       ),
    //       Err(e) => {
    //           assert!(
    //               e.code() == "E_THIN_CLIENT_ORIGIN_MISMATCH",
    //               "Origin mismatch MUST surface typed ErrorCode \
    //                E_THIN_CLIENT_ORIGIN_MISMATCH; saw {:?}",
    //               e.code(),
    //           );
    //       }
    //   }
    //
    // OBSERVABLE consequence: origin-binding defense at session-token
    // resolution. Defends against the failure shape where the engine
    // accepts any valid session token regardless of presenting origin.
    unimplemented!(
        "G24-F wires thin-client session origin-mismatch pin per T2 \
         defense 3"
    );
}
