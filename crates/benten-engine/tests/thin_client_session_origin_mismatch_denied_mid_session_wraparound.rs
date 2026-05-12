//! Phase-4-Foundation R3 Family F1 — RED-PHASE pin for the MID-SESSION
//! cross-origin replay defense (Family F1 gap #2 per R2 §5 risk #2).
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §5
//! risk #2: "missing a dedicated session-establishment cross-origin-
//! replay-mid-session test that pins a session token issued for origin
//! A is rejected at origin B mid-session (not just at establishment).
//! The existing `thin_client_session_origin_mismatch_denied.rs` covers
//! establishment; a mid-session pin is needed for the wraparound case."
//!
//! ## What this pin defends against
//!
//! T2 defense 3 is enforced on EVERY engine call routed through the
//! thin-client bridge, not just at session establishment. The attack
//! shape this pin closes:
//!
//! 1. User establishes session on origin_a; admin UI runs many requests.
//! 2. Mid-session, a malicious page on origin_b somehow obtains the
//!    session token (XSS leak from a same-origin sub-iframe, debugger
//!    inspection, copy-pasted accidentally, etc.).
//! 3. Hostile page replays the token from origin_b on a write.
//! 4. Engine MUST reject — token's `expected_origin` does not match
//!    request origin, regardless of prior successful uses.

#![allow(clippy::unwrap_used)]

mod common;

#[test]
#[ignore = "phase-4-foundation R3 RED-PHASE — G24-F wave-7 wires this. Pin source: r2-test-landscape.md §5 risk #2 + Family F1 gap-#2 substance check. MID-SESSION wraparound: token used successfully N times from origin_a; one request from origin_b denied; subsequent origin_a request continues to succeed."]
fn thin_client_session_origin_mismatch_denied_mid_session_wraparound() {
    // G24-F wave wires this. Substantive shape (gap-#2 fold-in):
    //
    //   let harness = common::admin_ui_v0_harness::AdminUiV0TestHarness::
    //       new_thin_client_against_full_peer();
    //
    //   let origin_a = "https://benten.localhost:8443";
    //   let origin_b = "https://evil.example";
    //
    //   let challenge = harness.full_peer_emit_challenge();
    //   let signature = harness.thin_client_sign_challenge(&challenge);
    //   let token = harness.thin_client_establish_session(
    //       &challenge, &signature, origin_a
    //   ).unwrap();
    //
    //   // (1) Establish trust: 3 successful reads from origin_a:
    //   for i in 0..3 {
    //       let cid = harness.put_test_node(format!("from_a_{}", i)).unwrap();
    //       let result = harness.thin_client_read_with_session(
    //           &token, &cid, origin_a
    //       );
    //       assert!(
    //           result.is_ok(),
    //           "Same-origin read #{} MUST succeed", i,
    //       );
    //   }
    //
    //   // (2) MID-SESSION: same token presented from origin_b.
    //   // Per Family F1 gap-#2 + T2 defense 3 — the token-leak attack:
    //   let cid_for_attack = harness.put_test_node("victim_data").unwrap();
    //   let cross_origin_attempt = harness.thin_client_read_with_session(
    //       &token, &cid_for_attack, origin_b,
    //   );
    //   match cross_origin_attempt {
    //       Ok(_) => panic!(
    //           "MID-SESSION token presented from origin_b MUST be \
    //            denied per T2 defense 3 even after N prior successful \
    //            uses; gap-#2 unaddressed — engine is enforcing origin \
    //            only at session establishment, not at each request"
    //       ),
    //       Err(e) => {
    //           assert!(
    //               e.code() == "E_THIN_CLIENT_ORIGIN_MISMATCH",
    //               "Cross-origin mid-session replay MUST surface typed \
    //                ErrorCode; saw {:?}",
    //               e.code(),
    //           );
    //       }
    //   }
    //
    //   // (3) Defense-in-depth: origin_a token CONTINUES to work after
    //   // the attack attempt — full peer didn't auto-invalidate on
    //   // cross-origin attempt (avoid a self-inflicted DoS pattern):
    //   let post_attack_read = harness.thin_client_read_with_session(
    //       &token, &cid_for_attack, origin_a
    //   );
    //   assert!(
    //       post_attack_read.is_ok(),
    //       "Origin-A token MUST remain usable after a deflected \
    //        cross-origin attempt; full-peer is over-invalidating \
    //        (DoS surface)"
    //   );
    //
    // OBSERVABLE consequence: per-request origin-binding defense.
    // Defends against the failure shape where defense is only at
    // session establishment (cookies-style) and is bypassed by
    // token-leak attacks (XSS, debugger, etc.).
    unimplemented!(
        "G24-F wires thin-client mid-session origin-mismatch pin per \
         Family F1 gap #2 + T2 defense 3"
    );
}
