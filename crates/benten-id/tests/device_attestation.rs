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
//! - `tests/browser_target_auto_asserts_runs_sandbox_false` — `br-r4-r1-4` / `br-r4-r2-3` MAJOR
//! - `tests/browser_target_with_runs_sandbox_true_claim_rejected_at_attestation_construction_time` — `br-r4-r1-4` / `br-r4-r2-3` MAJOR
//! - `tests/ucan_delegation_to_browser_target_for_sandbox_handler_rejected_at_chain_construction_not_invocation` — `br-r4-r1-4` / `br-r4-r2-3` MAJOR
//!
//! ## Trust-graph forgery attack surface (br-r4-r1-4 / br-r4-r2-3 MAJOR)
//!
//! The 3 composition pins above defend a specific attack surface: a
//! device-DID claiming `runs_sandbox=true` from a browser-target
//! context (where wasmtime is unavailable per Phase-2b
//! `E_SANDBOX_UNAVAILABLE_ON_WASM`) can pollute the trust graph + the
//! routing layer dispatches to a target that fails at runtime, but
//! the trust graph believes the routing was correct. The fail must
//! land at ATTESTATION CONSTRUCTION TIME (or chain-construction
//! time for derived UCANs) — NOT at invocation time — so the trust
//! graph never sees the malformed envelope.
//!
//! The typed error `E_DEVICE_ATTESTATION_INCOMPATIBLE_WITH_RUNTIME`
//! fires at the attestation-acceptor + UCAN chain-walker — minted at
//! G14-A1 wave-4a implementation time per the test-pin contract
//! (RED-PHASE drives implementer; orchestrator does not pre-empt the
//! enum mint).
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

#[test]
#[ignore = "RED-PHASE: G14-A2 — cap-r4-7 — envelope must be attenuated by parent DID"]
fn device_attestation_envelope_must_be_attenuated_by_parent_did() {
    // cap-r4-7 pin (cap-major-4 capability-system structural closure).
    // The device-DID attestation envelope MUST be a structurally-
    // attenuated UCAN claim: issuer = parent identity-DID; subject =
    // device-DID; capabilities subset of parent's capability set.
    //
    // Implementer wires:
    //
    //   let parent_kp = benten_id::keypair::Keypair::generate();
    //   let device_kp = benten_id::keypair::Keypair::generate();
    //
    //   // Parent has a constrained authority set (e.g., not host:sandbox:exec):
    //   let parent_authority = benten_id::ucan::AuthoritySet::new()
    //       .add_capability("/zone/posts", "read")
    //       .add_capability("/zone/posts", "write");
    //
    //   // Issue attestation: envelope MUST be subset of parent_authority:
    //   let envelope = benten_id::device_attestation::CapabilityEnvelope {
    //       runs_sandbox: false, // parent doesn't have host:sandbox:exec, so this is consistent
    //       holds_zones: benten_id::device_attestation::ZoneScope::Specific(vec!["/zone/posts".into()]),
    //       ..Default::default()
    //   };
    //   let attestation = benten_id::device_attestation::DeviceAttestation::issue_with_authority(
    //       &parent_kp, device_kp.public_key().to_did(), envelope, parent_authority).unwrap();
    //
    //   // Chain-walk: attestation envelope is structurally attenuated:
    //   let chain = benten_id::ucan::AttenuationChain::from_attestation(&attestation);
    //   assert_eq!(chain.issuer(), parent_kp.public_key().to_did());
    //   assert_eq!(chain.subject(), device_kp.public_key().to_did());
    //   assert!(chain.is_subset_of(&parent_authority),
    //       "device attestation envelope MUST be subset of parent authority per cap-r4-7");
    //
    // OBSERVABLE consequence: attestation envelopes are structurally
    // tied to parent's capability set; future chain-walkers can
    // verify attenuation just like any other UCAN delegation. Defends
    // against the "compromised device self-attests with arbitrary
    // capabilities" attack class.
    unimplemented!(
        "G14-A2 wires structural attenuation: attestation envelope subset of parent authority per cap-r4-7"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-A2 — cap-r4-7 — widening parent authority is rejected"]
fn device_attestation_widening_parent_authority_is_rejected() {
    // cap-r4-7 pin (cap-major-4 closure, attenuation widening case).
    // An attestation envelope claiming wider authority than the parent
    // chain rejects at chain-walk with a typed envelope-widening error.
    //
    // Implementer wires:
    //
    //   let parent_kp = benten_id::keypair::Keypair::generate();
    //   let device_kp = benten_id::keypair::Keypair::generate();
    //
    //   // Parent: only /zone/posts read.
    //   let parent_authority = benten_id::ucan::AuthoritySet::new()
    //       .add_capability("/zone/posts", "read");
    //
    //   // Adversarial envelope: claims write on /zone/admin (widening!):
    //   let widening_envelope = benten_id::device_attestation::CapabilityEnvelope {
    //       runs_sandbox: true, // parent has no sandbox authority
    //       holds_zones: benten_id::device_attestation::ZoneScope::Full, // wider than parent's specific zone
    //       ..Default::default()
    //   };
    //
    //   // Issuance must reject (or chain-walk must reject the envelope):
    //   let result = benten_id::device_attestation::DeviceAttestation::issue_with_authority(
    //       &parent_kp, device_kp.public_key().to_did(),
    //       widening_envelope, parent_authority);
    //   assert!(matches!(result.unwrap_err(),
    //       benten_id::device_attestation::IssueError::EnvelopeWidening { .. }),
    //       "issue must reject envelope widening per cap-r4-7");
    //
    // OBSERVABLE consequence: a parent-DID cannot issue an attestation
    // claiming wider authority than the parent itself holds. Defends
    // against the "compromised parent issues over-broad device
    // attestation" failure shape at the issuance gate.
    unimplemented!(
        "G14-A2 wires envelope-widening rejection at attestation issuance + chain-walk per cap-r4-7"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-A2 — cap-r4-7 — runs_sandbox=false cannot be widened by device-signed re-attestation"]
fn device_attestation_runs_sandbox_false_cannot_be_widened_by_device_signed_re_attestation() {
    // cap-r4-7 pin (cap-major-4 closure, self-re-attestation case).
    // A compromised device that holds the device-keypair MUST NOT be
    // able to publish a self-signed attestation with widened envelope
    // (runs_sandbox=true when parent issued runs_sandbox=false).
    //
    // Implementer wires:
    //
    //   let parent_kp = benten_id::keypair::Keypair::generate();
    //   let device_kp = benten_id::keypair::Keypair::generate();
    //
    //   // Parent issues attestation with runs_sandbox=false:
    //   let constrained_envelope = benten_id::device_attestation::CapabilityEnvelope {
    //       runs_sandbox: false,
    //       ..Default::default()
    //   };
    //   let _legit_attestation = benten_id::device_attestation::DeviceAttestation::issue(
    //       &parent_kp, device_kp.public_key().to_did(), constrained_envelope).unwrap();
    //
    //   // Adversarial path: compromised device tries to self-sign a
    //   // wider envelope. The chain-walker must reject because the
    //   // issuer (device_kp) is not the parent_kp.
    //   let widened_envelope = benten_id::device_attestation::CapabilityEnvelope {
    //       runs_sandbox: true, // widened
    //       ..Default::default()
    //   };
    //   let self_signed_widening = benten_id::device_attestation::DeviceAttestation::issue(
    //       &device_kp, // SELF-issued, not parent-issued
    //       device_kp.public_key().to_did(),
    //       widened_envelope).unwrap();
    //
    //   // Acceptor consults parent-DID lookup; rejects self-signed envelope:
    //   let acceptor = benten_id::device_attestation::Acceptor::with_parent_lookup(
    //       parent_kp.public_key().to_did());
    //   let err = acceptor.accept(&self_signed_widening).unwrap_err();
    //   assert!(matches!(err,
    //       benten_id::device_attestation::AcceptError::IssuerNotParent { .. })
    //         || matches!(err,
    //       benten_id::device_attestation::AcceptError::EnvelopeWidening { .. }),
    //       "self-signed widening attestation MUST reject per cap-r4-7");
    //
    // OBSERVABLE consequence: a stolen device-keypair cannot widen
    // its own envelope by self-signing. Defends against the most
    // severe compromise scenario (device key extraction + attempted
    // privilege escalation).
    unimplemented!(
        "G14-A2 wires acceptor rejection of device-self-signed widening re-attestation per cap-r4-7"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-A2 — sec-r4r1-6 — capability-envelope downgrade attack blocked"]
fn device_attestation_capability_envelope_downgrade_attack_blocked_by_runtime_recheck_against_parent_chain()
 {
    // sec-r4r1-6 pin (multi-device-identity attack family). A device
    // that legitimately claims FEWER capabilities than parent envelope
    // grants (legitimate downgrade), then attempts to invoke a
    // NOT-claimed capability MUST be blocked by runtime re-check
    // against the parent UCAN chain.
    //
    // Implementer wires:
    //
    //   let parent_kp = benten_id::keypair::Keypair::generate();
    //   let device_kp = benten_id::keypair::Keypair::generate();
    //
    //   // Parent grants device a minimal envelope (e.g., read-only):
    //   let downgrade_envelope = benten_id::device_attestation::CapabilityEnvelope {
    //       runs_sandbox: false,
    //       holds_zones: benten_id::device_attestation::ZoneScope::CacheOnly,
    //       ..Default::default()
    //   };
    //   let attestation = benten_id::device_attestation::DeviceAttestation::issue(
    //       &parent_kp, device_kp.public_key().to_did(), downgrade_envelope).unwrap();
    //
    //   // Device attempts to invoke host:sandbox:exec (NOT in envelope):
    //   let invocation_ucan = benten_id::ucan::Ucan::builder()
    //       .issuer(device_kp.public_key().to_did())
    //       .audience(...)
    //       .capability("host:sandbox:exec", "*")
    //       .sign(&device_kp).unwrap();
    //
    //   // Runtime re-check: chain-walker consults attestation envelope;
    //   // rejects because runs_sandbox=false:
    //   let err = benten_id::ucan::validate_chain_with_attestations(
    //       &[invocation_ucan], &[attestation]).unwrap_err();
    //   assert!(matches!(err,
    //       benten_id::ucan::ChainError::DeviceEnvelopeViolated { .. })
    //         || matches!(err,
    //       benten_id::ucan::ChainError::CapabilityNotInEnvelope { .. }),
    //       "device cannot invoke capability outside its own envelope per sec-r4r1-6");
    //
    // OBSERVABLE consequence: a device with a constrained envelope
    // cannot bypass that constraint by signing UCANs that grant
    // capabilities it doesn't hold. Defends against the
    // multi-device-identity envelope-downgrade attack family
    // (sec-r1-7 / sec-r4r1-6).
    unimplemented!(
        "G14-A2 wires runtime envelope-vs-invocation re-check at chain-walk per sec-r4r1-6"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-A2 — br-r4-r1-4 / br-r4-r2-3 MAJOR — browser-target attestation auto-asserts runs_sandbox=false"]
fn browser_target_auto_asserts_runs_sandbox_false() {
    // br-r4-r1-4 / br-r4-r2-3 MAJOR pin (composition of D-PHASE-3-25
    // heterogeneity contract + CLAUDE.md baked-in #17 thin-client
    // commitment). When a browser-target context constructs a
    // DeviceAttestation, the envelope MUST auto-assert
    // `runs_sandbox=false`. The browser target has no wasmtime; a
    // truthful envelope cannot claim runs_sandbox=true on this
    // platform.
    //
    // G14-A2 implementer wires this:
    //
    //   // Browser-target context (cfg-gated to wasm32-unknown-unknown
    //   // OR via a runtime-detected target enum):
    //   let parent_kp = benten_id::keypair::Keypair::generate();
    //   let device_kp = benten_id::keypair::Keypair::generate();
    //
    //   // Use the browser-target convenience constructor:
    //   let attestation = benten_id::device_attestation::DeviceAttestation::issue_for_browser_target(
    //       &parent_kp,
    //       device_kp.public_key().to_did(),
    //   ).unwrap();
    //
    //   // The envelope MUST have runs_sandbox=false (auto-asserted):
    //   assert_eq!(attestation.envelope().runs_sandbox, false,
    //       "browser-target attestation MUST auto-assert runs_sandbox=false \
    //        per CLAUDE.md baked-in #17 (wasmtime unavailable on \
    //        wasm32-unknown-unknown per Phase-2b E_SANDBOX_UNAVAILABLE_ON_WASM)");
    //
    //   // Other browser-target minimum-envelope auto-assertions:
    //   assert_eq!(attestation.envelope().holds_zones,
    //       benten_id::device_attestation::ZoneScope::CacheOnly,
    //       "browser-target auto-asserts holds_zones=CacheOnly per baked-in #17");
    //   assert_eq!(attestation.envelope().runs_atrium_peer, false,
    //       "browser-target auto-asserts runs_atrium_peer=false per baked-in #17");
    //   assert_eq!(attestation.envelope().online_uptime,
    //       benten_id::device_attestation::UptimePolicy::SessionBounded,
    //       "browser-target auto-asserts online_uptime=SessionBounded per baked-in #17");
    //
    // OBSERVABLE consequence: the browser-target convenience constructor
    // produces a truthful envelope that the trust graph can route
    // against without surprises. Defends against the failure shape
    // where browser-target code accidentally claims wider capability
    // than the platform supports.
    unimplemented!(
        "G14-A2 wires DeviceAttestation::issue_for_browser_target() auto-asserting \
         runs_sandbox=false + holds_zones=CacheOnly + runs_atrium_peer=false + \
         online_uptime=SessionBounded per CLAUDE.md baked-in #17 + D-PHASE-3-25"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-A2 — br-r4-r1-4 / br-r4-r2-3 MAJOR — browser-target with runs_sandbox=true claim rejected at attestation-construction time"]
fn browser_target_with_runs_sandbox_true_claim_rejected_at_attestation_construction_time() {
    // br-r4-r1-4 / br-r4-r2-3 MAJOR pin (trust-graph forgery defense).
    // When code on a browser target attempts to construct a
    // DeviceAttestation with `runs_sandbox=true`, the construction
    // MUST fail with the typed error
    // `E_DEVICE_ATTESTATION_INCOMPATIBLE_WITH_RUNTIME`. The fail
    // lands at construction time (NOT at invocation time) so the
    // trust graph never receives a forged envelope.
    //
    // G14-A2 implementer wires this:
    //
    //   let parent_kp = benten_id::keypair::Keypair::generate();
    //   let device_kp = benten_id::keypair::Keypair::generate();
    //
    //   // Adversarial / mistaken envelope: claims runs_sandbox=true
    //   // from a browser-target context where wasmtime is unavailable.
    //   let envelope = benten_id::device_attestation::CapabilityEnvelope {
    //       runs_sandbox: true, // INCOMPATIBLE with browser target
    //       ..Default::default()
    //   };
    //
    //   // The construction call must observe the runtime context
    //   // (cfg-gate or RuntimeTarget probe) + reject:
    //   let result = benten_id::device_attestation::DeviceAttestation::issue_with_runtime_check(
    //       &parent_kp,
    //       device_kp.public_key().to_did(),
    //       envelope,
    //       benten_id::device_attestation::RuntimeTarget::Browser,
    //   );
    //
    //   let err = result.unwrap_err();
    //   assert!(
    //       matches!(err,
    //           benten_id::device_attestation::IssueError::IncompatibleWithRuntime { .. }),
    //       "browser-target + runs_sandbox=true MUST reject at construction \
    //        with IssueError::IncompatibleWithRuntime per br-r4-r1-4");
    //
    //   // The error code is the typed catalog code minted at G14-A1:
    //   assert_eq!(err.code().as_str(), "E_DEVICE_ATTESTATION_INCOMPATIBLE_WITH_RUNTIME",
    //       "the typed error code must be the canonical E_DEVICE_ATTESTATION_INCOMPATIBLE_WITH_RUNTIME \
    //        per ERROR-CATALOG.md");
    //
    // OBSERVABLE consequence: a browser-target context cannot
    // construct an attestation that lies about its sandbox capability.
    // The typed error code is operator-actionable + carries forward
    // through the napi binding so JS callers get a typed catch.
    // Defends the trust-graph forgery attack surface explicitly.
    //
    // NOTE: implementer-added typed error mint
    // `ErrorCode::DeviceAttestationIncompatibleWithRuntime` (catalog
    // string `"E_DEVICE_ATTESTATION_INCOMPATIBLE_WITH_RUNTIME"`)
    // lands at G14-A1 wave-4a — driven by this test pin per pim-2
    // §3.6b end-to-end shape (test names the typed error as the
    // observable consequence; implementer wires the mint).
    unimplemented!(
        "G14-A2 wires DeviceAttestation::issue_with_runtime_check rejection at construction \
         time when browser-target + runs_sandbox=true; typed error \
         E_DEVICE_ATTESTATION_INCOMPATIBLE_WITH_RUNTIME minted at G14-A1"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-A2 + G14-B — br-r4-r1-4 / br-r4-r2-3 MAJOR — UCAN delegation TO browser-target FOR sandbox handler rejected at chain-construction"]
fn ucan_delegation_to_browser_target_for_sandbox_handler_rejected_at_chain_construction_not_invocation()
 {
    // br-r4-r1-4 / br-r4-r2-3 MAJOR pin (chain-construction-time
    // rejection, not invocation-time). When a UCAN issuer attempts
    // to delegate `host:sandbox:exec` capability TO a device-DID
    // whose attestation envelope says `runs_sandbox=false` (e.g., a
    // browser-target device), the delegation MUST reject at CHAIN
    // CONSTRUCTION time — NOT at invocation time. Otherwise the
    // UCAN sits in the trust graph as if valid; the failure surfaces
    // only at the runtime SANDBOX dispatch where the routing layer
    // discovers it cannot dispatch (and the trust-graph integrity
    // is compromised in the meantime).
    //
    // G14-A2 + G14-B implementer wires this:
    //
    //   let parent_kp = benten_id::keypair::Keypair::generate();
    //   let browser_device_kp = benten_id::keypair::Keypair::generate();
    //
    //   // Browser-target attestation says runs_sandbox=false:
    //   let attestation = benten_id::device_attestation::DeviceAttestation::issue_for_browser_target(
    //       &parent_kp, browser_device_kp.public_key().to_did()).unwrap();
    //
    //   // Adversarial / mistaken UCAN: delegates host:sandbox:exec
    //   // to the browser device's DID:
    //   let attempted_ucan_build = benten_id::ucan::Ucan::builder()
    //       .issuer(parent_kp.public_key().to_did())
    //       .audience(browser_device_kp.public_key().to_did())
    //       .capability("host:sandbox:exec", "*")
    //       .with_attestation_lookup(&[attestation.clone()])
    //       .sign(&parent_kp);
    //
    //   // The delegation chain construction must reject because the
    //   // audience's attestation says runs_sandbox=false:
    //   let err = attempted_ucan_build.unwrap_err();
    //   assert!(matches!(err,
    //       benten_id::ucan::DelegationError::AudienceEnvelopeIncompatibleWithCapability { .. }),
    //       "UCAN delegating host:sandbox:exec to browser-target audience MUST \
    //        reject at chain-construction (NOT invocation) per br-r4-r1-4");
    //
    //   assert_eq!(err.code().as_str(), "E_DEVICE_ATTESTATION_INCOMPATIBLE_WITH_RUNTIME",
    //       "the typed error code must be E_DEVICE_ATTESTATION_INCOMPATIBLE_WITH_RUNTIME");
    //
    //   // CRITICAL: the rejected UCAN MUST NOT have been added to the
    //   // durable UCAN backend (the backend is the trust graph; we
    //   // never want a malformed delegation persisted):
    //   let backend = benten_id::ucan::DurableBackend::test_instance();
    //   assert!(backend.list_grants_to_audience(&browser_device_kp.public_key().to_did())
    //       .iter().all(|g| g.capability() != "host:sandbox:exec"),
    //       "rejected UCAN MUST NOT have been persisted to durable backend; \
    //        chain-construction rejection happens BEFORE persistence");
    //
    // OBSERVABLE consequence: a UCAN whose audience cannot fulfill
    // the delegated capability never enters the trust graph. The
    // routing layer never has the opportunity to dispatch to a target
    // that fails at runtime. Closes the trust-graph forgery surface
    // br-r4-r1-4 named.
    unimplemented!(
        "G14-A2 + G14-B wires UCAN chain-construction-time rejection of delegations to \
         audience-with-incompatible-attestation-envelope per br-r4-r1-4 + D-PHASE-3-25"
    );
}
