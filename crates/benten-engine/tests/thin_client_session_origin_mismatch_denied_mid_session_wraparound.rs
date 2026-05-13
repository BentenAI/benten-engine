//! Phase-4-Foundation R3 Family F1 — RED-PHASE pin (now LIVE at G24-F)
//! for the MID-SESSION cross-origin replay defense (Family F1 gap #2
//! per R2 §5 risk #2).
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

use benten_engine::thin_client::ThinClientSessionError;
use benten_errors::ErrorCode;

#[test]
fn thin_client_session_origin_mismatch_denied_mid_session_wraparound() {
    let harness =
        common::admin_ui_v0_harness::AdminUiV0TestHarness::new_thin_client_against_full_peer();
    let origin_a = "https://benten.localhost:8443";
    let origin_b = "https://evil.example";

    let challenge = harness.full_peer_emit_challenge();
    let signature = harness.thin_client_sign_challenge(&challenge);
    let token = harness
        .thin_client_establish_session(&challenge, &signature, origin_a)
        .expect("Baseline handshake MUST succeed");

    // (1) Establish trust: 3 successful reads from origin_a:
    for i in 0..3 {
        let cid = harness.put_test_node(format!("from_a_{i}")).unwrap();
        let result = harness.thin_client_read_with_session(&token, &cid, origin_a);
        assert!(
            result.is_ok(),
            "Same-origin read #{i} MUST succeed (baseline trust)",
        );
    }
    assert_eq!(
        harness.active_session_count(),
        1,
        "Single active session after baseline reads"
    );

    // (2) MID-SESSION: same token presented from origin_b.
    // Per Family F1 gap-#2 + T2 defense 3 — the token-leak attack:
    let cid_for_attack = harness.put_test_node("victim_data").unwrap();
    let cross_origin_attempt =
        harness.thin_client_read_with_session(&token, &cid_for_attack, origin_b);
    match cross_origin_attempt {
        Ok(()) => panic!(
            "MID-SESSION token presented from origin_b MUST be denied per \
             T2 defense 3 even after N prior successful uses; gap-#2 \
             unaddressed — engine is enforcing origin only at session \
             establishment, not at each request"
        ),
        Err(e) => {
            assert!(
                matches!(e, ThinClientSessionError::OriginMismatch { .. }),
                "Cross-origin mid-session replay MUST surface \
                 OriginMismatch; saw {e:?}"
            );
            assert_eq!(
                e.error_code(),
                ErrorCode::ThinClientOriginMismatch,
                "Cross-origin mid-session replay MUST surface typed \
                 ErrorCode E_THIN_CLIENT_ORIGIN_MISMATCH"
            );
            if let ThinClientSessionError::OriginMismatch { bound, presented } = e {
                assert_eq!(bound, origin_a);
                assert_eq!(presented, origin_b);
            }
        }
    }

    // (3) Defense-in-depth: origin_a token CONTINUES to work after the
    // attack attempt — full peer didn't auto-invalidate on cross-origin
    // attempt (avoid a self-inflicted DoS pattern where a hostile probe
    // knocks legit sessions offline).
    let post_attack_read = harness.thin_client_read_with_session(&token, &cid_for_attack, origin_a);
    assert!(
        post_attack_read.is_ok(),
        "Origin-A token MUST remain usable after a deflected cross-origin \
         attempt; full-peer is over-invalidating (DoS surface)"
    );
    assert_eq!(
        harness.active_session_count(),
        1,
        "Active session count MUST be unchanged across the deflected probe"
    );

    // (4) Defense-in-depth: a SECOND cross-origin probe from yet
    // another origin (origin_c) also fails — recheck is not order-
    // sensitive / doesn't grandfather "first cross-origin attempt".
    let origin_c = "https://other-evil.example";
    let second_probe = harness.thin_client_read_with_session(&token, &cid_for_attack, origin_c);
    assert!(
        second_probe.is_err(),
        "Second cross-origin probe from origin_c MUST also be denied"
    );
    assert_eq!(
        second_probe.unwrap_err().error_code(),
        ErrorCode::ThinClientOriginMismatch,
    );

    // (5) Defense-in-depth: another legit origin_a read after the
    // probe sequence — still works. The active session count remains
    // a single record (no zombie sessions, no auto-revocation).
    let final_legit_read = harness.thin_client_read_with_session(&token, &cid_for_attack, origin_a);
    assert!(
        final_legit_read.is_ok(),
        "Final origin_a read MUST succeed after probe sequence"
    );
    assert_eq!(harness.active_session_count(), 1);
}
