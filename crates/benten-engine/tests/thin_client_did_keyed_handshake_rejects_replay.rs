//! Phase-4-Foundation R3 Family F1 — RED-PHASE pin (now LIVE at G24-F)
//! for the DID-keyed handshake rejecting captured-and-replayed session
//! establishment.
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

use benten_engine::thin_client::ThinClientSessionError;
use benten_errors::ErrorCode;

#[test]
fn thin_client_did_keyed_handshake_rejects_replay() {
    let harness =
        common::admin_ui_v0_harness::AdminUiV0TestHarness::new_thin_client_against_full_peer();
    let origin_a = "https://benten.localhost:8443";

    // (1) Mint a fresh challenge + sign it.
    let challenge = harness.full_peer_emit_challenge();
    let signature = harness.thin_client_sign_challenge(&challenge);

    // (2) First handshake succeeds — establishes the baseline.
    let token = harness
        .thin_client_establish_session(&challenge, &signature, origin_a)
        .expect("Fresh handshake MUST establish session");
    let now = 1_700_000_000_u64; // harness clock baseline.
    assert!(
        token.is_valid_at(now),
        "Fresh handshake token MUST be valid against the current clock"
    );

    // (3) Replay against same full peer with same (challenge, sig):
    let replayed = harness.thin_client_establish_session(&challenge, &signature, origin_a);
    match replayed {
        Ok(_token) => panic!(
            "Captured handshake (challenge+signature) MUST NOT replay \
             per T2 defense 1; replay succeeded — challenge is not \
             nonce-bound or full peer doesn't track consumed challenges"
        ),
        Err(e) => {
            assert_eq!(
                e,
                ThinClientSessionError::ChallengeReplay,
                "Replay rejection MUST surface ChallengeReplay variant"
            );
            assert_eq!(
                e.error_code(),
                ErrorCode::ThinClientChallengeReplay,
                "Replay rejection MUST surface typed ErrorCode \
                 E_THIN_CLIENT_CHALLENGE_REPLAY",
            );
        }
    }

    // (4) Defense-in-depth: replay against a DIFFERENT origin must
    // ALSO fail — captured signature is bound to origin_a context.
    // After the (3) replay above the challenge is in the
    // consumed-nonces set so it surfaces ChallengeReplay (the more
    // specific failure mode). That's the correct disposition: even
    // cross-origin replay defense doesn't override the consumed-nonce
    // check.
    let origin_b = "https://evil.example";
    let cross_origin_replay =
        harness.thin_client_establish_session(&challenge, &signature, origin_b);
    assert!(
        cross_origin_replay.is_err(),
        "Cross-origin replay MUST fail per T2 defense 3 + br-r1-1"
    );

    // (5) Fresh challenge after the replays — succeeds. Regression-
    // guard that the replay defense doesn't permanently break the
    // handshake path for the principal DID.
    let challenge2 = harness.full_peer_emit_challenge();
    let sig2 = harness.thin_client_sign_challenge(&challenge2);
    let token2 = harness
        .thin_client_establish_session(&challenge2, &sig2, origin_a)
        .expect("Fresh handshake after replay defense MUST succeed");
    assert_ne!(
        token2.token_id, token.token_id,
        "Fresh handshake MUST mint a distinct token id"
    );
}
