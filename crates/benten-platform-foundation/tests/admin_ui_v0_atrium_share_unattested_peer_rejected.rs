//! Phase-4-Foundation R4-FP-1 — T6c pin: admin UI v0 Atrium share from
//! unattested peer rejected (HandshakeFrame peer DID validation).
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §2 MAJOR row
//! r4-tc-4 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md`
//! §T6c ("Peer-identity forgery during share handshake") + defense
//! step 3 (HandshakeFrame type-state machine validates peer DID against
//! device-attestation envelope V2; Phase-3 G16-D wave-6b precedent
//! PR #163).
//!
//! ## What this pin establishes
//!
//! Per threat-model §T6c: "Hostile peer claims a sharing-peer identity
//! it doesn't hold; without `HandshakeFrame` type-state enforcement
//! validating peer DID, receiver might accept share from un-attested
//! peer."
//!
//! Defense: existing `benten-sync` `HandshakeFrame` type-state machine
//! validates peer DID against device-attestation envelope V2 (signed,
//! Ed25519, payload-hash-bound; PR #163 G16-D wave-6b). Hostile peer
//! with un-attested identity rejected BEFORE share negotiation begins.
//!
//! Note per threat-model: this validates the SHARING peer's device-
//! identity, NOT the plugin's content provenance. Sharing-peer vs
//! content-author can legitimately differ (peer re-sharing is the
//! whole point of Atrium ecosystem).
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer wires share-receive path but skips HandshakeFrame peer
//! DID validation (relies on transport-layer peer identity). Hostile
//! peer claims attested-device-DID at handshake; receiver accepts
//! share; admin UI v0 bundle delivered from un-attested source.

#![allow(clippy::unwrap_used)]

mod common;

use common::manifest_fixtures::{stub_peer_did_alice, stub_peer_did_attacker, stub_plugin_did};

#[ignore = "RED-PHASE-BODY: panic-stub body needs substantive G24-D-FP / wave-N rewrite against landed API surface"]
#[test]
fn admin_ui_v0_atrium_share_from_unattested_peer_rejected_at_handshake_frame() {
    let _plugin = stub_plugin_did();
    let _alice = stub_peer_did_alice();
    let _attacker = stub_peer_did_attacker();

    // G24-D wave wires this. Substantive shape:
    //
    //   use benten_platform_foundation::plugin_lifecycle::accept_atrium_share;
    //
    //   let mut engine = common::manifest_fixtures::test_engine_with_user_did();
    //   let alice = stub_peer_did_alice();
    //   let attacker = stub_peer_did_attacker();
    //
    //   // Alice has a properly attested device-DID + Ed25519 signature
    //   // chain via Phase-3 G16-D wave-6b precedent.
    //   common::manifest_fixtures::register_attested_peer(
    //       &mut engine, alice.clone(),
    //   );
    //   // Attacker is NOT registered as an attested peer.
    //
    //   // Construct HandshakeFrame claiming attacker = trusted peer
    //   // but with no device-attestation envelope.
    //   let hostile_handshake = common::manifest_fixtures::
    //       handshake_frame_claiming_peer_without_attestation(
    //           attacker.clone(),
    //       );
    //
    //   let share_attempt = accept_atrium_share(
    //       &mut engine,
    //       /* sharing_peer */ attacker.clone(),
    //       /* handshake */ hostile_handshake,
    //       /* admin UI v0 bundle */
    //       common::manifest_fixtures::admin_ui_v0_share_bundle(),
    //   );
    //
    //   let err = share_attempt.expect_err(
    //       "T6c: Atrium share from unattested peer MUST be REJECTED \
    //        at HandshakeFrame validation — un-attested device identity"
    //   );
    //   assert!(
    //       matches!(err.code(),
    //           ErrorCode::E_DEVICE_ATTESTATION_FORGED
    //           | ErrorCode::E_PLUGIN_DEVICE_ATTESTATION_FORGED),
    //       "T6c: must surface typed device-attestation-forged error \
    //        (Phase-3 carry + G24-D-renamed variant per r4-triage §7 \
    //        Decision 3); got {:?}", err.code()
    //   );
    //
    //   // Defense-in-depth: rejection happened BEFORE share negotiation
    //   // — no bytes from attacker were buffered into ManifestStore:
    //   let installed = engine.manifest_store().installed_plugins();
    //   assert!(installed.is_empty(),
    //       "T6c: rejected handshake MUST NOT buffer any share bytes \
    //        into ManifestStore");
    //
    //   // Boundary: same admin UI bundle delivered through ALICE
    //   // (properly attested peer) succeeds — sharing-peer ≠ content-
    //   // author per threat-model §T6 "Sharing-peer vs content-author
    //   // can legitimately differ":
    //   let alice_share = accept_atrium_share(
    //       &mut engine,
    //       alice.clone(),
    //       common::manifest_fixtures::valid_handshake_frame_for(&alice),
    //       common::manifest_fixtures::admin_ui_v0_share_bundle(),
    //   );
    //   assert!(alice_share.is_ok(),
    //       "T6c boundary: attested-peer share MUST succeed — defense \
    //        protects against forgery, NOT against legitimate peer \
    //        re-sharing");
    //
    // OBSERVABLE consequence: HandshakeFrame type-state machine catches
    // un-attested-peer at handshake; defense reuses Phase-3 G16-D
    // wave-6b infrastructure (no new attestation pathway).
    panic!(
        "RED-PHASE: G24-D must wire HandshakeFrame peer-DID validation \
         at Atrium-share boundary (T6c). Substantive: unattested-peer-\
         rejected + typed E_PLUGIN_DEVICE_ATTESTATION_FORGED + no-\
         buffer-commit + attested-peer-OK boundary."
    );
}
