//! COLLAPSE (P2) CONSOLIDATE — moved chain-authority behavioral pins
//! + §3.6b closure-pin (pim-2, would-FAIL-if-reverted).
//!
//! # Charter
//!
//! Spec: `.addl/refinement-audit-2026-05/impl-design-COLLAPSE.md` §2
//! (the CONSOLIDATE move) + `DECISION-RECORD-trust-model-reframe.md`
//! §4 (RATIFIED). COLLAPSE P2 MOVED
//! `benten_id::ucan::validate_chain_with_rotation_log` +
//! `benten_id::ucan::validate_chain_with_attestations` into
//! `benten_caps::chain_authority` (the latter generalized + renamed to
//! `validate_chain_with_envelope_ceiling`). These are policy-bearing
//! authority-surface consultations; the CONSOLIDATE line is *pure
//! crypto/structural validation = benten-id; policy-bearing authority
//! consultation = benten-caps*.
//!
//! These two tests are the behavioral-parity successors of the deleted
//! `benten-id/tests/did_rotation.rs::superseded_did_cannot_sign_new_ucan_delegations`
//! and the deleted
//! `benten-id/tests/device_attestation.rs` consume-time pins
//! (`device_attestation_consumed_at_ucan_delegation_chain_walk` +
//! `..._downgrade_attack_blocked_by_runtime_recheck_against_parent_chain`).
//! They moved with the functions (benten-id can no longer call them —
//! the crate dependency arrow is benten-caps → benten-id, never the
//! reverse).
//!
//! # The §3.6b closure-pin (pim-2)
//!
//! `collapse_p2_envelope_ceiling_rejects_sandbox_from_thin_principal`
//! is the would-FAIL-if-reverted pin: it asserts the moved+generalized
//! `validate_chain_with_envelope_ceiling` enforces the IDENTICAL
//! load-bearing CLAUDE.md #17 thin-shape property the pre-COLLAPSE
//! benten-id walker enforced — a `runs_sandbox=false` principal cannot
//! exercise `host:sandbox:*` even through an otherwise structurally
//! valid chain. If the move dropped or weakened the ceiling-AND
//! (e.g. returned `Ok(())` unconditionally, or only ran
//! `validate_chain_no_time_check` without the envelope loop), this
//! test FAILs — the ceiling regression cannot land silently.

#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

use benten_caps::chain_authority::{
    envelope_ceiling_rejects_cap, validate_chain_with_envelope_ceiling,
    validate_chain_with_rotation_log,
};
use benten_id::device_attestation::{CapabilityEnvelope, DeviceAttestation, ZoneScope};
use benten_id::did_rotation::{RotationLog, rotate_keypair};
use benten_id::errors::UcanError;
use benten_id::keypair::Keypair;
use benten_id::ucan::Ucan;

#[test]
fn superseded_did_cannot_sign_new_ucan_delegations() {
    // crypto-major-3 (moved from benten-id at COLLAPSE P2) —
    // post-rotation old-key UCAN rejected at chain-walk via rotation
    // log. Behavioral parity with the deleted benten-id pin.
    let old_kp = Keypair::generate();
    let new_kp = Keypair::generate();
    let leaf_aud = Keypair::generate();
    let did = old_kp.public_key().to_did();
    let now = 1_000_000_000;

    let attestation = rotate_keypair(&did, &old_kp, &new_kp, now).unwrap();
    let log = RotationLog::from_entries(vec![attestation]);

    // Attacker holding old_kp tries to issue a new UCAN AFTER rotation:
    let post_rotation_ucan = Ucan::builder()
        .issuer(old_kp.public_key().to_did().as_str())
        .audience(leaf_aud.public_key().to_did().as_str())
        .capability("/zone/posts", "read")
        .not_before(now + 100)
        .expiry(now + 3600)
        .sign(&old_kp);

    let err = validate_chain_with_rotation_log(&[post_rotation_ucan], &log).unwrap_err();
    assert!(
        matches!(err, UcanError::IssuerKeypairSuperseded { .. }),
        "{err:?}"
    );
}

#[test]
fn device_attestation_consumed_at_ucan_delegation_chain_walk() {
    // exploration-device-mesh (moved from benten-id at COLLAPSE P2) —
    // the moved+generalized chain-walker rejects UCANs that exceed the
    // device's declared envelope.
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

    let err = validate_chain_with_envelope_ceiling(&[ucan], &[attestation]).unwrap_err();
    assert!(
        matches!(err, UcanError::DeviceEnvelopeViolated { .. }),
        "{err:?}"
    );
}

#[test]
fn device_attestation_capability_envelope_downgrade_attack_blocked_by_runtime_recheck_against_parent_chain()
 {
    // sec-r4r1-6 (moved from benten-id at COLLAPSE P2) — runtime
    // re-check against attestation envelope at the moved seam.
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

    let err = validate_chain_with_envelope_ceiling(&[invocation_ucan], &[attestation]).unwrap_err();
    assert!(
        matches!(err, UcanError::DeviceEnvelopeViolated { .. }),
        "{err:?}"
    );
}

/// §3.6b closure-pin (pim-2, would-FAIL-if-reverted). The CONSOLIDATE
/// move MUST preserve the IDENTICAL chain-validation + envelope-ceiling
/// semantics. This is the single load-bearing property: the moved
/// `validate_chain_with_envelope_ceiling` AND-s the
/// `runs_sandbox=false → reject host:sandbox:*` ceiling into chain
/// validation (CLAUDE.md #17 thin-shape property; the seam P5 #669
/// extends as ONE code path, build-constraint iii).
///
/// If the move regressed the ceiling-AND — dropped the envelope loop,
/// returned `Ok(())` unconditionally, or only ran the structural
/// `validate_chain_no_time_check` without consulting the
/// envelope-ceiling — the negative arm below FAILs (the sandbox cap
/// would be accepted). The positive arms pin that a structurally valid
/// chain WITHOUT a ceiling-violating cap, and a `runs_sandbox=true`
/// envelope, both still pass (no over-rejection regression).
#[test]
fn collapse_p2_envelope_ceiling_rejects_sandbox_from_thin_principal() {
    let parent = Keypair::generate();
    let thin_device = Keypair::generate();
    let leaf_aud = Keypair::generate();

    // ---- Predicate-level pin (the factored unified seam P5 reuses) --
    let thin_env = CapabilityEnvelope {
        runs_sandbox: false,
        ..CapabilityEnvelope::default()
    };
    let full_env = CapabilityEnvelope {
        runs_sandbox: true,
        ..CapabilityEnvelope::default()
    };
    assert!(
        envelope_ceiling_rejects_cap(&thin_env, "host:sandbox:exec"),
        "thin (runs_sandbox=false) MUST reject host:sandbox:* — the \
         CLAUDE.md #17 load-bearing thin-shape ceiling"
    );
    assert!(
        !envelope_ceiling_rejects_cap(&full_env, "host:sandbox:exec"),
        "runs_sandbox=true principal may exercise host:sandbox:* (no \
         over-rejection regression)"
    );
    assert!(
        !envelope_ceiling_rejects_cap(&thin_env, "/zone/posts"),
        "non-sandbox cap is NOT gated by the runs_sandbox ceiling"
    );

    // ---- Chain-level negative arm (would-FAIL-if-reverted) ----------
    let attestation =
        DeviceAttestation::issue(&parent, thin_device.public_key().to_did(), thin_env.clone())
            .unwrap();
    let sandbox_ucan = Ucan::builder()
        .issuer(thin_device.public_key().to_did().as_str())
        .audience(leaf_aud.public_key().to_did().as_str())
        .capability("host:sandbox:exec", "*")
        .not_before(0)
        .expiry(u64::MAX)
        .sign(&thin_device);
    let err =
        validate_chain_with_envelope_ceiling(&[sandbox_ucan], std::slice::from_ref(&attestation))
            .expect_err(
                "REGRESSION: the moved CONSOLIDATE seam dropped the \
             envelope-ceiling AND — a runs_sandbox=false principal \
             was allowed host:sandbox:* (CLAUDE.md #17 thin-shape \
             ceiling silently lost across the benten-id → benten-caps \
             move). pim-2 §3.6b closure-pin.",
            );
    assert!(
        matches!(err, UcanError::DeviceEnvelopeViolated { .. }),
        "expected DeviceEnvelopeViolated, got {err:?}"
    );

    // ---- Chain-level positive arm (no over-rejection) --------------
    let benign_ucan = Ucan::builder()
        .issuer(thin_device.public_key().to_did().as_str())
        .audience(leaf_aud.public_key().to_did().as_str())
        .capability("/zone/posts", "read")
        .not_before(0)
        .expiry(u64::MAX)
        .sign(&thin_device);
    validate_chain_with_envelope_ceiling(&[benign_ucan], &[attestation]).expect(
        "a non-sandbox cap from a thin principal MUST still pass the \
         moved seam (the move preserved the structural+attenuation \
         primitive call and only gates host:sandbox:*)",
    );
}
