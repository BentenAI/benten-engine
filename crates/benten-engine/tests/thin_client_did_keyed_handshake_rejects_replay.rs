//! Phase-4-Foundation R3 Family F1 — RED-PHASE pin for G24-F thin-client
//! DID-keyed handshake rejecting captured-and-replayed session establishment.
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.11
//! row 1 (LOAD-BEARING substantive); closes T2 + br-r1-1
//! (admin-ui-v0-threat-model.md §T2 defense 1 + 3).
//!
//! ## What this pin establishes
//!
//! Per T2 defense 1: admin UI v0 establishes session via DID-keyed
//! handshake — admin-UI-DID signs a challenge from the full peer; full
//! peer issues a session token bound to (DID, origin, time). Per T2
//! defense 3: the captured handshake exchange MUST NOT replay
//! cross-origin (the malicious origin has no admin-UI-DID private key
//! AND the bound origin in the session record won't match).
//!
//! This pin pins the cryptographic-replay resistance of the handshake
//! itself (a strictly stronger property than origin-binding alone) —
//! even if origin matched, the challenge MUST be single-use.

#![allow(clippy::unwrap_used)]

mod common;

#[test]
#[ignore = "phase-4-foundation R3 RED-PHASE — G24-F wave-7 wires this. Pin source: r2-test-landscape.md §2.11 row 1 + T2 defense 1 + 3. LOAD-BEARING substantive: capture handshake bytes; replay against same full peer; assert second handshake REJECTED with typed error. Would FAIL if challenge isn't nonce-bound."]
fn thin_client_did_keyed_handshake_rejects_replay() {
    // G24-F wave wires this. Substantive shape:
    //
    //   let harness = common::admin_ui_v0_harness::AdminUiV0TestHarness::
    //       new_thin_client_against_full_peer();
    //
    //   let challenge = harness.full_peer_emit_challenge();
    //   let signature = harness.thin_client_sign_challenge(&challenge);
    //   let origin_a = "https://benten.localhost:8443";
    //
    //   // First handshake succeeds:
    //   let token = harness.thin_client_establish_session(
    //       &challenge, &signature, origin_a
    //   ).unwrap();
    //   assert!(token.is_valid(), "Fresh handshake MUST establish session");
    //
    //   // Capture the bytes + replay against same full peer:
    //   let replayed = harness.thin_client_establish_session(
    //       &challenge, &signature, origin_a,
    //   );
    //   match replayed {
    //       Ok(_) => panic!(
    //           "Captured handshake (challenge+signature) MUST NOT replay \
    //            per T2 defense 1; replay succeeded — challenge is not \
    //            nonce-bound or full peer doesn't track consumed challenges"
    //       ),
    //       Err(e) => {
    //           // Per minted ErrorCode at G24-F:
    //           assert!(
    //               e.code() == "E_THIN_CLIENT_CHALLENGE_REPLAY"
    //               || e.code() == "E_THIN_CLIENT_HANDSHAKE_INVALID",
    //               "Replay rejection MUST surface typed ErrorCode; saw {:?}",
    //               e.code(),
    //           );
    //       }
    //   }
    //
    //   // Defense-in-depth: replay against a DIFFERENT origin must
    //   // ALSO fail — captured signature is bound to origin_a context:
    //   let origin_b = "https://evil.example";
    //   let cross_origin_replay = harness.thin_client_establish_session(
    //       &challenge, &signature, origin_b,
    //   );
    //   assert!(
    //       cross_origin_replay.is_err(),
    //       "Cross-origin replay MUST fail per T2 defense 3 + br-r1-1"
    //   );
    //
    // OBSERVABLE consequence: handshake-level replay defense. Defends
    // against the attack class where a hostile origin captures the
    // network exchange via a transparent proxy and replays it later.
    unimplemented!(
        "G24-F wires thin-client handshake replay-rejection pin per T2 \
         defense 1+3"
    );
}
