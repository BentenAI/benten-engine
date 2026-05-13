//! Phase-4-Foundation R3 Family F1 — RED-PHASE pin for thin-client
//! session token being time-bound (expires after a finite TTL).
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.11
//! supplementary pin (per Family F1 brief "3-4 more per R2 §2.11");
//! closes T2 defense 2 supplementary — "session token is origin-bound
//! and time-bound" (admin-ui-v0-threat-model.md §T2 line 104).
//!
//! ## What this pin establishes
//!
//! Per T2 defense 2: session tokens carry an explicit time bound. A
//! captured token from an old session (even one that completed
//! legitimately) MUST NOT be replayed long after issuance. Defends
//! against the failure shape where token leak from a log file weeks
//! later is still usable.

#![allow(clippy::unwrap_used)]

mod common;

#[test]
#[ignore = "phase-4-foundation R3 RED-PHASE — G24-F wave-7 wires this. Pin source: r2-test-landscape.md §2.11 supplementary + T2 defense 2. Substantive: establish session; advance clock past TTL; replay → DENIED with typed E_THIN_CLIENT_SESSION_EXPIRED."]
fn admin_ui_v0_thin_client_session_token_time_bound() {
    // G24-F wave wires this. Substantive shape:
    //
    //   let harness = common::admin_ui_v0_harness::AdminUiV0TestHarness::
    //       new_thin_client_against_full_peer();
    //   let origin_a = "https://benten.localhost:8443";
    //
    //   let challenge = harness.full_peer_emit_challenge();
    //   let sig = harness.thin_client_sign_challenge(&challenge);
    //   let token = harness.thin_client_establish_session(
    //       &challenge, &sig, origin_a
    //   ).unwrap();
    //
    //   // Token TTL per G24-F default: e.g. 1 hour. Sanity that the
    //   // token DOES work pre-expiry:
    //   let read_pre = harness.thin_client_read_with_session(
    //       &token, b"some/cid", origin_a
    //   );
    //   assert!(read_pre.is_ok(), "Pre-expiry token MUST succeed");
    //
    //   // Advance harness clock past TTL + a small margin:
    //   let ttl = harness.thin_client_session_ttl();
    //   harness.advance_test_clock(ttl + std::time::Duration::from_secs(60));
    //
    //   // Token must now be expired:
    //   let read_post = harness.thin_client_read_with_session(
    //       &token, b"some/cid", origin_a
    //   );
    //   match read_post {
    //       Ok(_) => panic!(
    //           "Post-expiry token MUST be rejected per T2 defense 2 \
    //            time-bound clause; session-token has no TTL or full \
    //            peer doesn't check it"
    //       ),
    //       Err(e) => {
    //           assert!(
    //               e.code() == "E_THIN_CLIENT_SESSION_EXPIRED",
    //               "Expired token MUST surface typed ErrorCode \
    //                E_THIN_CLIENT_SESSION_EXPIRED; saw {:?}",
    //               e.code(),
    //           );
    //       }
    //   }
    //
    //   // Fresh handshake after expiry succeeds — regression-guard:
    //   let challenge2 = harness.full_peer_emit_challenge();
    //   let sig2 = harness.thin_client_sign_challenge(&challenge2);
    //   let token2 = harness.thin_client_establish_session(
    //       &challenge2, &sig2, origin_a
    //   ).unwrap();
    //   let post_re_establish = harness.thin_client_read_with_session(
    //       &token2, b"some/cid", origin_a
    //   );
    //   assert!(
    //       post_re_establish.is_ok(),
    //       "Fresh handshake post-expiry MUST succeed (regression-guard)"
    //   );
    //
    // OBSERVABLE consequence: long-lived token leak doesn't translate
    // to unbounded access. Defends against the failure shape where
    // tokens never expire.
    unimplemented!(
        "G24-F wires thin-client session-token TTL pin per T2 defense 2 \
         time-bound clause"
    );
}
