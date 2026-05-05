//! R3-B RED-PHASE pins for `benten-id` device-DID capability-attestation
//! surface (G14-A2 wave-4a'; D-PHASE-3-25 + crypto-major-6 +
//! device-DID-attestation-replay defect-class + pim-r1-pim-induction-7).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.2 G14-A2 +
//! §10 device-mesh exploration + §3.F multi-device-sync cluster):
//!
//! - `tests/device_attestation_round_trip` — plan §3 G14-A2
//! - `tests/device_attestation_consumed_at_ucan_delegation_chain_walk` — exploration-device-mesh
//! - `tests/device_attestation_replay_resistant_within_freshness_window` — device-DID-attestation-replay defect-class
//! - `tests/device_attestation_replay_resistance_via_nonce_freshness_window` — pattern-induction unnamed-defect-class
//! - `tests/device_attestation_revocation_emitted_by_parent_did_on_loss_event` — crypto-major-6
//! - `tests/device_attestation_revoked_device_cannot_sign_new_ucan_delegation` — crypto-major-6
//!
//! ## Architectural intent (D-PHASE-3-25)
//!
//! Each device under a shared logical identity declares its
//! capability envelope via a signed device-DID attestation:
//! - `runs_sandbox: bool` — does this device execute SANDBOX modules?
//! - `holds_zones: ZoneScope` — full / cache-only / specific-list
//! - `online_uptime: UptimePolicy` — always-on / session-bounded
//! - `runs_atrium_peer: bool` — full peer or thin client?
//!
//! Per CLAUDE.md baked-in #17, thin compute surfaces (browser tabs,
//! Phase-9+ edge workers) declare minimum envelopes. The attestation
//! is consumed at UCAN delegation chain-walk so per-device cap
//! policy can enforce envelope-derived limits.
//!
//! ## Replay-resistance (defect-class)
//!
//! Per `feedback_3_plus_recurrence_deep_sweep` + pim-r1-pim-induction-7,
//! device attestations MUST be replay-resistant via:
//! 1. nonce + freshness-window (attestation ages out)
//! 2. nonce-store (per-issuing-DID; bounds on size + retention)
//!
//! Without this, a captured attestation could be replayed against the
//! parent DID's UCAN backend after the device was decommissioned.
//!
//! ## RED-PHASE discipline
//!
//! Per R3-A canary precedent. Stays `#[ignore]`'d until G14-A2
//! implementer un-ignores. Per §3.6b pim-2, replay-resistance tests
//! must drive the production attestation-acceptance path and assert
//! that a stale attestation observably rejects.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-A2 — plan §3 G14-A2 — device attestation round-trip"]
fn device_attestation_round_trip() {
    // plan §3 G14-A2 pin. G14-A2 implementer wires this:
    //
    //   let parent_kp = benten_id::keypair::Keypair::generate();
    //   let device_kp = benten_id::keypair::Keypair::generate();
    //
    //   let envelope = benten_id::device_attestation::CapabilityEnvelope {
    //       runs_sandbox: false,
    //       holds_zones: benten_id::device_attestation::ZoneScope::CacheOnly,
    //       online_uptime: benten_id::device_attestation::UptimePolicy::SessionBounded,
    //       runs_atrium_peer: false,
    //   };
    //   let attestation = benten_id::device_attestation::DeviceAttestation::issue(
    //       &parent_kp,
    //       device_kp.public_key().to_did(),
    //       envelope.clone(),
    //   ).unwrap();
    //
    //   // Round-trip: serialize + deserialize via canonical bytes
    //   let bytes = attestation.canonical_bytes();
    //   let parsed = benten_id::device_attestation::DeviceAttestation::from_canonical_bytes(&bytes).unwrap();
    //   assert_eq!(parsed.envelope(), &envelope);
    //   assert_eq!(parsed.device_did(), device_kp.public_key().to_did());
    //   assert_eq!(parsed.parent_did(), parent_kp.public_key().to_did());
    //   parsed.verify_signature_with(&parent_kp.public_key()).unwrap();
    //
    // OBSERVABLE consequence: attestation issuance + canonical-bytes
    // round-trip recovers the exact envelope; signature verifies
    // against the parent keypair. Thin-client browser tab uses this
    // path to declare its envelope to the full peer.
    unimplemented!("G14-A2 wires DeviceAttestation::issue() + canonical-bytes round-trip + verify");
}

#[test]
#[ignore = "RED-PHASE: G14-A2 + G14-B — exploration-device-mesh — attestation consumed at chain-walk"]
fn device_attestation_consumed_at_ucan_delegation_chain_walk() {
    // exploration-device-mesh pin. The device attestation is consumed
    // at UCAN delegation chain construction so per-device cap policy
    // can enforce envelope-derived limits (e.g., a device that
    // declared `runs_sandbox=false` cannot sign UCANs that grant
    // `host:sandbox:*` capabilities).
    //
    // Implementer wires:
    //
    //   let parent_kp = benten_id::keypair::Keypair::generate();
    //   let device_kp = benten_id::keypair::Keypair::generate();
    //
    //   let envelope = benten_id::device_attestation::CapabilityEnvelope {
    //       runs_sandbox: false, // Thin client cannot run SANDBOX
    //       ..Default::default()
    //   };
    //   let attestation = benten_id::device_attestation::DeviceAttestation::issue(
    //       &parent_kp, device_kp.public_key().to_did(), envelope).unwrap();
    //
    //   // Device tries to issue a UCAN granting host:sandbox:exec:
    //   let ucan = benten_id::ucan::Ucan::builder()
    //       .issuer(device_kp.public_key().to_did())
    //       .audience(...)
    //       .capability("host:sandbox:exec", "*")
    //       .sign(&device_kp).unwrap();
    //
    //   // Chain-walk consults the attestation log; rejects because
    //   // the device's envelope says runs_sandbox=false:
    //   let err = benten_id::ucan::validate_chain_with_attestations(
    //       &[ucan], &[attestation]).unwrap_err();
    //   assert!(matches!(err,
    //       benten_id::ucan::ChainError::DeviceEnvelopeViolated { .. }));
    //
    // OBSERVABLE consequence: a thin-client device cannot issue UCANs
    // that exceed its declared capability envelope, even when the
    // signature is valid. Defends against the "compromised browser
    // tab tries to delegate sandbox-exec to itself" attack class.
    unimplemented!("G14-A2 + G14-B wires device-envelope-violation rejection at UCAN chain-walk");
}

#[test]
#[ignore = "RED-PHASE: G14-A2 — defect-class — replay-resistant within freshness window"]
fn device_attestation_replay_resistant_within_freshness_window() {
    // device-DID-attestation-replay defect-class pin. An attestation
    // captured on the wire MUST NOT be replay-able outside its
    // freshness window. The window bounds memory-cost of the nonce
    // store while pinning the attack surface.
    //
    // Implementer wires:
    //
    //   let parent_kp = benten_id::keypair::Keypair::generate();
    //   let device_kp = benten_id::keypair::Keypair::generate();
    //   let envelope = benten_id::device_attestation::CapabilityEnvelope::default();
    //
    //   let issuance_secs = 1_000_000_000;
    //   let attestation = benten_id::device_attestation::DeviceAttestation::issue_at(
    //       &parent_kp, device_kp.public_key().to_did(), envelope, issuance_secs).unwrap();
    //
    //   let acceptor = benten_id::device_attestation::Acceptor::new(
    //       benten_id::device_attestation::FreshnessPolicy::seconds(300));
    //
    //   // Within window: accepts.
    //   acceptor.accept_at(&attestation, issuance_secs + 60).unwrap();
    //
    //   // Outside window: rejects:
    //   let err = acceptor.accept_at(&attestation, issuance_secs + 600).unwrap_err();
    //   assert!(matches!(err,
    //       benten_id::device_attestation::AcceptError::FreshnessExpired { .. }));
    //
    // OBSERVABLE consequence: a captured attestation rejected by the
    // freshness gate cannot be replayed by an attacker after the
    // device is decommissioned.
    unimplemented!("G14-A2 wires DeviceAttestation freshness-window enforcement at acceptor");
}

#[test]
#[ignore = "RED-PHASE: G14-A2 — pattern-induction defect-class — nonce + freshness store"]
fn device_attestation_replay_resistance_via_nonce_freshness_window() {
    // pattern-induction unnamed-defect-class pin (composes with
    // `device_attestation_replay_resistant_within_freshness_window`
    // but at a different attack vector: REPLAY WITHIN the freshness
    // window). The nonce-store MUST reject duplicate attestations
    // even within their freshness window.
    //
    // Implementer wires:
    //
    //   let parent_kp = benten_id::keypair::Keypair::generate();
    //   let device_kp = benten_id::keypair::Keypair::generate();
    //   let envelope = benten_id::device_attestation::CapabilityEnvelope::default();
    //
    //   let issuance_secs = 1_000_000_000;
    //   let attestation = benten_id::device_attestation::DeviceAttestation::issue_at(
    //       &parent_kp, device_kp.public_key().to_did(), envelope, issuance_secs).unwrap();
    //
    //   let acceptor = benten_id::device_attestation::Acceptor::new(
    //       benten_id::device_attestation::FreshnessPolicy::seconds(300));
    //
    //   // First presentation: accepts.
    //   acceptor.accept_at(&attestation, issuance_secs + 30).unwrap();
    //   // Second presentation (replay) within window: rejects via nonce store.
    //   let err = acceptor.accept_at(&attestation, issuance_secs + 60).unwrap_err();
    //   assert!(matches!(err,
    //       benten_id::device_attestation::AcceptError::NonceReplay { .. }));
    //
    // OBSERVABLE consequence: nonce-store remembers presentations
    // within the freshness window; a replay during the same window
    // rejects. Defense-in-depth alongside freshness-expiration.
    unimplemented!("G14-A2 wires DeviceAttestation nonce-store rejection of within-window replay");
}

#[test]
#[ignore = "RED-PHASE: G14-A2 — crypto-major-6 — parent emits revocation on device loss"]
fn device_attestation_revocation_emitted_by_parent_did_on_loss_event() {
    // crypto-major-6 pin. When a device is lost / compromised, the
    // parent DID emits a revocation; the revocation propagates to
    // every UCAN backend that holds the device's attestation.
    //
    // Implementer wires:
    //
    //   let parent_kp = benten_id::keypair::Keypair::generate();
    //   let device_kp = benten_id::keypair::Keypair::generate();
    //
    //   let attestation = benten_id::device_attestation::DeviceAttestation::issue(
    //       &parent_kp,
    //       device_kp.public_key().to_did(),
    //       benten_id::device_attestation::CapabilityEnvelope::default(),
    //   ).unwrap();
    //
    //   // Loss event: parent revokes the device:
    //   let revocation = benten_id::device_attestation::DeviceRevocation::issue(
    //       &parent_kp,
    //       device_kp.public_key().to_did(),
    //       benten_id::device_attestation::RevocationReason::DeviceLoss,
    //   ).unwrap();
    //
    //   // Revocation is signed by parent + names device + carries reason:
    //   assert_eq!(revocation.device_did(), device_kp.public_key().to_did());
    //   assert_eq!(revocation.reason(), benten_id::device_attestation::RevocationReason::DeviceLoss);
    //   revocation.verify_signature_with(&parent_kp.public_key()).unwrap();
    //
    //   // The pre-revocation attestation now reads as superseded:
    //   let acceptor = benten_id::device_attestation::Acceptor::new_with_revocations(
    //       benten_id::device_attestation::FreshnessPolicy::seconds(300),
    //       vec![revocation],
    //   );
    //   let err = acceptor.accept(&attestation).unwrap_err();
    //   assert!(matches!(err, benten_id::device_attestation::AcceptError::DeviceRevoked { .. }));
    //
    // OBSERVABLE consequence: post-revocation, the attestation is
    // observably rejected at the acceptor; the revocation chain is
    // signed end-to-end by the parent.
    unimplemented!("G14-A2 wires DeviceRevocation issuance + acceptor rejection of revoked device");
}

#[test]
#[ignore = "RED-PHASE: G14-A2 — crypto-major-6 — revoked device cannot sign new UCANs"]
fn device_attestation_revoked_device_cannot_sign_new_ucan_delegation() {
    // crypto-major-6 pin. Per crypto-major-3 + crypto-major-6, after
    // a device revocation the device's keypair MUST NOT be able to
    // sign new UCAN delegations — even if the attacker still holds
    // the device's secret key.
    //
    // Implementer wires:
    //
    //   let parent_kp = benten_id::keypair::Keypair::generate();
    //   let device_kp = benten_id::keypair::Keypair::generate();
    //
    //   let _attestation = benten_id::device_attestation::DeviceAttestation::issue(
    //       &parent_kp, device_kp.public_key().to_did(),
    //       benten_id::device_attestation::CapabilityEnvelope::default()).unwrap();
    //
    //   let revocation = benten_id::device_attestation::DeviceRevocation::issue(
    //       &parent_kp,
    //       device_kp.public_key().to_did(),
    //       benten_id::device_attestation::RevocationReason::DeviceLoss,
    //   ).unwrap();
    //
    //   // Attacker holds device_kp; tries to issue a new UCAN AFTER revocation:
    //   let post_revoke_ucan = benten_id::ucan::Ucan::builder()
    //       .issuer(device_kp.public_key().to_did())
    //       .audience(...)
    //       .capability("/zone/posts", "read")
    //       .sign(&device_kp).unwrap();
    //
    //   // Chain-walker consults revocation log; rejects:
    //   let err = benten_id::ucan::validate_chain_with_device_revocations(
    //       &[post_revoke_ucan], &[revocation]).unwrap_err();
    //   assert!(matches!(err, benten_id::ucan::ChainError::IssuerDeviceRevoked { .. }));
    //
    // OBSERVABLE consequence: a UCAN signed by a stolen device key
    // post-revocation rejects at validate_chain even though the
    // signature is structurally valid. This closes the "stolen device
    // continues to sign forever" attack class.
    unimplemented!(
        "G14-A2 wires post-revocation device-UCAN rejection at chain-walk via device-revocation log"
    );
}
