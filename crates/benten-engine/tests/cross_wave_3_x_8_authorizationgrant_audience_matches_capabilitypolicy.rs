//! Phase-4-Meta-Core — R3-W5 — Cross-wave integration test §4-D:
//! G-CORE-3 × G-CORE-8 security-surface composition. The §8-E SEALED-
//! discipline `CapabilityPolicy` audience-aware `check_write` hook
//! composes with the new `AuthorizationGrant` validator (audience
//! binding consistent across both; same audience field flows through;
//! mismatch on either side denies). Production-arm: AuthorizationGrant
//! audience = X; CapabilityPolicy audience = Y → composition denies
//! (the structural "audience-aware" intent).
//!
//! ============================================================================
//! RED-PHASE STATUS
//! ============================================================================
//!
//! `#[ignore = "RED-PHASE: un-ignore at G-CORE-3b + G-CORE-8 composition
//! wave"]`. Both halves are unbuilt at synced HEAD `c9c11c56`.
//!
//! Disjointness vs W3's `tf3b_authorization_grant_binding_sig_one_signed_artifact.rs`
//! (HARD §3.5i): W3 tests `AuthorizationGrant` at the caps-crate unit
//! level — the binding_sig produces ONE signed artifact bundling
//! {ucan, key_material}. THIS file tests the INTEGRATION-LEVEL
//! cross-wave composition: that the audience field actually flows
//! through CapabilityPolicy::check_write at engine-call time (the
//! §4-D structural audience-aware intent). Different crate
//! (benten-engine vs benten-caps); different surface (engine
//! integration vs caps unit).
//!
//! ============================================================================
//! GROUND-TRUTH (synced HEAD c9c11c56 — R3-W5 §3.5n pass)
//! ============================================================================
//!
//!   * G-CORE-3b: AuthorizationGrant + binding_sig + Scope enum
//!     UNBUILT.
//!   * G-CORE-8: the new §8-E SEALED CapabilityPolicy hooks — install-
//!     time consent / per-delegation runtime / audience-aware
//!     check_write — UNBUILT. The §1.A.FROZEN item 8 plan body
//!     describes the audience-aware check_write as a new
//!     WriteContext field; the §4-D pin verifies BOTH halves see
//!     the SAME audience.
//!
//! ============================================================================
//! §3.6g LITERAL discipline checklist
//! ============================================================================
//!
//!  1. §3.5b HARDENED — implementer sweeps SECURITY-POSTURE.md +
//!     CapabilityPolicy doc surface.
//!  2. §3.6b sub-rule 4 — SPECIFIC arm = same audience flows through
//!     BOTH AuthorizationGrant verify AND CapabilityPolicy
//!     check_write; OBSERVABLE = a mismatch on either side denies;
//!     WOULD-FAIL if the new hook ignores the audience field
//!     (silently admit on mismatch).
//!  3. §3.6e — RED-PHASE staged.
//!  4. §3.6f (pim-18) — production call-site = future
//!     `Engine::write_with_grant(&grant, &write)` threading audience
//!     through both validators; substantive body exercises the
//!     mismatch path end-to-end.
//!  5. §3.13 — per-test-static.
//!  6. §3.5g — no new ErrorCode here (the implementer mints a typed
//!     audience-mismatch variant which mirrors §3.5g; this pin
//!     just asserts the deny path).
//!  7. §3.5n — verified the §8-E hooks are unbuilt at HEAD;
//!     `manifest_envelope_recheck.rs` carries the existing recheck
//!     surface but not yet the audience-aware variant.
//!
//! Pin source: R2-test-landscape.md §4-D + plan §1.A.FROZEN item 8
//! (audience context as new WriteContext field) + G-CORE-8 §4.23 +
//! G-CORE-3b §.

#![allow(clippy::unwrap_used)]

/// §4-D ARM 1 — A grant with audience=X presented to a write whose
/// CapabilityPolicy check_write hook sees audience=Y → BOTH
/// validators must individually deny. The composition is AND-shaped
/// (both must admit), not OR-shaped (either-suffices would be the
/// wrong shape).
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3b + G-CORE-8 composition wave. §1.A.FROZEN item 8 audience-aware check_write."]
fn audience_mismatch_between_authorizationgrant_and_capabilitypolicy_denies() {
    // Body intent at un-ignore:
    //
    //   // Issue a grant with audience=X.
    //   let grant_to_x = AuthorizationGrant::issue(&alice, &x_did, &spec);
    //   // Configure engine_y with a CapabilityPolicy that expects
    //   // audience=Y on its check_write (the SEALED-trait hook the
    //   // §8-E decision produces).
    //   let mut engine_y = Engine::test_native_with_audience_policy(y_did);
    //
    //   // Write attempt: grant audience X, policy audience Y.
    //   let result = engine_y.write_with_grant(&grant_to_x, &some_write);
    //
    //   // §4-D AND-shape: even if one side admitted, the other must
    //   // deny.  Acceptable failure modes (both correct):
    //   //   - typed UcanAudienceMismatch from the grant validator;
    //   //   - typed AudienceCheckRejected from CapabilityPolicy.
    //   //
    //   match result {
    //       Err(EngineError::UcanAudienceMismatch(_)) |
    //       Err(EngineError::AudienceCheckRejected(_)) => { /* OK */ },
    //       _ => panic!("§4-D AND-composition: mismatched audience MUST \
    //                    be denied by at LEAST one of (grant verify, \
    //                    CapabilityPolicy.check_write). Would-FAIL if \
    //                    the audience field is silently ignored on \
    //                    either side (#1301 + G-CORE-8 composition gap)."),
    //   }
    //
    panic!(
        "RED-PHASE: §4-D G-CORE-3b×G-CORE-8 audience-composition. At \
         HEAD AuthorizationGrant + audience-aware CapabilityPolicy hook \
         are unbuilt. Un-ignore at G-CORE-8 substrate-tier (sealed-trait \
         hook landing) + G-CORE-3b. Would-FAIL on silent-ignore of \
         audience mismatch on either side."
    );
}

/// §4-D ARM 2 — Audience MATCH on both sides admits the write (the
/// positive control: composition is correctly AND, not always-deny).
/// This pins that the composition does NOT inadvertently deny a
/// well-formed match — proves the AND-shape is correct in both
/// directions.
#[test]
#[ignore = "RED-PHASE: un-ignore at composition wave (positive-control arm)."]
fn audience_match_on_both_sides_admits_the_write_positive_control() {
    // Body at un-ignore: issue grant with audience=plugin_did;
    // configure CapabilityPolicy with audience=plugin_did; write
    // succeeds. Verifies the composition is NOT a both-always-deny
    // bug.
    panic!(
        "RED-PHASE: §4-D positive-control: matched audience on both \
         sides admits. Un-ignore at composition wave."
    );
}

/// §4-D ARM 3 — Chain composition: an AuthorizationGrant carrying a
/// UCAN chain that terminates at user-DID-root (the §4.23 user-DID
/// root-chain validator) AND audience=plugin_did matches the
/// CapabilityPolicy audience field → admit. Mutate any link in the
/// chain → deny. Pins that the chain-validator (G-CORE-3b) and the
/// audience-aware hook (G-CORE-8) co-validate without any cross-talk
/// gap that would admit on a chain that does NOT terminate at the
/// root.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3b + G-CORE-8 §4.23 + §4.36 composition wave."]
fn chain_validator_terminates_at_user_root_audience_matches_admit_else_deny() {
    // Body at un-ignore: build a UCAN chain alice→bob→plugin_did
    // signed correctly; admit. Build a chain alice→fake-not-by-bob→
    // plugin_did (forged middle); deny via chain validator. Build a
    // chain bob→plugin_did (does NOT terminate at user-DID-root, the
    // §4.23 structural-always-on check); deny.
    panic!(
        "RED-PHASE: §4-D chain-terminate-at-user-root composition. \
         Un-ignore at G-CORE-3b chain-validator landing + G-CORE-8 \
         §4.23 write-boundary user-DID-root chain check."
    );
}
