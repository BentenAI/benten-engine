//! Phase-4-Foundation R3 Family F1 — RED-PHASE pin (now LIVE at G24-F)
//! for a session token issued for origin A being rejected at origin B
//! at session-establishment time.
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

use benten_engine::thin_client::ThinClientSessionError;
use benten_errors::ErrorCode;

#[test]
fn thin_client_session_origin_mismatch_denied_at_establishment() {
    let harness =
        common::admin_ui_v0_harness::AdminUiV0TestHarness::new_thin_client_against_full_peer();
    let origin_a = "https://benten.localhost:8443";
    let origin_b = "https://evil.example";

    // (1) Full peer mints a challenge bound to origin_a (the user's
    // legit local admin UI tab origin).
    let challenge = harness.full_peer_emit_challenge();
    assert_eq!(
        challenge.claimed_origin, origin_a,
        "Harness mints challenges bound to its default origin"
    );
    let signature = harness.thin_client_sign_challenge(&challenge);

    // (2) Same-origin handshake completes — sanity baseline.
    let token_a = harness
        .thin_client_establish_session(&challenge, &signature, origin_a)
        .expect("Same-origin handshake MUST succeed (baseline)");
    assert_eq!(
        token_a.bound_origin, origin_a,
        "Same-origin handshake MUST bind token to origin_a"
    );

    // (3) Fresh challenge for the cross-origin probe (the challenge in
    // (1) was consumed). Origin_a is still the claimed_origin on the
    // server side (full peer mints challenges bound to its serving
    // origin, NOT to the eventual presenter's origin).
    let challenge2 = harness.full_peer_emit_challenge();
    let signature2 = harness.thin_client_sign_challenge(&challenge2);

    // (4) Establishment from origin_b — DENIED with typed code.
    let cross_origin_establishment =
        harness.thin_client_establish_session(&challenge2, &signature2, origin_b);
    match cross_origin_establishment {
        Ok(_token) => panic!(
            "Cross-origin handshake MUST be denied at the establishment \
             boundary per T2 defense 3; the engine accepted a handshake \
             with the wrong origin"
        ),
        Err(e) => {
            assert!(
                matches!(e, ThinClientSessionError::OriginMismatch { .. }),
                "Cross-origin handshake MUST surface OriginMismatch variant; \
                 saw {e:?}"
            );
            assert_eq!(
                e.error_code(),
                ErrorCode::ThinClientOriginMismatch,
                "Origin mismatch MUST surface typed ErrorCode \
                 E_THIN_CLIENT_ORIGIN_MISMATCH"
            );
            if let ThinClientSessionError::OriginMismatch { bound, presented } = e {
                assert_eq!(
                    bound, origin_a,
                    "OriginMismatch.bound MUST cite the challenge's claimed origin"
                );
                assert_eq!(
                    presented, origin_b,
                    "OriginMismatch.presented MUST cite the requesting origin"
                );
            }
        }
    }

    // (5) Defense-in-depth: token_a (from the legit origin_a handshake)
    // presented from origin_b at resolve time is ALSO rejected. This
    // is the cross-cut with the mid-session-wraparound pin: the
    // recheck fires at BOTH the establishment boundary AND the resolve
    // boundary.
    let read_with_token_a_from_origin_b =
        harness.thin_client_read_with_session(&token_a, b"some/cid", origin_b);
    match read_with_token_a_from_origin_b {
        Ok(()) => panic!(
            "Token bound to origin_a MUST NOT be honored when presented \
             from origin_b per T2 defense 3"
        ),
        Err(e) => {
            assert_eq!(
                e.error_code(),
                ErrorCode::ThinClientOriginMismatch,
                "Cross-origin resolve MUST surface E_THIN_CLIENT_ORIGIN_MISMATCH"
            );
        }
    }
}
