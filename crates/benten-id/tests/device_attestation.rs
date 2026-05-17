//! G14-A2 wave-4a' — device-DID capability-attestation test pins
//! (un-ignored except where noted).
//!
//! Pin sources (per `crypto-major-6` + `br-r4-r1-4` / `br-r4-r2-3`
//! MAJOR + the device-DID-attestation-replay defect-class +
//! `pim-r1-pim-induction-7` + `cap-r4-7`):
//!
//! COLLAPSE (P3): the `Acceptor` / `DeviceRevocation` test families
//! (replay-resistance / revocation-emission / forged-revocation /
//! expected-parent / self-re-attestation) were DELETED with the
//! device-attestation *acceptance* pipe — see the inline COLLAPSE
//! notes below. Surviving pins exercise the kept primitives:
//!
//! - `device_attestation_round_trip`
//! - `device_attestation_envelope_must_be_attenuated_by_parent_did`
//! - `device_attestation_widening_parent_authority_is_rejected`
//! - `browser_target_auto_asserts_runs_sandbox_false`
//! - `browser_target_with_runs_sandbox_true_claim_rejected_at_attestation_construction_time`
//! - `envelope_widens_zone_scope_matrix`
//! - `ucan_delegation_to_browser_target_for_sandbox_handler_rejected_at_chain_construction_not_invocation` — RED-PHASE (G14-B integration)
//!
//! COLLAPSE (P2 CONSOLIDATE): the two consume-time chain-walk pins
//! (`device_attestation_consumed_at_ucan_delegation_chain_walk` +
//! `device_attestation_capability_envelope_downgrade_attack_blocked_by_runtime_recheck_against_parent_chain`)
//! moved to `benten-caps/tests/collapse_p2_consolidate_chain_authority.rs`
//! with the `validate_chain_with_attestations` function (renamed
//! `validate_chain_with_envelope_ceiling`). benten-id cannot call the
//! moved function (the crate dependency arrow is benten-caps →
//! benten-id). The `DeviceAttestation` / `CapabilityEnvelope` *types*
//! + `issue*` / `envelope_widens` / round-trip stay benten-id
//! primitives and keep full coverage here.

#![allow(clippy::unwrap_used)]

use benten_id::DeviceAttestationError;
use benten_id::device_attestation::{
    CapabilityEnvelope, DeviceAttestation, RuntimeTarget, UptimePolicy, ZoneScope,
};
use benten_id::keypair::Keypair;

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

// COLLAPSE P2 CONSOLIDATE: `device_attestation_consumed_at_ucan_delegation_chain_walk`
// moved to `benten-caps/tests/collapse_p2_consolidate_chain_authority.rs`
// with `validate_chain_with_attestations` (renamed
// `validate_chain_with_envelope_ceiling`). The `DeviceAttestation`
// issue/round-trip/envelope primitives stay pinned here.

// COLLAPSE (P3): `device_attestation_replay_resistant_within_freshness_window`
// + `device_attestation_replay_resistance_via_nonce_freshness_window`
// DELETED with `Acceptor`. Freshness/stale-frame replay defense is
// re-homed into `benten_engine::engine_sync::DeviceAttestationEnvelope`
// `::verify` (covered by `benten-engine/tests/device_attestation_envelope_direct.rs`).

// COLLAPSE (P3): `device_attestation_revocation_emitted_by_parent_did_on_loss_event`
// DELETED with `DeviceRevocation`. Device-key revocation flows through
// user-root UCAN-grant revocation (`benten-caps::revoke`).

// NOTE (COLLAPSE-WITH-RESIDUAL, refinement-audit-2026-05 S3 P1): the
// `device_attestation_revoked_device_cannot_sign_new_ucan_delegation`
// test was deleted with the `validate_chain_with_device_revocations`
// standalone walker (the #1230 un-anchored device-revocation pipe).
// Device-key revocation now flows through user-root UCAN-grant
// revocation (`benten-caps::revoke`); coverage moves there.

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

// COLLAPSE (P3): `device_attestation_runs_sandbox_false_cannot_be_widened_by_device_signed_re_attestation`
// DELETED with `Acceptor::with_parent_lookup` (Option-B parent-lookup,
// rejected per design-1230 §1). Envelope-widening at ISSUANCE is still
// pinned by `device_attestation_widening_parent_authority_is_rejected`
// (`issue_with_authority`); runtime ceiling-AND is pinned by the P3
// closure-pin in `benten-engine` (inbound `runs_sandbox=false` rejects
// `host:sandbox:*` at the single chain-validation seam).

// COLLAPSE P2 CONSOLIDATE: `device_attestation_capability_envelope_downgrade_attack_blocked_by_runtime_recheck_against_parent_chain`
// moved to `benten-caps/tests/collapse_p2_consolidate_chain_authority.rs`
// with `validate_chain_with_attestations` (renamed
// `validate_chain_with_envelope_ceiling`). The runtime ceiling-AND is
// now pinned at the consolidated benten-caps seam (+ the P2
// would-FAIL-if-reverted §3.6b closure-pin there).

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
#[ignore = "Hyg-4 #478 trigger-retense: destination phase-3-backlog §2.1-followup `ssi` external UCAN/VC spec compatibility re-evaluation STILL OPEN, but the prior 'Phase 3 G16 Atrium-handshake re-evaluation point' trigger has PASSED (Phase 3 + Phase-4-Foundation SHIPPED). Note: under COLLAPSE the device-attestation *acceptance* pipe is deleted; the `with_attestation_lookup` construction-time gate referenced below is moot post-COLLAPSE (the surviving gate is the validate-side `validate_chain_with_attestations`). Current trigger = the v1-assessment-window in Phase-4-Meta. Un-ignore at that v1-assessment-window re-evaluation. — production prerequisite NOT YET shipped at HEAD. The `Ucan::builder().with_attestation_lookup(...)` construction-time gate + `DelegationError::AudienceEnvelopeIncompatibleWithCapability` typed error do NOT exist (only mentioned in `crates/benten-id/src/ucan.rs:33-37` doc comment; no symbol). the validate-side envelope-ceiling seam DOES exist (post-COLLAPSE-P2 CONSOLIDATE it MOVED from `benten-id::ucan::validate_chain_with_attestations` to `benten_caps::chain_authority::validate_chain_with_envelope_ceiling`) — runtime gate before trust-graph dispatch. Construction-time rejection composes with §2.1-followup re-evaluation outcome (G16-D wave-6b PR #163 shipped 2026-05-09; cryptography-reviewer dispatch pending). Un-ignore at §2.1-followup re-evaluation that determines `ssi` integration is needed (or, if hand-rolled remains, the `with_attestation_lookup` chain-construction-time path lands as a benten-id-internal extension)."]
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

// COLLAPSE (P3): `acceptor_rejects_attestation_with_forged_signature`
// DELETED with `Acceptor`. The embedded-attestation signature gate is
// re-homed into `DeviceAttestationEnvelope::verify` (parent_did sig
// verify); `DeviceAttestation::verify_signature_with` still has direct
// coverage. Negative-path pinned in device_attestation_envelope_direct.rs.

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
// COLLAPSE (P3) — DELETED device-trust *acceptance*-pipe pins:
//
// - Hyg-1 #336 / F-FWD-2-01 #1051 `DeviceRevocation` signature
//   authenticity: both the chain-walker half (deleted at COLLAPSE P1
//   with `validate_chain_with_device_revocations`) AND the
//   `Acceptor::accept_at` revocation-step half
//   (`acceptor_ignores_forged_device_revocation_unsigned_by_parent`)
//   are gone — the device-revocation parallel pipe is dissolved;
//   device-key revocation now flows through user-root UCAN-grant
//   revocation (`benten-caps::revoke`).
// - Safe-1 #515 `Acceptor::accept_at` expected_parent ct-eq behavior
//   (`acceptor_expected_parent_ct_eq_preserves_reject_and_accept_behavior`):
//   the expected-parent gate was never production-wired (design-1230
//   §1 fact 3); the embedded attestation's parent_did is now verified
//   as a delegation link inside
//   `benten_engine::engine_sync::DeviceAttestationEnvelope::verify`
//   (the ct-eq UNIFORMITY SHAPE remains pinned by `tests/ucan.rs`).
// ---------------------------------------------------------------------
