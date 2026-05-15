//! G14-A2 wave-4a' — device-DID capability-attestation test pins
//! (un-ignored except where noted).
//!
//! Pin sources (per `crypto-major-6` + `br-r4-r1-4` / `br-r4-r2-3`
//! MAJOR + the device-DID-attestation-replay defect-class +
//! `pim-r1-pim-induction-7` + `cap-r4-7`):
//!
//! - `device_attestation_round_trip`
//! - `device_attestation_consumed_at_ucan_delegation_chain_walk`
//! - `device_attestation_replay_resistant_within_freshness_window`
//! - `device_attestation_replay_resistance_via_nonce_freshness_window`
//! - `device_attestation_revocation_emitted_by_parent_did_on_loss_event`
//! - `device_attestation_revoked_device_cannot_sign_new_ucan_delegation`
//! - `device_attestation_envelope_must_be_attenuated_by_parent_did`
//! - `device_attestation_widening_parent_authority_is_rejected`
//! - `device_attestation_runs_sandbox_false_cannot_be_widened_by_device_signed_re_attestation`
//! - `device_attestation_capability_envelope_downgrade_attack_blocked_by_runtime_recheck_against_parent_chain`
//! - `browser_target_auto_asserts_runs_sandbox_false`
//! - `browser_target_with_runs_sandbox_true_claim_rejected_at_attestation_construction_time`
//! - `ucan_delegation_to_browser_target_for_sandbox_handler_rejected_at_chain_construction_not_invocation` — RED-PHASE (G14-B integration)

#![allow(clippy::unwrap_used)]

use benten_id::device_attestation::{
    Acceptor, CapabilityEnvelope, DeviceAttestation, DeviceRevocation, FreshnessPolicy,
    RevocationReason, RuntimeTarget, UptimePolicy, ZoneScope,
};
use benten_id::keypair::Keypair;
use benten_id::ucan::{
    Ucan, validate_chain_with_attestations, validate_chain_with_device_revocations,
};
use benten_id::{DeviceAttestationError, UcanError};

#[test]
fn device_attestation_round_trip() {
    // plan §3 G14-A2 — issuance + canonical-bytes round-trip + verify.
    let parent = Keypair::generate();
    let device = Keypair::generate();

    let envelope = CapabilityEnvelope {
        runs_sandbox: false,
        holds_zones: ZoneScope::CacheOnly,
        online_uptime: UptimePolicy::SessionBounded,
        runs_atrium_peer: false,
    };
    let attestation =
        DeviceAttestation::issue(&parent, device.public_key().to_did(), envelope.clone()).unwrap();

    // Canonical-bytes round-trip:
    let bytes = attestation.canonical_bytes();
    let parsed = DeviceAttestation::from_canonical_bytes(&bytes).unwrap();
    assert_eq!(parsed.envelope(), &envelope);
    assert_eq!(
        parsed.device_did().as_str(),
        device.public_key().to_did().as_str()
    );
    assert_eq!(
        parsed.parent_did().as_str(),
        parent.public_key().to_did().as_str()
    );

    // Signature verifies against the parent keypair:
    parsed.verify_signature_with(parent.public_key()).unwrap();

    // A different keypair fails:
    assert!(matches!(
        parsed
            .verify_signature_with(device.public_key())
            .unwrap_err(),
        DeviceAttestationError::BadSignature
    ));
}

#[test]
fn device_attestation_consumed_at_ucan_delegation_chain_walk() {
    // exploration-device-mesh — chain-walker rejects UCANs that
    // exceed the device's declared envelope.
    let parent = Keypair::generate();
    let device = Keypair::generate();
    let leaf_aud = Keypair::generate();

    let envelope = CapabilityEnvelope {
        runs_sandbox: false,
        ..CapabilityEnvelope::default()
    };
    let attestation =
        DeviceAttestation::issue(&parent, device.public_key().to_did(), envelope).unwrap();

    // Device tries to issue a UCAN granting host:sandbox:exec:
    let ucan = Ucan::builder()
        .issuer(device.public_key().to_did().as_str())
        .audience(leaf_aud.public_key().to_did().as_str())
        .capability("host:sandbox:exec", "*")
        .not_before(0)
        .expiry(u64::MAX)
        .sign(&device);

    let err = validate_chain_with_attestations(&[ucan], &[attestation]).unwrap_err();
    assert!(
        matches!(err, UcanError::DeviceEnvelopeViolated { .. }),
        "{err:?}"
    );
}

#[test]
fn device_attestation_replay_resistant_within_freshness_window() {
    // device-DID-attestation-replay defect-class — freshness window.
    let parent = Keypair::generate();
    let device = Keypair::generate();

    let issuance_secs: u64 = 1_000_000_000;
    let attestation = DeviceAttestation::issue_at(
        &parent,
        device.public_key().to_did(),
        CapabilityEnvelope::default(),
        issuance_secs,
    )
    .unwrap();

    let acceptor = Acceptor::new(FreshnessPolicy::seconds(300));

    // Within window: accepts.
    acceptor
        .accept_at(&attestation, issuance_secs + 60)
        .unwrap();

    // Outside window: rejects (use a fresh attestation to dodge the
    // nonce-store double-spend rejection from the prior accept).
    let attestation2 = DeviceAttestation::issue_at(
        &parent,
        device.public_key().to_did(),
        CapabilityEnvelope::default(),
        issuance_secs,
    )
    .unwrap();
    let acceptor2 = Acceptor::new(FreshnessPolicy::seconds(300));
    let err = acceptor2
        .accept_at(&attestation2, issuance_secs + 600)
        .unwrap_err();
    assert!(
        matches!(err, DeviceAttestationError::FreshnessExpired { .. }),
        "{err:?}"
    );
}

#[test]
fn device_attestation_replay_resistance_via_nonce_freshness_window() {
    // pattern-induction unnamed-defect-class — nonce store rejects
    // duplicate attestations within freshness window.
    let parent = Keypair::generate();
    let device = Keypair::generate();

    let issuance_secs: u64 = 1_000_000_000;
    let attestation = DeviceAttestation::issue_at(
        &parent,
        device.public_key().to_did(),
        CapabilityEnvelope::default(),
        issuance_secs,
    )
    .unwrap();

    let acceptor = Acceptor::new(FreshnessPolicy::seconds(300));

    // First presentation: accepts.
    acceptor
        .accept_at(&attestation, issuance_secs + 30)
        .unwrap();

    // Second presentation (replay) within window: rejects via nonce store.
    let err = acceptor
        .accept_at(&attestation, issuance_secs + 60)
        .unwrap_err();
    assert!(
        matches!(err, DeviceAttestationError::NonceReplay),
        "{err:?}"
    );
}

#[test]
fn device_attestation_revocation_emitted_by_parent_did_on_loss_event() {
    // crypto-major-6 — parent revokes device on loss event.
    let parent = Keypair::generate();
    let device = Keypair::generate();

    let attestation = DeviceAttestation::issue(
        &parent,
        device.public_key().to_did(),
        CapabilityEnvelope::default(),
    )
    .unwrap();

    let revocation = DeviceRevocation::issue(
        &parent,
        device.public_key().to_did(),
        RevocationReason::DeviceLoss,
    )
    .unwrap();

    assert_eq!(
        revocation.device_did().as_str(),
        device.public_key().to_did().as_str()
    );
    assert_eq!(revocation.reason(), RevocationReason::DeviceLoss);
    revocation
        .verify_signature_with(parent.public_key())
        .unwrap();

    // Pre-revocation attestation now reads as superseded:
    let acceptor =
        Acceptor::new_with_revocations(FreshnessPolicy::seconds(u64::MAX), vec![revocation]);
    let err = acceptor.accept(&attestation).unwrap_err();
    assert!(
        matches!(err, DeviceAttestationError::DeviceRevoked { .. }),
        "{err:?}"
    );
}

#[test]
fn device_attestation_revoked_device_cannot_sign_new_ucan_delegation() {
    // crypto-major-6 — revoked device cannot sign new UCANs.
    let parent = Keypair::generate();
    let device = Keypair::generate();
    let leaf_aud = Keypair::generate();

    let _attestation = DeviceAttestation::issue(
        &parent,
        device.public_key().to_did(),
        CapabilityEnvelope::default(),
    )
    .unwrap();

    let revocation = DeviceRevocation::issue(
        &parent,
        device.public_key().to_did(),
        RevocationReason::DeviceLoss,
    )
    .unwrap();

    // Attacker holds device kp; tries to issue post-revocation UCAN:
    let post_revoke_ucan = Ucan::builder()
        .issuer(device.public_key().to_did().as_str())
        .audience(leaf_aud.public_key().to_did().as_str())
        .capability("/zone/posts", "read")
        .not_before(0)
        .expiry(u64::MAX)
        .sign(&device);

    let err =
        validate_chain_with_device_revocations(&[post_revoke_ucan], &[revocation]).unwrap_err();
    assert!(
        matches!(err, UcanError::IssuerDeviceRevoked { .. }),
        "{err:?}"
    );
}

#[test]
fn device_attestation_envelope_must_be_attenuated_by_parent_did() {
    // cap-r4-7 — parent-authority envelope check.
    let parent = Keypair::generate();
    let device = Keypair::generate();

    let parent_authority = CapabilityEnvelope {
        runs_sandbox: false,
        holds_zones: ZoneScope::Specific(vec!["/zone/posts".into()]),
        online_uptime: UptimePolicy::AlwaysOn,
        runs_atrium_peer: true,
    };

    let consistent_envelope = CapabilityEnvelope {
        runs_sandbox: false,
        holds_zones: ZoneScope::Specific(vec!["/zone/posts".into()]),
        online_uptime: UptimePolicy::SessionBounded,
        runs_atrium_peer: false,
    };

    DeviceAttestation::issue_with_authority(
        &parent,
        device.public_key().to_did(),
        consistent_envelope,
        &parent_authority,
    )
    .unwrap();
}

#[test]
fn device_attestation_widening_parent_authority_is_rejected() {
    // cap-r4-7 — envelope widening rejected at issuance.
    let parent = Keypair::generate();
    let device = Keypair::generate();

    let parent_authority = CapabilityEnvelope {
        runs_sandbox: false,
        holds_zones: ZoneScope::Specific(vec!["/zone/posts".into()]),
        online_uptime: UptimePolicy::AlwaysOn,
        runs_atrium_peer: true,
    };

    let widening_envelope = CapabilityEnvelope {
        runs_sandbox: true,
        holds_zones: ZoneScope::Full,
        online_uptime: UptimePolicy::AlwaysOn,
        runs_atrium_peer: true,
    };

    let err = DeviceAttestation::issue_with_authority(
        &parent,
        device.public_key().to_did(),
        widening_envelope,
        &parent_authority,
    )
    .unwrap_err();
    assert!(
        matches!(err, DeviceAttestationError::EnvelopeWidening { .. }),
        "{err:?}"
    );
}

#[test]
fn device_attestation_runs_sandbox_false_cannot_be_widened_by_device_signed_re_attestation() {
    // cap-r4-7 — self-re-attestation rejected at acceptor parent
    // lookup.
    let parent = Keypair::generate();
    let device = Keypair::generate();

    // Compromised device tries to self-sign a wider envelope:
    let widened = CapabilityEnvelope {
        runs_sandbox: true,
        ..CapabilityEnvelope::default()
    };
    let self_signed = DeviceAttestation::issue(
        &device, // SELF-issued, not parent-issued
        device.public_key().to_did(),
        widened,
    )
    .unwrap();

    let acceptor = Acceptor::with_parent_lookup(parent.public_key().to_did());
    let err = acceptor.accept(&self_signed).unwrap_err();
    assert!(
        matches!(err, DeviceAttestationError::IssuerNotParent { .. }),
        "{err:?}"
    );
}

#[test]
fn device_attestation_capability_envelope_downgrade_attack_blocked_by_runtime_recheck_against_parent_chain()
 {
    // sec-r4r1-6 — runtime re-check against attestation envelope.
    let parent = Keypair::generate();
    let device = Keypair::generate();
    let leaf_aud = Keypair::generate();

    let downgrade_envelope = CapabilityEnvelope {
        runs_sandbox: false,
        holds_zones: ZoneScope::CacheOnly,
        ..CapabilityEnvelope::default()
    };
    let attestation =
        DeviceAttestation::issue(&parent, device.public_key().to_did(), downgrade_envelope)
            .unwrap();

    // Device attempts host:sandbox:exec (NOT in envelope):
    let invocation_ucan = Ucan::builder()
        .issuer(device.public_key().to_did().as_str())
        .audience(leaf_aud.public_key().to_did().as_str())
        .capability("host:sandbox:exec", "*")
        .not_before(0)
        .expiry(u64::MAX)
        .sign(&device);

    let err = validate_chain_with_attestations(&[invocation_ucan], &[attestation]).unwrap_err();
    assert!(
        matches!(err, UcanError::DeviceEnvelopeViolated { .. }),
        "{err:?}"
    );
}

#[test]
fn browser_target_auto_asserts_runs_sandbox_false() {
    // br-r4-r1-4 / br-r4-r2-3 MAJOR — browser-target convenience
    // constructor auto-asserts minimum-capability envelope per
    // CLAUDE.md baked-in #17.
    let parent = Keypair::generate();
    let device = Keypair::generate();

    let attestation =
        DeviceAttestation::issue_for_browser_target(&parent, device.public_key().to_did()).unwrap();

    assert!(!attestation.envelope().runs_sandbox);
    assert_eq!(attestation.envelope().holds_zones, ZoneScope::CacheOnly);
    assert!(!attestation.envelope().runs_atrium_peer);
    assert_eq!(
        attestation.envelope().online_uptime,
        UptimePolicy::SessionBounded
    );
}

#[test]
fn browser_target_with_runs_sandbox_true_claim_rejected_at_attestation_construction_time() {
    // br-r4-r1-4 / br-r4-r2-3 MAJOR — typed
    // E_DEVICE_ATTESTATION_INCOMPATIBLE_WITH_RUNTIME at construction.
    let parent = Keypair::generate();
    let device = Keypair::generate();

    let envelope = CapabilityEnvelope {
        runs_sandbox: true,
        ..CapabilityEnvelope::default()
    };

    let err = DeviceAttestation::issue_with_runtime_check(
        &parent,
        device.public_key().to_did(),
        envelope,
        RuntimeTarget::Browser,
    )
    .unwrap_err();

    assert!(
        matches!(err, DeviceAttestationError::IncompatibleWithRuntime { .. }),
        "{err:?}"
    );
    assert_eq!(err.code(), "E_DEVICE_ATTESTATION_INCOMPATIBLE_WITH_RUNTIME");

    // Native target: same envelope succeeds.
    let envelope2 = CapabilityEnvelope {
        runs_sandbox: true,
        ..CapabilityEnvelope::default()
    };
    DeviceAttestation::issue_with_runtime_check(
        &parent,
        device.public_key().to_did(),
        envelope2,
        RuntimeTarget::Native,
    )
    .unwrap();
}

#[test]
#[ignore = "phase-3-backlog §2.1-followup `ssi` external UCAN/VC spec compatibility re-evaluation at G16 Atrium handshake — production prerequisite NOT YET shipped at HEAD. The `Ucan::builder().with_attestation_lookup(...)` construction-time gate + `DelegationError::AudienceEnvelopeIncompatibleWithCapability` typed error do NOT exist (only mentioned in `crates/benten-id/src/ucan.rs:33-37` doc comment; no symbol). `validate_chain_with_attestations` (validate-side seam) DOES exist at `crates/benten-id/src/ucan.rs:602` — runtime gate before trust-graph dispatch. Construction-time rejection composes with §2.1-followup re-evaluation outcome (G16-D wave-6b PR #163 shipped 2026-05-09; cryptography-reviewer dispatch pending). Un-ignore at §2.1-followup re-evaluation that determines `ssi` integration is needed (or, if hand-rolled remains, the `with_attestation_lookup` chain-construction-time path lands as a benten-id-internal extension)."]
fn ucan_delegation_to_browser_target_for_sandbox_handler_rejected_at_chain_construction_not_invocation()
 {
    // br-r4-r1-4 / br-r4-r2-3 MAJOR cross-wave pin. The chain-
    // construction-time rejection requires the Ucan::builder() path to
    // carry a `with_attestation_lookup` argument that consults the
    // durable backend at sign() time + emits a typed
    // DelegationError::AudienceEnvelopeIncompatibleWithCapability. That
    // wiring lives at the durable-backend seam (G14-B); G14-A2's
    // ucan.rs pins the validate-side seam
    // (validate_chain_with_attestations) which is the runtime gate
    // before the trust-graph dispatch. G14-B closes the construction-
    // time gate end-to-end.
    unreachable!("G14-B + G14-C wires this pin");
}

#[test]
fn acceptor_rejects_attestation_with_forged_signature() {
    // g14-a2-mr-1 MAJOR pin. Without signature verification inside
    // `Acceptor::accept_at`, a forged attestation with valid (nonce,
    // freshness, parent_did) but corrupted signature would pass
    // acceptance — a footgun-shaped surface. This test pins the
    // signature-verification gate END-TO-END per pim-2 §3.6b.
    let parent = Keypair::generate();
    let device = Keypair::generate();

    let envelope = CapabilityEnvelope {
        runs_sandbox: false,
        holds_zones: ZoneScope::CacheOnly,
        online_uptime: UptimePolicy::SessionBounded,
        runs_atrium_peer: false,
    };
    let mut attestation =
        DeviceAttestation::issue(&parent, device.public_key().to_did(), envelope).unwrap();

    // Mutate the signature — flip a single bit so the (nonce,
    // freshness, parent_did) gates pass but the signature gate must
    // reject.
    assert_eq!(attestation.signature.len(), 64, "Ed25519 sig is 64 bytes");
    attestation.signature[0] ^= 0x01;

    let acceptor = Acceptor::new(FreshnessPolicy::seconds(u64::MAX));
    let err = acceptor.accept(&attestation).unwrap_err();
    assert!(
        matches!(err, DeviceAttestationError::BadSignature),
        "expected BadSignature, got {err:?}"
    );
}

#[test]
fn envelope_widens_zone_scope_matrix() {
    // g14-a2-mr-6 MINOR pin. Exercise all 9 (parent, child) zone-scope
    // combinations + verify edge cases (empty Specific parent + child
    // wanting Full / Specific) per mini-review fix-pass.
    use benten_id::device_attestation::CapabilityEnvelope;

    let parent_keypair = Keypair::generate();
    let device_keypair = Keypair::generate();

    fn try_issue(
        parent: &Keypair,
        device: &Keypair,
        parent_zones: ZoneScope,
        child_zones: ZoneScope,
    ) -> Result<(), DeviceAttestationError> {
        let parent_env = CapabilityEnvelope {
            runs_sandbox: false,
            holds_zones: parent_zones,
            online_uptime: UptimePolicy::AlwaysOn,
            runs_atrium_peer: false,
        };
        let child_env = CapabilityEnvelope {
            runs_sandbox: false,
            holds_zones: child_zones,
            online_uptime: UptimePolicy::AlwaysOn,
            runs_atrium_peer: false,
        };
        DeviceAttestation::issue_with_authority(
            parent,
            device.public_key().to_did(),
            child_env,
            &parent_env,
        )
        .map(|_| ())
    }

    // Helpers.
    let full = || ZoneScope::Full;
    let cache = || ZoneScope::CacheOnly;
    let spec = |zones: &[&str]| ZoneScope::Specific(zones.iter().map(|s| s.to_string()).collect());

    // (Full, *) → never widens.
    assert!(try_issue(&parent_keypair, &device_keypair, full(), full()).is_ok());
    assert!(try_issue(&parent_keypair, &device_keypair, full(), cache()).is_ok());
    assert!(try_issue(&parent_keypair, &device_keypair, full(), spec(&["z1"])).is_ok());

    // (CacheOnly, *) → only CacheOnly→CacheOnly is allowed.
    assert!(try_issue(&parent_keypair, &device_keypair, cache(), cache()).is_ok());
    assert!(try_issue(&parent_keypair, &device_keypair, cache(), full()).is_err());
    assert!(try_issue(&parent_keypair, &device_keypair, cache(), spec(&["z1"])).is_err());

    // (Specific(_), CacheOnly) → narrowing; OK.
    assert!(try_issue(&parent_keypair, &device_keypair, spec(&["z1"]), cache()).is_ok());
    assert!(try_issue(&parent_keypair, &device_keypair, spec(&[]), cache()).is_ok());

    // (Specific(_), Full) → widening.
    assert!(try_issue(&parent_keypair, &device_keypair, spec(&["z1"]), full()).is_err());
    assert!(try_issue(&parent_keypair, &device_keypair, spec(&[]), full()).is_err());

    // (Specific(p), Specific(c)) → widens iff c contains zone outside p.
    assert!(
        try_issue(
            &parent_keypair,
            &device_keypair,
            spec(&["z1", "z2"]),
            spec(&["z1"])
        )
        .is_ok()
    );
    assert!(
        try_issue(
            &parent_keypair,
            &device_keypair,
            spec(&["z1"]),
            spec(&["z1", "z2"])
        )
        .is_err()
    );
    assert!(try_issue(&parent_keypair, &device_keypair, spec(&[]), spec(&[])).is_ok());
    assert!(try_issue(&parent_keypair, &device_keypair, spec(&[]), spec(&["z1"])).is_err());
}

// ---------------------------------------------------------------------
// Hyg-1 #336 / F-FWD-2-01 #1051 — DeviceRevocation signature
// authenticity wired into BOTH the chain-walker
// (`validate_chain_with_device_revocations`) AND the `Acceptor::accept_at`
// revocation step. SUBSTANTIVE runtime-arm pins (pim-2 §3.6b): the
// production revocation-consumption paths are exercised; the observable
// consequence is that a FORGED revocation does NOT revoke; each test
// would FAIL against the pre-fix (device_did-byte-compare-only)
// implementation, which honored ANY byte-blob with a matching
// device_did regardless of signature.
// ---------------------------------------------------------------------

#[test]
fn chain_walker_ignores_forged_device_revocation_unsigned_by_parent() {
    // ATTACK PATH (#336): an attacker who knows a victim device's DID
    // synthesizes a DeviceRevocation byte-blob (device_did = victim,
    // parent_did = a real did:key the attacker controls, signature =
    // a real sig by the ATTACKER, NOT the device's actual parent).
    // Pre-fix the chain-walker honored it on device_did match alone →
    // perpetual DoS of the victim device's UCANs. Post-fix the forged
    // revocation does NOT verify against the parent_did it claims
    // (because the device's real parent never signed it) — but here
    // the sharper attack is: attacker sets parent_did to attacker's
    // OWN did and signs with attacker's key, so the signature DOES
    // verify against the claimed parent_did. That must STILL not
    // revoke because the chain-walk only cares that *a* genuine
    // parent signed — the trust anchor is whether the revocation's
    // signature is authentic for the parent_did it names. We model
    // the simpler forgery: a bogus signature that verifies against
    // NOTHING.
    let real_parent = Keypair::generate();
    let device = Keypair::generate();
    let leaf_aud = Keypair::generate();

    // Genuine revocation by the real parent — used as a SHAPE
    // template the attacker captures, then corrupts the signature.
    let genuine = DeviceRevocation::issue(
        &real_parent,
        device.public_key().to_did(),
        RevocationReason::Compromise,
    )
    .unwrap();
    let mut forged = genuine.clone();
    forged.signature[0] ^= 0xFF; // no longer a valid real_parent sig

    let device_ucan = Ucan::builder()
        .issuer(device.public_key().to_did().as_str())
        .audience(leaf_aud.public_key().to_did().as_str())
        .capability("/zone/posts", "read")
        .not_before(0)
        .expiry(u64::MAX)
        .sign(&device);

    // Pre-fix: this returned Err(IssuerDeviceRevoked). Post-fix the
    // forged revocation is SKIPPED (fail-CLOSED against forgery) so
    // the chain validates cleanly — the victim is NOT DoS'd.
    let res = validate_chain_with_device_revocations(&[device_ucan], &[forged]);
    assert!(
        res.is_ok(),
        "forged (bad-signature) device revocation MUST NOT revoke the device: {res:?}"
    );
}

#[test]
fn chain_walker_honors_genuine_signed_device_revocation() {
    // POSITIVE arm: a genuinely parent-signed revocation STILL
    // revokes (the authenticity gate does not regress legitimate
    // revocations). Guards against a fix that simply disables
    // revocation enforcement.
    let parent = Keypair::generate();
    let device = Keypair::generate();
    let leaf_aud = Keypair::generate();

    let genuine = DeviceRevocation::issue(
        &parent,
        device.public_key().to_did(),
        RevocationReason::DeviceLoss,
    )
    .unwrap();

    let device_ucan = Ucan::builder()
        .issuer(device.public_key().to_did().as_str())
        .audience(leaf_aud.public_key().to_did().as_str())
        .capability("/zone/posts", "read")
        .not_before(0)
        .expiry(u64::MAX)
        .sign(&device);

    let err = validate_chain_with_device_revocations(&[device_ucan], &[genuine]).unwrap_err();
    assert!(
        matches!(err, UcanError::IssuerDeviceRevoked { .. }),
        "genuine signed revocation MUST still revoke: {err:?}"
    );
}

#[test]
fn acceptor_ignores_forged_device_revocation_unsigned_by_parent() {
    // #336 second wired site: the Acceptor::accept_at revocation step
    // also verifies the revocation signature before honoring it.
    let parent = Keypair::generate();
    let device = Keypair::generate();

    let attestation = DeviceAttestation::issue(
        &parent,
        device.public_key().to_did(),
        CapabilityEnvelope::default(),
    )
    .unwrap();

    let genuine = DeviceRevocation::issue(
        &parent,
        device.public_key().to_did(),
        RevocationReason::Compromise,
    )
    .unwrap();
    let mut forged = genuine.clone();
    forged.signature[0] ^= 0xFF;

    // Pre-fix: accept() returned Err(DeviceRevoked) because the
    // device_did matched. Post-fix the forged revocation is skipped,
    // so the attestation is NOT rejected on the revocation arm (it
    // passes the revocation gate and proceeds; with u64::MAX
    // freshness + genuine attestation signature it accepts cleanly).
    let acceptor = Acceptor::new_with_revocations(FreshnessPolicy::seconds(u64::MAX), vec![forged]);
    assert!(
        acceptor.accept(&attestation).is_ok(),
        "forged device revocation MUST NOT cause Acceptor to reject a genuine attestation"
    );

    // Sanity: the GENUINE revocation still causes rejection.
    let acceptor_genuine =
        Acceptor::new_with_revocations(FreshnessPolicy::seconds(u64::MAX), vec![genuine]);
    let err = acceptor_genuine.accept(&attestation).unwrap_err();
    assert!(
        matches!(err, DeviceAttestationError::DeviceRevoked { .. }),
        "genuine revocation still rejects: {err:?}"
    );
}

// ---------------------------------------------------------------------
// Safe-1 #515 — Acceptor::accept_at expected_parent compare ct-eq
// UNIFORMITY. Behavioral pin: the expected-parent gate still rejects a
// non-matching parent and accepts a matching one after routing through
// `ct_signature_eq` (a fix that breaks the compare semantics would
// FAIL this). The grep-audit widening in `tests/ucan.rs` pins the
// constant-time SHAPE; this pins the BEHAVIOR is preserved.
// ---------------------------------------------------------------------

#[test]
fn acceptor_expected_parent_ct_eq_preserves_reject_and_accept_behavior() {
    let real_parent = Keypair::generate();
    let other_parent = Keypair::generate();
    let device = Keypair::generate();

    // Attestation signed by `other_parent` but the acceptor expects
    // `real_parent` → IssuerNotParent (mismatch arm, now ct-eq'd).
    let mismatched = DeviceAttestation::issue(
        &other_parent,
        device.public_key().to_did(),
        CapabilityEnvelope::default(),
    )
    .unwrap();
    let acceptor = Acceptor::with_parent_lookup(real_parent.public_key().to_did());
    let err = acceptor.accept(&mismatched).unwrap_err();
    assert!(
        matches!(err, DeviceAttestationError::IssuerNotParent { .. }),
        "ct-eq expected-parent gate must still REJECT a non-matching parent: {err:?}"
    );

    // Attestation signed by the expected parent → passes the
    // expected-parent gate (match arm, now ct-eq'd).
    let matched = DeviceAttestation::issue(
        &real_parent,
        device.public_key().to_did(),
        CapabilityEnvelope::default(),
    )
    .unwrap();
    let acceptor_ok = Acceptor::with_parent_lookup(real_parent.public_key().to_did());
    assert!(
        acceptor_ok.accept(&matched).is_ok(),
        "ct-eq expected-parent gate must still ACCEPT a matching parent"
    );
}
